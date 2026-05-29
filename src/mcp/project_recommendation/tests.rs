use super::*;
use crate::storage::queries::VerificationRun;
use serde_json::json;
use std::path::PathBuf;

fn make_state(id: &str, status: &str, root: &str) -> ProjectGuidanceState {
    // Use a recent unix timestamp so the snapshot/activity are not considered stale.
    // snapshot_is_stale checks for "stale" or "unknown" status; fresh timestamps avoid both.
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let recent_ts = (now_secs - 3600).to_string(); // 1 hour ago
    ProjectGuidanceState {
        id: id.to_string(),
        status: status.to_string(),
        root_path: PathBuf::from(root),
        total_files: 10,
        accessed_files: 5,
        unused_files: 2,
        latest_snapshot_captured_at: Some(recent_ts.clone()),
        latest_activity_at: Some(recent_ts.clone()),
        latest_verification_at: Some(recent_ts),
    }
}

fn clean_repo_risk() -> Value {
    json!({
        "risk_level": "low",
        "operation_states": [],
        "conflicted_count": 0,
        "lockfile_anomalies": [],
        "large_diff": false,
        "changed_file_count": 0,
    })
}

fn make_verification_run(status: &str, kind: &str) -> VerificationRun {
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let recent_ts = (now_secs - 3600).to_string();
    VerificationRun {
        id: 1,
        kind: kind.to_string(),
        status: status.to_string(),
        command: "cargo test".to_string(),
        exit_code: Some(0),
        summary: None,
        source: "cli".to_string(),
        started_at: None,
        finished_at: recent_ts,
    }
}

// --- review_focus_for_action ---

#[test]
fn review_focus_for_action_inspect_hot_files_low_risk() {
    let repo_risk = clean_repo_risk();
    let result = review_focus_for_action("inspect_hot_files", &repo_risk);
    assert_eq!(result["candidate_family"], "hot_file");
    assert_eq!(
        result["candidate_basis"],
        json!(["highest_access_activity", "activity_present"])
    );
    assert!(result["candidate_risk_hints"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn review_focus_for_action_inspect_hot_files_elevated_risk() {
    let repo_risk = json!({
        "risk_level": "high",
        "operation_states": [],
        "conflicted_count": 0,
        "lockfile_anomalies": [],
        "large_diff": false,
        "changed_file_count": 0,
    });
    let result = review_focus_for_action("inspect_hot_files", &repo_risk);
    assert_eq!(result["candidate_family"], "hot_file");
    let hints = result["candidate_risk_hints"].as_array().unwrap();
    assert_eq!(hints, &vec![json!("repo_risk_elevated")]);
}

#[test]
fn review_focus_for_action_inspect_hot_files_large_diff() {
    // low risk but large diff
    let repo_risk = json!({
        "risk_level": "low",
        "operation_states": [],
        "conflicted_count": 0,
        "lockfile_anomalies": [],
        "large_diff": true,
        "changed_file_count": 50,
    });
    let result = review_focus_for_action("inspect_hot_files", &repo_risk);
    let hints = result["candidate_risk_hints"].as_array().unwrap();
    assert_eq!(hints, &vec![json!("repo_risk_elevated")]);
}

#[test]
fn review_focus_for_action_review_unused_files() {
    let repo_risk = clean_repo_risk();
    let result = review_focus_for_action("review_unused_files", &repo_risk);
    assert_eq!(result["candidate_family"], "unused_candidate");
    assert_eq!(
        result["candidate_basis"],
        json!(["zero_recorded_access", "snapshot_present"])
    );
    assert!(result["candidate_risk_hints"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn review_focus_for_action_unknown_returns_null() {
    let repo_risk = clean_repo_risk();
    let result = review_focus_for_action("take_snapshot", &repo_risk);
    assert!(result.is_null());
}

// --- recommend_project_action: start_monitor path ---

#[test]
fn recommend_project_action_start_monitor_when_not_monitoring() {
    let dir = tempfile::tempdir().unwrap();
    let state = make_state("proj-1", "stopped", dir.path().to_str().unwrap());
    let repo_risk = clean_repo_risk();
    let result = recommend_project_action(&state, &repo_risk, &[]);
    assert_eq!(result["recommended_next_action"], "start_monitor");
    assert_eq!(result["strategy_mode"], "collect_evidence_first");
    assert!(result["suggested_commands"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c.as_str().unwrap().contains("opendog start")));
}

// --- recommend_project_action: take_snapshot path ---

#[test]
fn recommend_project_action_take_snapshot_when_no_files() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = make_state("proj-2", "monitoring", dir.path().to_str().unwrap());
    state.total_files = 0;
    let repo_risk = clean_repo_risk();
    let result = recommend_project_action(&state, &repo_risk, &[]);
    assert_eq!(result["recommended_next_action"], "take_snapshot");
    assert!(result["reason"]
        .as_str()
        .unwrap()
        .contains("no snapshot data exists"));
}

// --- recommend_project_action: generate_activity_then_stats path ---

#[test]
fn recommend_project_action_generate_activity_when_no_accessed_files() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = make_state("proj-3", "monitoring", dir.path().to_str().unwrap());
    state.accessed_files = 0;
    let repo_risk = clean_repo_risk();
    let result = recommend_project_action(&state, &repo_risk, &[]);
    assert_eq!(
        result["recommended_next_action"],
        "generate_activity_then_stats"
    );
    assert!(result["reason"]
        .as_str()
        .unwrap()
        .contains("no file access activity"));
}

// --- recommend_project_action: review_unused_files or inspect_hot_files ---

#[test]
fn recommend_project_action_returns_valid_json_with_active_project() {
    let dir = tempfile::tempdir().unwrap();
    // Create a Cargo.toml so detect_project_commands finds something
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"test\"").unwrap();
    let state = make_state("proj-4", "monitoring", dir.path().to_str().unwrap());
    let repo_risk = clean_repo_risk();
    let runs = vec![make_verification_run("passed", "test")];
    let result = recommend_project_action(&state, &repo_risk, &runs);
    // Should be one of the review/inspect actions
    let action = result["recommended_next_action"].as_str().unwrap();
    assert!(
        ["inspect_hot_files", "review_unused_files"].contains(&action),
        "unexpected action: {}",
        action
    );
    assert!(result["recommended_flow"].is_array());
    assert!(result["reason"].is_string());
    assert!(result["confidence"].is_string());
    assert!(result["strategy_mode"].is_string());
    assert!(result["suggested_commands"].is_array());
    // execution_sequence is only non-null for certain action types;
    // inspect_hot_files and review_unused_files return Null from the
    // execution_sequence dispatch, so just verify the field exists.
    assert!(result.get("execution_sequence").is_some());
}

#[test]
fn recommend_project_action_includes_review_focus() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"test\"").unwrap();
    let state = make_state("proj-5", "monitoring", dir.path().to_str().unwrap());
    let repo_risk = clean_repo_risk();
    let runs = vec![make_verification_run("passed", "test")];
    let result = recommend_project_action(&state, &repo_risk, &runs);
    let action = result["recommended_next_action"].as_str().unwrap();
    if action == "inspect_hot_files" || action == "review_unused_files" {
        assert!(result["review_focus"].is_object());
    }
}

// --- recommend_project_action: forced verification failure ---

#[test]
fn recommend_project_action_forces_failing_verification_review() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"test\"").unwrap();
    let mut state = make_state("proj-6", "monitoring", dir.path().to_str().unwrap());
    // Make the project look healthy
    state.total_files = 100;
    state.accessed_files = 50;
    // But verification is failing
    let mut repo_risk = clean_repo_risk();
    repo_risk["risk_level"] = json!("critical");
    repo_risk["operation_states"] = json!(["merge"]);
    let runs = vec![VerificationRun {
        id: 1,
        kind: "test".to_string(),
        status: "failed".to_string(),
        command: "cargo test".to_string(),
        exit_code: Some(1),
        summary: None,
        source: "cli".to_string(),
        started_at: None,
        finished_at: {
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            (now_secs - 3600).to_string()
        },
    }];
    let result = recommend_project_action(&state, &repo_risk, &runs);
    // Should force review_failing_verification
    assert_eq!(
        result["recommended_next_action"],
        "review_failing_verification"
    );
}

