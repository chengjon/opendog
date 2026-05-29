use super::*;
use serde_json::json;

// --- apply_path_filter_observation ---

#[test]
fn apply_path_filter_all_does_not_mutate_guidance() {
    let mut guidance = json!({
        "layers": {
            "workspace_observation": {},
            "verification_evidence": {}
        }
    });
    apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::All, 10, 5);
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
    apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::Source, 10, 5);
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
        10,
        3,
    );
    assert_eq!(
        guidance["layers"]["workspace_observation"]["path_classification_filter"],
        "infrastructure"
    );
}

#[test]
fn apply_path_filter_backup_sets_filter_field() {
    let mut guidance = json!({
        "layers": {
            "workspace_observation": {},
            "verification_evidence": {}
        }
    });
    apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::Backup, 10, 2);
    assert_eq!(
        guidance["layers"]["workspace_observation"]["path_classification_filter"],
        "backup"
    );
}

#[test]
fn apply_path_filter_project_sets_filter_field() {
    let mut guidance = json!({
        "layers": {
            "workspace_observation": {},
            "verification_evidence": {}
        }
    });
    apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::Project, 10, 1);
    assert_eq!(
        guidance["layers"]["workspace_observation"]["path_classification_filter"],
        "project"
    );
}

#[test]
fn apply_path_filter_with_total_files_positive_and_zero_filtered_adds_note() {
    let mut guidance = json!({
        "layers": {
            "workspace_observation": {},
            "verification_evidence": {}
        }
    });
    apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::Source, 20, 0);
    assert!(guidance["layers"]["workspace_observation"]["filter_note"]
        .as_str()
        .unwrap()
        .contains("filter returned no rows"));
    let inferences = guidance["layers"]["verification_evidence"]["inferences"]
        .as_array()
        .unwrap();
    assert!(!inferences.is_empty());
}

#[test]
fn apply_path_filter_with_zero_total_files_and_zero_filtered_does_not_add_note() {
    let mut guidance = json!({
        "layers": {
            "workspace_observation": {},
            "verification_evidence": {}
        }
    });
    // total_files == 0, filtered_rows == 0: condition is `total_files > 0 && filtered_rows == 0`
    apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::Source, 0, 0);
    // Should set the filter field but NOT the note (total_files is 0)
    assert_eq!(
        guidance["layers"]["workspace_observation"]["path_classification_filter"],
        "source"
    );
    assert!(guidance["layers"]["workspace_observation"]
        .get("filter_note")
        .is_none());
}

#[test]
fn apply_path_filter_with_nonzero_total_and_nonzero_filtered_no_note() {
    let mut guidance = json!({
        "layers": {
            "workspace_observation": {},
            "verification_evidence": {}
        }
    });
    apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::Source, 15, 10);
    assert_eq!(
        guidance["layers"]["workspace_observation"]["path_classification_filter"],
        "source"
    );
    assert!(guidance["layers"]["workspace_observation"]
        .get("filter_note")
        .is_none());
}
