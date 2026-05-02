use super::*;
use serde_json::Value;
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
        unused_files: 6,
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
fn recommend_project_action_emits_start_monitor_sequence() {
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
    assert_eq!(
        recommendation["execution_sequence"],
        json!({
            "mode": "start_monitor_then_resume",
            "current_phase": "enable_monitoring",
            "resume_with": "refresh_guidance_after_observation",
            "observation_steps": ["start_monitor", "generate_real_project_activity"],
            "resume_conditions": ["monitoring_active", "activity_evidence_recorded"]
        })
    );
}

#[test]
fn recommend_project_action_emits_snapshot_refresh_sequence() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            total_files: 0,
            accessed_files: 0,
            unused_files: 0,
            latest_snapshot_captured_at: None,
            latest_activity_at: None,
            ..base_state(root.path())
        },
        &clean_repo_risk(),
        &passing_runs(),
    );

    assert_eq!(recommendation["recommended_next_action"], "take_snapshot");
    assert_eq!(
        recommendation["execution_sequence"],
        json!({
            "mode": "refresh_snapshot_then_resume",
            "current_phase": "snapshot",
            "resume_with": "refresh_guidance_after_snapshot",
            "observation_steps": ["take_snapshot"],
            "resume_conditions": ["snapshot_available", "snapshot_evidence_fresh"]
        })
    );
}

#[test]
fn recommend_project_action_emits_activity_generation_sequence() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            accessed_files: 0,
            latest_activity_at: None,
            ..base_state(root.path())
        },
        &clean_repo_risk(),
        &passing_runs(),
    );

    assert_eq!(
        recommendation["recommended_next_action"],
        "generate_activity_then_stats"
    );
    assert_eq!(
        recommendation["execution_sequence"],
        json!({
            "mode": "generate_activity_then_resume",
            "current_phase": "generate_activity",
            "resume_with": "refresh_guidance_after_activity",
            "observation_steps": ["generate_real_project_activity", "refresh_stats"],
            "resume_conditions": ["activity_evidence_recorded", "activity_evidence_fresh"]
        })
    );
}

#[test]
fn recommend_project_action_keeps_repo_stabilization_ahead_of_observation_sequence() {
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
        &json!({
            "status": "available",
            "risk_level": "high",
            "is_dirty": true,
            "operation_states": ["rebase"],
            "conflicted_count": 1,
            "lockfile_anomalies": [],
            "large_diff": false
        }),
        &passing_runs(),
    );

    assert_eq!(
        recommendation["recommended_next_action"],
        "stabilize_repository_state"
    );
    assert_eq!(
        recommendation["execution_sequence"]["mode"],
        "shell_stabilize_then_resume"
    );
}

#[test]
fn recommend_project_action_keeps_null_sequence_for_non_sequenced_review_actions() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &base_state(root.path()),
        &clean_repo_risk(),
        &passing_runs(),
    );

    assert_eq!(recommendation["execution_sequence"], Value::Null);
}
