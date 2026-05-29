use super::*;

fn boundary_value(boundary: ExternalTruthBoundary) -> Value {
    serde_json::to_value(boundary).unwrap()
}

// --- repo_state_triggers_for ---

#[test]
fn repo_state_triggers_filters_known_gaps() {
    let rec = serde_json::json!({
        "repo_truth_gaps": ["repository_mid_operation", "working_tree_conflicted", "some_unknown_gap"]
    });
    let triggers = repo_state_triggers_for(&rec);
    assert!(triggers.contains(&"repository_mid_operation".to_string()));
    assert!(triggers.contains(&"working_tree_conflicted".to_string()));
    assert!(!triggers.contains(&"some_unknown_gap".to_string()));
}

#[test]
fn repo_state_triggers_all_known_gaps() {
    let rec = serde_json::json!({
        "repo_truth_gaps": [
            "repository_mid_operation",
            "working_tree_conflicted",
            "dependency_state_requires_repo_review",
            "git_metadata_unavailable"
        ]
    });
    let triggers = repo_state_triggers_for(&rec);
    assert_eq!(triggers.len(), 4);
}

#[test]
fn repo_state_triggers_empty_json() {
    let rec = serde_json::json!({});
    let triggers = repo_state_triggers_for(&rec);
    assert!(triggers.is_empty());
}

#[test]
fn repo_state_triggers_non_array_returns_empty() {
    let rec = serde_json::json!({"repo_truth_gaps": "not_array"});
    let triggers = repo_state_triggers_for(&rec);
    assert!(triggers.is_empty());
}

// --- verification_trigger_for ---

#[test]
fn verification_trigger_run_then_resume() {
    let rec = serde_json::json!({
        "execution_sequence": {"mode": "run_project_verification_then_resume"}
    });
    assert_eq!(
        verification_trigger_for(&rec),
        Some("verification_run_required".to_string())
    );
}

#[test]
fn verification_trigger_resolve_failing() {
    let rec = serde_json::json!({
        "execution_sequence": {"mode": "resolve_failing_verification_then_resume"}
    });
    assert_eq!(
        verification_trigger_for(&rec),
        Some("failing_verification_repair_required".to_string())
    );
}

#[test]
fn verification_trigger_other_mode_returns_none() {
    let rec = serde_json::json!({
        "execution_sequence": {"mode": "proceed_normally"}
    });
    assert!(verification_trigger_for(&rec).is_none());
}

#[test]
fn verification_trigger_missing_execution_sequence() {
    let rec = serde_json::json!({});
    assert!(verification_trigger_for(&rec).is_none());
}

// --- verification_commands_for ---

#[test]
fn verification_commands_extracts_strings() {
    let rec = serde_json::json!({
        "execution_sequence": {
            "verification_commands": ["cargo test", "cargo clippy"]
        }
    });
    let cmds = verification_commands_for(&rec);
    assert_eq!(cmds, vec!["cargo test", "cargo clippy"]);
}

#[test]
fn verification_commands_empty_array() {
    let rec = serde_json::json!({
        "execution_sequence": {"verification_commands": []}
    });
    let cmds = verification_commands_for(&rec);
    assert!(cmds.is_empty());
}

#[test]
fn verification_commands_missing_key() {
    let rec = serde_json::json!({});
    let cmds = verification_commands_for(&rec);
    assert!(cmds.is_empty());
}

#[test]
fn verification_commands_non_array_returns_empty() {
    let rec = serde_json::json!({
        "execution_sequence": {"verification_commands": "not_array"}
    });
    let cmds = verification_commands_for(&rec);
    assert!(cmds.is_empty());
}

// --- external_truth_boundary_for_top_project ---

