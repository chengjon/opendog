use super::*;

#[test]
fn recommend_project_action_exposes_repo_truth_gaps_for_mid_operation_repo() {
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

    assert_eq!(
        recommendation["recommended_next_action"],
        json!("stabilize_repository_state")
    );
    assert_eq!(
        recommendation["repo_truth_gaps"],
        json!(["repository_mid_operation"])
    );
    assert_eq!(
        recommendation["mandatory_shell_checks"],
        json!(["git status", "git diff"])
    );
}

#[test]
fn recommend_project_action_keeps_non_git_boundary_advisory() {
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

    assert_eq!(
        recommendation["recommended_next_action"],
        json!("review_unused_files")
    );
    assert_eq!(
        recommendation["repo_truth_gaps"],
        json!(["not_git_repository"])
    );
    assert_eq!(recommendation["mandatory_shell_checks"], json!([]));
}
