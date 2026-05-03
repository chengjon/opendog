use super::*;
use crate::mcp::project_recommendation::eligibility::{GateLevel, RecommendationSignals};
use crate::mcp::project_recommendation::review_focus_for_action;
use serde_json::json;
use std::fs;
use tempfile::TempDir;

fn rust_project_root() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    dir
}

fn clean_repo_risk() -> serde_json::Value {
    json!({
        "status": "available",
        "risk_level": "low",
        "is_dirty": false,
        "operation_states": [],
        "conflicted_count": 0,
        "lockfile_anomalies": [],
        "large_diff": false
    })
}

fn base_state(root: &std::path::Path) -> ProjectGuidanceState {
    ProjectGuidanceState {
        id: "demo".to_string(),
        status: "monitoring".to_string(),
        root_path: root.to_path_buf(),
        total_files: 20,
        accessed_files: 8,
        unused_files: 4,
        latest_snapshot_captured_at: Some(fresh_ts()),
        latest_activity_at: Some(fresh_ts()),
        latest_verification_at: Some(fresh_ts()),
    }
}

fn passing_runs() -> Vec<VerificationRun> {
    vec![VerificationRun {
        id: 1,
        kind: "test".to_string(),
        status: "passed".to_string(),
        command: "cargo test".to_string(),
        exit_code: Some(0),
        summary: Some("ok".to_string()),
        source: "cli".to_string(),
        started_at: None,
        finished_at: fresh_ts(),
    }]
}

#[test]
fn recommend_project_action_emits_hot_file_review_focus() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            unused_files: 0,
            ..base_state(root.path())
        },
        &json!({
            "status": "available",
            "risk_level": "high",
            "is_dirty": true,
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": true
        }),
        &passing_runs(),
    );

    assert_eq!(
        recommendation["recommended_next_action"],
        "inspect_hot_files"
    );
    assert_eq!(
        recommendation["review_focus"],
        json!({
            "candidate_family": "hot_file",
            "candidate_basis": ["highest_access_activity", "activity_present"],
            "candidate_risk_hints": ["repo_risk_elevated"]
        })
    );
}

#[test]
fn recommend_project_action_emits_unused_review_focus_for_stale_snapshot() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &base_state(root.path()),
        &clean_repo_risk(),
        &passing_runs(),
    );

    assert_eq!(
        recommendation["recommended_next_action"],
        "review_unused_files"
    );
    assert_eq!(
        recommendation["review_focus"],
        json!({
            "candidate_family": "unused_candidate",
            "candidate_basis": ["zero_recorded_access", "snapshot_present"],
            "candidate_risk_hints": []
        })
    );

    let review_focus = review_focus_for_action(
        "review_unused_files",
        &RecommendationSignals {
            cleanup_gate_level: GateLevel::Allow,
            refactor_gate_level: GateLevel::Allow,
            monitoring_active: true,
            snapshot_available: true,
            activity_available: true,
            snapshot_stale: true,
            activity_stale: false,
            verification_missing: false,
            verification_stale: false,
            verification_failing: false,
            unused_files: 4,
        },
        &clean_repo_risk(),
    );

    assert_eq!(
        review_focus,
        json!({
            "candidate_family": "unused_candidate",
            "candidate_basis": ["zero_recorded_access", "snapshot_present"],
            "candidate_risk_hints": ["snapshot_evidence_stale"]
        })
    );
}

#[test]
fn recommend_project_action_keeps_review_focus_null_for_non_review_actions() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            status: "stopped".to_string(),
            total_files: 0,
            accessed_files: 0,
            unused_files: 0,
            latest_snapshot_captured_at: None,
            latest_activity_at: None,
            latest_verification_at: None,
            ..base_state(root.path())
        },
        &clean_repo_risk(),
        &[],
    );

    assert_eq!(recommendation["recommended_next_action"], "start_monitor");
    assert!(recommendation["review_focus"].is_null());
}
