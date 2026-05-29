use super::*;
use crate::storage::queries::StatsEntry;

fn make_entry(path: &str) -> StatsEntry {
    StatsEntry {
        file_path: path.to_string(),
        size: 100,
        file_type: "file".to_string(),
        access_count: 1,
        estimated_duration_ms: 0,
        modification_count: 0,
        last_access_time: None,
        first_seen_time: None,
    }
}

// --- normalized_observation_limit ---

#[test]
fn normalized_limit_zero_returns_default() {
    assert_eq!(
        normalized_observation_limit(0),
        DEFAULT_OBSERVATION_PAYLOAD_LIMIT
    );
}

#[test]
fn normalized_limit_small_value_passes_through() {
    assert_eq!(normalized_observation_limit(1), 1);
    assert_eq!(normalized_observation_limit(5), 5);
}

#[test]
fn normalized_limit_large_value_passes_through() {
    assert_eq!(normalized_observation_limit(10_000), 10_000);
}

// --- observation_result_window ---

#[test]
fn result_window_basic_counts() {
    let win = observation_result_window(100, 50, 50, FilePathClassificationFilter::All);
    assert_eq!(win["total_count"], 100);
    assert_eq!(win["returned_count"], 50);
    assert_eq!(win["limit"], 50);
    assert_eq!(win["truncated"], true);
    assert_eq!(win["path_classification"], "all");
}

#[test]
fn result_window_no_truncation_when_all_returned() {
    let win = observation_result_window(10, 10, 50, FilePathClassificationFilter::Source);
    assert_eq!(win["truncated"], false);
    assert_eq!(win["path_classification"], "source");
}

#[test]
fn result_window_zero_counts() {
    let win = observation_result_window(0, 0, 50, FilePathClassificationFilter::All);
    assert_eq!(win["total_count"], 0);
    assert_eq!(win["returned_count"], 0);
    assert_eq!(win["truncated"], false);
}

// --- classification_summary ---

#[test]
fn classification_summary_empty() {
    let summary = classification_summary(&[]);
    assert_eq!(summary["source_files"], 0);
    assert_eq!(summary["infrastructure_files"], 0);
    assert_eq!(summary["backup_files"], 0);
    assert_eq!(summary["project_files"], 0);
}

#[test]
fn classification_summary_mixed_entries() {
    let entries = vec![
        make_entry("src/main.rs"),           // Source
        make_entry("lib/app.py"),            // Source
        make_entry(".claude/settings.json"), // Infrastructure
        make_entry("notes/todo.txt.bak"),    // Backup
        make_entry("docs/README.md"),        // Project (no source extension)
    ];
    let summary = classification_summary(&entries);
    assert_eq!(summary["source_files"], 2);
    assert_eq!(summary["infrastructure_files"], 1);
    assert_eq!(summary["backup_files"], 1);
    assert_eq!(summary["project_files"], 1);
}

#[test]
fn classification_summary_all_source() {
    let entries = vec![make_entry("src/main.rs"), make_entry("lib/app.py")];
    let summary = classification_summary(&entries);
    assert_eq!(summary["source_files"], 2);
    assert_eq!(summary["infrastructure_files"], 0);
    assert_eq!(summary["backup_files"], 0);
    assert_eq!(summary["project_files"], 0);
}
