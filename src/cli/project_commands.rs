use std::path::Path;

use crate::contracts::{CLI_CLEANUP_PROJECT_DATA_V1, CLI_EXPORT_PROJECT_EVIDENCE_V1};
use crate::control::{
    CliProjectLifecycle, DaemonClient, DaemonProjectLifecycle, FallbackLifecycle,
    ProjectLifecycle, SnapshotMonitor,
};
use crate::core::export::{self, ExportFormat, ExportView};
use crate::core::file_classification::{classify_file_path, FilePathClassificationFilter};
use crate::core::monitor;
use crate::core::project::ProjectManager;
use crate::core::retention::{self, ProjectDataCleanupRequest};
use crate::core::stats;
use crate::error::OpenDogError;
use crate::mcp::export_project_evidence_payload;
use crate::storage::queries::StatsEntry;

use super::output;

fn project_lifecycle(pm: &ProjectManager) -> FallbackLifecycle<DaemonProjectLifecycle<'static>, CliProjectLifecycle<'_>> {
    static DAEMON: std::sync::OnceLock<DaemonClient> = std::sync::OnceLock::new();
    let client = DAEMON.get_or_init(DaemonClient::new);
    FallbackLifecycle::new(
        DaemonProjectLifecycle::new(client),
        CliProjectLifecycle::new(pm),
    )
}

pub(super) fn cmd_register(pm: &ProjectManager, id: &str, path: &str) -> Result<(), OpenDogError> {
    let svc = project_lifecycle(pm);
    let info = svc.create_project(id, path)?;
    output::print_registered(&info);
    Ok(())
}

pub(super) fn cmd_snapshot(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let svc = project_lifecycle(pm);
    let result = svc.take_snapshot(id)?;
    output::print_snapshot_result(id, &result);
    Ok(())
}

