use crate::control::DaemonClient;
use crate::core::file_classification::{classify_file_path, FilePathClassificationFilter};
use crate::core::project::ProjectManager;
use crate::core::stats;
use crate::error::OpenDogError;
use crate::storage::queries::StatsEntry;

use super::super::output;

pub(in crate::cli) fn cmd_stats(
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

pub(in crate::cli) fn cmd_unused(
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

    #[test]
    fn parse_path_classification_filter_delegates_to_enum_parser() {
        assert_eq!(
            parse_path_classification_filter("source").unwrap(),
            FilePathClassificationFilter::Source
        );
        assert_eq!(
            parse_path_classification_filter("all").unwrap(),
            FilePathClassificationFilter::All
        );
        assert_eq!(
            parse_path_classification_filter("infrastructure").unwrap(),
            FilePathClassificationFilter::Infrastructure
        );
    }

    #[test]
    fn parse_path_classification_filter_rejects_invalid_value() {
        let err = parse_path_classification_filter("documents").unwrap_err();
        assert!(err
            .to_string()
            .contains("path_classification must be one of"));
    }

    #[test]
    fn filter_entries_by_classification_empty_input_returns_empty() {
        let entries: Vec<StatsEntry> = vec![];
        let filtered =
            filter_entries_by_classification(&entries, FilePathClassificationFilter::Source);
        assert!(filtered.is_empty());
    }

    #[test]
    fn filter_entries_by_classification_infrastructure_filter() {
        let entries = vec![entry(".claude/settings.json"), entry("src/main.rs")];
        let infra = filter_entries_by_classification(
            &entries,
            FilePathClassificationFilter::Infrastructure,
        );
        assert_eq!(infra.len(), 1);
        assert_eq!(infra[0].file_path, ".claude/settings.json");
    }

    #[test]
    fn filter_entries_by_classification_no_matching_entries() {
        let entries = vec![entry("README.md"), entry("LICENSE")];
        let source =
            filter_entries_by_classification(&entries, FilePathClassificationFilter::Source);
        assert!(source.is_empty());
    }
}