#[test]
fn none_returns_no_priority_project() {
    let result = boundary_value(external_truth_boundary_for_top_project(None));
    assert_eq!(result["status"], "no_priority_project");
    assert!(result["source"].is_null());
    assert!(result["source_project_id"].is_null());
    assert!(result["mode"].is_null());
    assert_eq!(result["repo_state_required"], false);
    assert_eq!(result["verification_required"], false);
    assert!(result["triggers"].as_array().unwrap().is_empty());
    assert!(result["minimum_external_checks"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(result["summary"].is_null());
}

#[test]
fn with_repo_triggers_only() {
    let rec = serde_json::json!({
        "project_id": "proj1",
        "repo_truth_gaps": ["working_tree_conflicted"],
        "mandatory_shell_checks": ["git status"],
        "execution_sequence": {"mode": "proceed_normally", "verification_commands": []}
    });
    let result = boundary_value(external_truth_boundary_for_top_project(Some(&rec)));
    assert_eq!(result["status"], "available");
    assert_eq!(result["source"], "top_priority_project");
    assert_eq!(result["source_project_id"], "proj1");
    assert_eq!(result["mode"], "must_switch_to_external_truth");
    assert_eq!(result["repo_state_required"], true);
    assert_eq!(result["verification_required"], false);
    assert!(result["summary"]
        .as_str()
        .unwrap()
        .contains("repository truth"));
}

#[test]
fn with_verification_trigger_only() {
    let rec = serde_json::json!({
        "project_id": "proj2",
        "repo_truth_gaps": [],
        "mandatory_shell_checks": [],
        "execution_sequence": {
            "mode": "run_project_verification_then_resume",
            "verification_commands": ["cargo test"]
        }
    });
    let result = boundary_value(external_truth_boundary_for_top_project(Some(&rec)));
    assert_eq!(result["status"], "available");
    assert_eq!(result["repo_state_required"], false);
    assert_eq!(result["verification_required"], true);
    assert_eq!(result["mode"], "must_switch_to_external_truth");
    assert!(result["summary"].as_str().unwrap().contains("verification"));
    // external checks include verification command
    let checks = result["minimum_external_checks"].as_array().unwrap();
    assert!(checks.iter().any(|c| c == "cargo test"));
}

#[test]
fn with_both_repo_and_verification_triggers() {
    let rec = serde_json::json!({
        "project_id": "proj3",
        "repo_truth_gaps": ["repository_mid_operation"],
        "mandatory_shell_checks": ["git status", "git diff"],
        "execution_sequence": {
            "mode": "run_project_verification_then_resume",
            "verification_commands": ["npm test"]
        }
    });
    let result = boundary_value(external_truth_boundary_for_top_project(Some(&rec)));
    assert_eq!(result["repo_state_required"], true);
    assert_eq!(result["verification_required"], true);
    assert!(result["summary"]
        .as_str()
        .unwrap()
        .contains("repository and verification"));
    let checks = result["minimum_external_checks"].as_array().unwrap();
    assert!(checks.iter().any(|c| c == "git status"));
    assert!(checks.iter().any(|c| c == "git diff"));
    assert!(checks.iter().any(|c| c == "npm test"));
}

#[test]
fn can_continue_when_no_triggers() {
    let rec = serde_json::json!({
        "project_id": "proj4",
        "repo_truth_gaps": [],
        "mandatory_shell_checks": [],
        "execution_sequence": {"mode": "proceed_normally", "verification_commands": []}
    });
    let result = boundary_value(external_truth_boundary_for_top_project(Some(&rec)));
    assert_eq!(result["mode"], "opendog_guidance_can_continue");
    assert!(result["summary"]
        .as_str()
        .unwrap()
        .contains("continue under OPENDOG"));
}

#[test]
fn minimum_external_checks_deduplicates() {
    let rec = serde_json::json!({
        "project_id": "proj5",
        "repo_truth_gaps": [],
        "mandatory_shell_checks": ["git status"],
        "execution_sequence": {
            "mode": "run_project_verification_then_resume",
            "verification_commands": ["git status"]
        }
    });
    let result = boundary_value(external_truth_boundary_for_top_project(Some(&rec)));
    let checks = result["minimum_external_checks"].as_array().unwrap();
    let git_status_count = checks
        .iter()
        .filter(|c| c.as_str() == Some("git status"))
        .count();
    assert_eq!(git_status_count, 1);
}

#[test]
fn push_once_prevents_duplicates() {
    let mut items: Vec<String> = Vec::new();
    push_once(&mut items, "alpha");
    push_once(&mut items, "alpha");
    push_once(&mut items, "beta");
    assert_eq!(items, vec!["alpha", "beta"]);
}
