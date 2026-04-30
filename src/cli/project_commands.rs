use std::path::Path;

use crate::contracts::{CLI_CLEANUP_PROJECT_DATA_V1, CLI_EXPORT_PROJECT_EVIDENCE_V1};
use crate::control::DaemonClient;
use crate::core::export::{self, ExportFormat, ExportView};
use crate::core::monitor;
use crate::core::project::ProjectManager;
use crate::core::retention::{self, CleanupScope, ProjectDataCleanupRequest};
use crate::core::{snapshot, stats};
use crate::error::OpenDogError;
use crate::mcp::export_project_evidence_payload;

use super::output;

pub(super) fn cmd_create(pm: &ProjectManager, id: &str, path: &str) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    let info = match daemon.create_project(id, path) {
        Ok(info) => info,
        Err(OpenDogError::DaemonUnavailable) => pm.create(id, Path::new(path))?,
        Err(e) => return Err(e),
    };
    output::print_created(&info);
    Ok(())
}

pub(super) fn cmd_snapshot(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    match daemon.take_snapshot(id) {
        Ok(result) => {
            output::print_snapshot_result(id, &result);
            return Ok(());
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return Err(e),
    }

    let db = pm.open_project_db(id)?;
    let info = pm
        .get(id)?
        .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
    let effective_config = pm.resolve_project_config(&info)?;
    let result = snapshot::take_snapshot(&db, &info.root_path, &effective_config)?;
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
        id,
        format.as_str(),
        view.as_str(),
        output_path,
        bytes_written,
        artifact.row_count,
        &artifact.summary,
        &content,
    );

    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

pub(super) fn cmd_cleanup_data(
    pm: &ProjectManager,
    id: &str,
    scope: &str,
    older_than_days: Option<i64>,
    keep_snapshot_runs: Option<usize>,
    vacuum: bool,
    dry_run: bool,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let scope = CleanupScope::parse(scope)?;
    let request = ProjectDataCleanupRequest {
        scope,
        older_than_days,
        keep_snapshot_runs,
        vacuum,
        dry_run,
    };

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

pub(super) fn cmd_stats(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    match daemon.get_stats(id) {
        Ok((summary, entries)) => {
            output::print_stats(id, &summary, &entries);
            return Ok(());
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return Err(e),
    }

    let db = pm.open_project_db(id)?;
    let summary = stats::get_summary(&db)?;
    let entries = stats::get_stats(&db)?;
    output::print_stats(id, &summary, &entries);
    Ok(())
}

pub(super) fn cmd_unused(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    match daemon.get_unused_files(id) {
        Ok(unused) => {
            output::print_unused(id, &unused);
            return Ok(());
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return Err(e),
    }

    let db = pm.open_project_db(id)?;
    let unused = stats::get_unused_files(&db)?;
    output::print_unused(id, &unused);
    Ok(())
}

pub(super) fn cmd_list(pm: &ProjectManager) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    let projects = match daemon.list_projects() {
        Ok(projects) => projects,
        Err(OpenDogError::DaemonUnavailable) => pm.list()?,
        Err(e) => return Err(e),
    };
    output::print_project_list(&projects);
    Ok(())
}

pub(super) fn cmd_delete(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    match daemon.delete_project(id) {
        Ok(true) => {
            println!("Project '{}' deleted.", id);
            return Ok(());
        }
        Ok(false) => {
            eprintln!("Project '{}' not found.", id);
            std::process::exit(1);
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return Err(e),
    }

    let deleted = pm.delete(id)?;
    if deleted {
        println!("Project '{}' deleted.", id);
    } else {
        eprintln!("Project '{}' not found.", id);
        std::process::exit(1);
    }
    Ok(())
}
