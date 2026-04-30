use super::*;

#[test]
fn recommend_project_action_reason_mentions_why_hotspot_review_lost() {
    let repo_risk = json!({
        "operation_states": [],
        "risk_level": "high",
        "is_dirty": true,
        "large_diff": true,
        "changed_file_count": 18,
        "conflicted_count": 0,
        "lockfile_anomalies": []
    });
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
        &repo_risk,
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
                command: "cargo clippy".to_string(),
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

    assert_eq!(
        recommendation["recommended_next_action"],
        json!("review_unused_files")
    );
    assert!(recommendation["reason"]
        .as_str()
        .unwrap()
        .contains("hotspot review"));
    assert!(recommendation["reason"]
        .as_str()
        .unwrap()
        .contains("repository state"));
}
