use super::*;
use serde_json::Value;

#[test]
fn recommend_project_action_emits_repo_stabilization_sequence() {
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            id: "demo".to_string(),
            status: "monitoring".to_string(),
            root_path: std::path::PathBuf::from("/tmp/demo"),
            total_files: 20,
            accessed_files: 8,
            unused_files: 4,
            latest_snapshot_captured_at: Some(fresh_ts()),
            latest_activity_at: Some(fresh_ts()),
            latest_verification_at: Some(fresh_ts()),
        },
        &json!({
            "status": "available",
            "risk_level": "high",
            "is_dirty": true,
            "operation_states": ["rebase"],
            "conflicted_count": 2,
            "lockfile_anomalies": [],
            "large_diff": false
        }),
        &[VerificationRun {
            id: 1,
            kind: "test".to_string(),
            status: "passed".to_string(),
            command: "cargo test".to_string(),
            exit_code: Some(0),
            summary: Some("ok".to_string()),
            source: "cli".to_string(),
            started_at: None,
            finished_at: fresh_ts(),
        }],
    );

    assert_eq!(
        recommendation["execution_sequence"],
        json!({
            "mode": "shell_stabilize_then_resume",
            "current_phase": "stabilize",
            "resume_with": "refresh_guidance_after_repo_stable",
            "stability_checks": ["git status", "git diff"],
            "resume_conditions": [
                "operation_states_cleared",
                "conflicted_count_zero"
            ]
        })
    );
}

#[test]
fn recommend_project_action_keeps_null_sequence_for_non_stabilization_actions() {
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            id: "demo".to_string(),
            status: "monitoring".to_string(),
            root_path: std::path::PathBuf::from("/tmp/demo"),
            total_files: 20,
            accessed_files: 8,
            unused_files: 6,
            latest_snapshot_captured_at: Some(fresh_ts()),
            latest_activity_at: Some(fresh_ts()),
            latest_verification_at: Some(fresh_ts()),
        },
        &json!({
            "status": "not_git_repository",
            "risk_level": "low",
            "is_dirty": false,
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": false
        }),
        &[VerificationRun {
            id: 1,
            kind: "test".to_string(),
            status: "passed".to_string(),
            command: "cargo test".to_string(),
            exit_code: Some(0),
            summary: Some("ok".to_string()),
            source: "cli".to_string(),
            started_at: None,
            finished_at: fresh_ts(),
        }],
    );

    assert_eq!(recommendation.get("execution_sequence"), Some(&Value::Null));
}