pub(super) fn cmd_start(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    match daemon.start_monitor(id) {
        Ok(outcome) => {
            if outcome.already_running {
                println!(
                    "Daemon-managed monitor already running for project '{}'.",
                    id
                );
            } else if outcome.snapshot_taken {
                println!(
                    "Started daemon-managed monitor for project '{}' after taking an initial snapshot.",
                    id
                );
            } else {
                println!("Started daemon-managed monitor for project '{}'.", id);
            }
            return Ok(());
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return Err(e),
    }

    let info = pm
        .get(id)?
        .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
    let effective_config = pm.resolve_project_config(&info)?;
    let db = pm.open_project_db(id)?;
    drop(db);

    println!("Starting monitor for project '{}'...", id);
    let handle = monitor::start_monitor(&info.db_path, info.root_path.clone(), effective_config)?;
    let stop_requested = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_requested_handler = stop_requested.clone();
    ctrlc::set_handler(move || {
        stop_requested_handler.store(true, std::sync::atomic::Ordering::Relaxed);
    })
    .map_err(|e| OpenDogError::Io(std::io::Error::other(e.to_string())))?;

    println!("Monitor running. Press Ctrl+C to stop.");
    while handle.is_running() && !stop_requested.load(std::sync::atomic::Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    handle.stop();
    println!("Monitor stopped.");
    Ok(())
}

pub(super) fn cmd_stop(id: &str) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    match daemon.stop_monitor(id)? {
        true => println!("Stopped daemon-managed monitor for project '{}'.", id),
        false => println!("No daemon-managed monitor running for project '{}'.", id),
    }
    Ok(())
}

pub(super) fn cmd_export(
    pm: &ProjectManager,
    id: &str,
    format: &str,
    view: &str,
    output_path: &str,
    min_access_count: i64,
) -> Result<(), OpenDogError> {
    let format = ExportFormat::parse(format)?;
    let view = ExportView::parse(view)?;
    let db = pm.open_project_db(id)?;
    let summary = stats::get_summary(&db)?;
    let rows = export::export_rows(&db, view, min_access_count)?;
    let artifact = export::build_portable_export(id, format, view, summary, rows.clone());
    let content = match format {
        ExportFormat::Json => export::render_json_export(&artifact)?,
        ExportFormat::Csv => export::render_csv_export(&rows),
    };

    let bytes_written = export::write_export_file(Path::new(output_path), &content)?;
    let payload = export_project_evidence_payload(
        CLI_EXPORT_PROJECT_EVIDENCE_V1,
        &artifact,
        output_path,
        bytes_written,
        &content,
    );

    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

pub(super) fn cmd_cleanup_data(
    pm: &ProjectManager,
    id: &str,
    request: ProjectDataCleanupRequest,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    let result = match daemon.cleanup_project_data(id, request.clone()) {
        Ok(result) => result,
        Err(OpenDogError::DaemonUnavailable) => {
            let db = pm.open_project_db(id)?;
            retention::cleanup_project_data(&db, &request)?
        }
        Err(error) => return Err(error),
    };

    let payload =
        crate::mcp::cleanup_project_data_payload(CLI_CLEANUP_PROJECT_DATA_V1, id, &result);
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        output::print_cleanup_data_result(id, &result);
    }
    Ok(())
}

pub(super) fn cmd_stats(
    pm: &ProjectManager,
    id: &str,
    path_classification: &str,
) -> Result<(), OpenDogError> {
    let filter = parse_path_classification_filter(path_classification)?;
    let daemon = DaemonClient::new();
    match daemon.get_stats(id) {
        Ok((summary, entries)) => {
            let filtered = filter_entries_by_classification(&entries, filter);
            output::print_stats(id, &summary, &filtered, filter, entries.len());
            return Ok(());
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return Err(e),
    }

    let db = pm.open_project_db(id)?;
    let summary = stats::get_summary(&db)?;
    let entries = stats::get_stats(&db)?;
    let filtered = filter_entries_by_classification(&entries, filter);
    output::print_stats(id, &summary, &filtered, filter, entries.len());
    Ok(())
}

pub(super) fn cmd_unused(
    pm: &ProjectManager,
    id: &str,
    path_classification: &str,
) -> Result<(), OpenDogError> {
    let filter = parse_path_classification_filter(path_classification)?;
    let daemon = DaemonClient::new();
    match daemon.get_unused_files(id) {
        Ok(unused) => {
            let filtered = filter_entries_by_classification(&unused, filter);
            output::print_unused(id, &filtered, filter, unused.len());
            return Ok(());
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return Err(e),
    }

    let db = pm.open_project_db(id)?;
    let unused = stats::get_unused_files(&db)?;
    let filtered = filter_entries_by_classification(&unused, filter);
    output::print_unused(id, &filtered, filter, unused.len());
    Ok(())
}

fn parse_path_classification_filter(
    value: &str,
) -> Result<FilePathClassificationFilter, OpenDogError> {
    FilePathClassificationFilter::parse(Some(value)).map_err(OpenDogError::InvalidInput)
}

fn filter_entries_by_classification(
    entries: &[StatsEntry],
    filter: FilePathClassificationFilter,
) -> Vec<StatsEntry> {
    entries
        .iter()
        .filter(|entry| filter.matches(classify_file_path(&entry.file_path)))
        .cloned()
        .collect()
}

pub(super) fn cmd_list(pm: &ProjectManager) -> Result<(), OpenDogError> {
    let svc = project_lifecycle(pm);
    let projects = svc.list_projects()?;
    output::print_project_list(&projects);
    Ok(())
}

pub(super) fn cmd_delete(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let svc = project_lifecycle(pm);
    match svc.delete_project(id)? {
        true => println!("Project '{}' deleted.", id),
        false => {
            eprintln!("Project '{}' not found.", id);
            std::process::exit(1);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::file_classification::FilePathClassificationFilter;
    use crate::storage::queries::StatsEntry;

    fn entry(path: &str) -> StatsEntry {
        StatsEntry {
            file_path: path.to_string(),
            size: 1,
            file_type: "txt".to_string(),
            access_count: 0,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }
    }

    #[test]
    fn filters_entries_by_path_classification_for_cli_views() {
        let entries = vec![
            entry("src/main.rs"),
            entry(".claude/settings.json"),
            entry("README.md"),
        ];

        let source =
            filter_entries_by_classification(&entries, FilePathClassificationFilter::Source);
        assert_eq!(source.len(), 1);
        assert_eq!(source[0].file_path, "src/main.rs");

        let all = filter_entries_by_classification(&entries, FilePathClassificationFilter::All);
        assert_eq!(all.len(), 3);
    }
}