// --- recommend_project_action: stabilize_repository_state ---

#[test]
fn recommend_project_action_forces_stabilize_repository() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"test\"").unwrap();
    let state = make_state("proj-7", "monitoring", dir.path().to_str().unwrap());
    let mut repo_risk = clean_repo_risk();
    repo_risk["risk_level"] = json!("critical");
    repo_risk["operation_states"] = json!(["rebase"]);
    // Must provide passing verification runs so that eligibility does not
    // force run_verification_before_high_risk_changes instead.
    let runs = vec![make_verification_run("passed", "test")];
    let result = recommend_project_action(&state, &repo_risk, &runs);
    assert_eq!(
        result["recommended_next_action"],
        "stabilize_repository_state"
    );
}

// --- project_overview ---

#[test]
fn project_overview_assembles_correctly() {
    let dir = tempfile::tempdir().unwrap();
    let state = make_state("ov-1", "monitoring", dir.path().to_str().unwrap());
    let repo_risk = clean_repo_risk();
    let recommendation = json!({
        "recommended_next_action": "inspect_hot_files",
        "recommended_flow": ["Inspect hot files."],
        "reason": "Active project.",
        "confidence": "medium",
    });
    let verification_layer = json!({
        "status": "available",
        "safe_for_cleanup": true,
        "safe_for_refactor": false,
        "cleanup_blockers": [],
        "refactor_blockers": ["No lint evidence."],
        "gate_assessment": {
            "cleanup": { "level": "allow" },
            "refactor": { "level": "blocked" },
        },
    });
    let mock_data_summary = json!({"mock_candidate_count": 0, "hardcoded_candidate_count": 0});
    let storage_maintenance = json!({
        "maintenance_candidate": false,
        "vacuum_candidate": false,
        "approx_reclaimable_bytes": 0,
        "approx_db_size_bytes": 2048,
    });

    let result = project_overview(
        &state,
        &repo_risk,
        &recommendation,
        &verification_layer,
        &mock_data_summary,
        &storage_maintenance,
    );

    assert_eq!(result["project_id"], "ov-1");
    assert_eq!(result["status"], "monitoring");
    assert_eq!(result["snapshot_available"], true);
    assert_eq!(result["activity_available"], true);
    assert_eq!(result["unused_files"], 2);
    assert!(result["repo_status_risk"].is_object());
    assert!(result["verification_evidence"].is_object());
    assert!(result["mock_data_summary"].is_object());
    assert!(result["storage_maintenance"].is_object());
    assert!(result["project_toolchain"].is_object());
    assert_eq!(result["recommended_next_action"], "inspect_hot_files");
    // Attention enrichment should add attention fields
    assert!(result["attention_score"].is_number());
}
