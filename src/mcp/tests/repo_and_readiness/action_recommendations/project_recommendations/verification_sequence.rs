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

fn monitoring_state(root: &std::path::Path) -> ProjectGuidanceState {
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

#[test]
fn recommend_project_action_emits_missing_verification_sequence_from_toolchain_commands() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(&monitoring_state(root.path()), &clean_repo_risk(), &[]);

    assert_eq!(
        recommendation["recommended_next_action"],
        "run_verification_before_high_risk_changes"
    );
    assert_eq!(
        recommendation["execution_sequence"],
        json!({
            "mode": "run_project_verification_then_resume",
            "current_phase": "verify",
            "resume_with": "refresh_guidance_after_verification",
            "verification_commands": ["cargo test"],
            "resume_conditions": [
                "required_verification_recorded",
                "verification_evidence_fresh"
            ]
        })
    );
}

#[test]
fn recommend_project_action_emits_failing_verification_sequence_before_repo_stabilization() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &monitoring_state(root.path()),
        &json!({
            "status": "available",
            "risk_level": "high",
            "is_dirty": true,
            "operation_states": ["rebase"],
            "conflicted_count": 1,
            "lockfile_anomalies": [],
            "large_diff": false
        }),
        &[VerificationRun {
            id: 1,
            kind: "test".to_string(),
            status: "failed".to_string(),
            command: "cargo test -p api".to_string(),
            exit_code: Some(101),
            summary: Some("test failure".to_string()),
            source: "cli".to_string(),
            started_at: None,
            finished_at: fresh_ts(),
        }],
    );

    assert_eq!(
        recommendation["recommended_next_action"],
        "review_failing_verification"
    );
    assert_eq!(
        recommendation["execution_sequence"],
        json!({
            "mode": "resolve_failing_verification_then_resume",
            "current_phase": "repair_and_verify",
            "resume_with": "refresh_guidance_after_verification",
            "verification_commands": ["cargo test -p api"],
            "resume_conditions": [
                "no_failing_verification_runs",
                "verification_evidence_fresh"
            ]
        })
    );
}

#[test]
fn recommend_project_action_allows_empty_verification_command_lists() {
    let root = TempDir::new().unwrap();
    let recommendation = recommend_project_action(&monitoring_state(root.path()), &clean_repo_risk(), &[]);

    assert_eq!(
        recommendation["recommended_next_action"],
        "run_verification_before_high_risk_changes"
    );
    assert_eq!(
        recommendation["execution_sequence"]["verification_commands"],
        json!([])
    );
}

#[test]
fn recommend_project_action_keeps_null_sequence_for_non_sequenced_review_actions() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &monitoring_state(root.path()),
        &clean_repo_risk(),
        &[
            VerificationRun {
                id: 1,
                kind: "test".to_string(),
                status: "passed".to_string(),
                command: "cargo test".to_string(),
                exit_code: Some(0),
                summary: Some("ok".to_string()),
                source: "cli".to_string(),
                started_at: None,
                finished_at: fresh_ts(),
            },
            VerificationRun {
                id: 2,
                kind: "lint".to_string(),
                status: "passed".to_string(),
                command: "cargo clippy --all-targets --all-features -- -D warnings".to_string(),
                exit_code: Some(0),
                summary: Some("ok".to_string()),
                source: "cli".to_string(),
                started_at: None,
                finished_at: fresh_ts(),
            },
            VerificationRun {
                id: 3,
                kind: "build".to_string(),
                status: "passed".to_string(),
                command: "cargo check".to_string(),
                exit_code: Some(0),
                summary: Some("ok".to_string()),
                source: "cli".to_string(),
                started_at: None,
                finished_at: fresh_ts(),
            },
        ],
    );

    assert_eq!(recommendation["execution_sequence"], Value::Null);
}
