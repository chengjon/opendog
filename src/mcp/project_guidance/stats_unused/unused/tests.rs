use super::*;
use serde_json::json;

fn make_entry(path: &str, access_count: i64) -> StatsEntry {
    StatsEntry {
        file_path: path.to_string(),
        size: 100,
        file_type: "file".to_string(),
        access_count,
        estimated_duration_ms: 0,
        modification_count: 0,
        last_access_time: None,
        first_seen_time: None,
    }
}

// --- count_classification ---

#[test]
fn count_classification_empty_entries() {
    let entries: Vec<StatsEntry> = vec![];
    assert_eq!(
        count_classification(&entries, FilePathClassification::Source),
        0
    );
    assert_eq!(
        count_classification(&entries, FilePathClassification::Infrastructure),
        0
    );
    assert_eq!(
        count_classification(&entries, FilePathClassification::Backup),
        0
    );
    assert_eq!(
        count_classification(&entries, FilePathClassification::Project),
        0
    );
}

#[test]
fn count_classification_counts_source_files() {
    let entries = vec![
        make_entry("src/main.rs", 5),
        make_entry("lib/app.py", 3),
        make_entry("index.js", 1),
    ];
    assert_eq!(
        count_classification(&entries, FilePathClassification::Source),
        3
    );
    assert_eq!(
        count_classification(&entries, FilePathClassification::Infrastructure),
        0
    );
}

#[test]
fn count_classification_counts_infrastructure_files() {
    let entries = vec![
        make_entry(".claude/settings.json", 0),
        make_entry(".cursor/rules/guide.mdc", 0),
        make_entry("src/main.rs", 5),
    ];
    assert_eq!(
        count_classification(&entries, FilePathClassification::Infrastructure),
        2
    );
    assert_eq!(
        count_classification(&entries, FilePathClassification::Source),
        1
    );
}

#[test]
fn count_classification_counts_backup_files() {
    let entries = vec![
        make_entry("notes.txt~", 0),
        make_entry("config.yaml.bak", 0),
        make_entry("src/main.rs", 5),
    ];
    assert_eq!(
        count_classification(&entries, FilePathClassification::Backup),
        2
    );
}

#[test]
fn count_classification_counts_project_files() {
    let entries = vec![
        make_entry("README", 0),
        make_entry("LICENSE", 0),
        make_entry("Makefile", 0),
        make_entry("src/main.rs", 5),
    ];
    assert_eq!(
        count_classification(&entries, FilePathClassification::Project),
        3
    );
}

#[test]
fn count_classification_mixed_entries() {
    let entries = vec![
        make_entry("src/main.rs", 10),
        make_entry("lib/utils.py", 5),
        make_entry(".claude/CLAUDE.md", 0),
        make_entry("config.toml.bak", 0),
        make_entry("Cargo.toml", 0),
        make_entry("README", 0),
    ];
    assert_eq!(
        count_classification(&entries, FilePathClassification::Source),
        2
    );
    assert_eq!(
        count_classification(&entries, FilePathClassification::Infrastructure),
        1
    );
    assert_eq!(
        count_classification(&entries, FilePathClassification::Backup),
        1
    );
    assert_eq!(
        count_classification(&entries, FilePathClassification::Project),
        2
    );
}

// --- apply_path_filter_observation ---

#[test]
fn apply_path_filter_all_does_not_mutate_guidance() {
    let mut guidance = json!({
        "layers": {
            "workspace_observation": {},
            "verification_evidence": {}
        }
    });
    apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::All, 5);
    assert!(guidance["layers"]["workspace_observation"]
        .get("path_classification_filter")
        .is_none());
    assert!(guidance["layers"]["workspace_observation"]
        .get("filter_note")
        .is_none());
}

#[test]
fn apply_path_filter_source_sets_filter_field() {
    let mut guidance = json!({
        "layers": {
            "workspace_observation": {},
            "verification_evidence": {}
        }
    });
    apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::Source, 5);
    assert_eq!(
        guidance["layers"]["workspace_observation"]["path_classification_filter"],
        "source"
    );
}

#[test]
fn apply_path_filter_infrastructure_sets_filter_field() {
    let mut guidance = json!({
        "layers": {
            "workspace_observation": {},
            "verification_evidence": {}
        }
    });
    apply_path_filter_observation(
        &mut guidance,
        FilePathClassificationFilter::Infrastructure,
        3,
    );
    assert_eq!(
        guidance["layers"]["workspace_observation"]["path_classification_filter"],
        "infrastructure"
    );
}

#[test]
fn apply_path_filter_with_zero_rows_adds_filter_note() {
    let mut guidance = json!({
        "layers": {
            "workspace_observation": {},
            "verification_evidence": {}
        }
    });
    apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::Backup, 0);
    assert!(guidance["layers"]["workspace_observation"]["filter_note"]
        .as_str()
        .unwrap()
        .contains("filter returned no rows"));
    assert!(guidance["layers"]["verification_evidence"]["inferences"]
        .as_array()
        .is_some());
}

#[test]
fn apply_path_filter_with_nonzero_rows_does_not_add_filter_note() {
    let mut guidance = json!({
        "layers": {
            "workspace_observation": {},
            "verification_evidence": {}
        }
    });
    apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::Project, 7);
    assert!(guidance["layers"]["workspace_observation"]
        .get("filter_note")
        .is_none());
}
