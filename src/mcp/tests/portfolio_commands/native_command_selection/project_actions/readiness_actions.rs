use super::*;

#[test]
fn recommend_project_action_prefers_review_unused_when_data_exists() {
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            id: "demo".to_string(),
            status: "monitoring".to_string(),
            root_path: std::path::PathBuf::from("/tmp/demo"),
            total_files: 20,
            accessed_files: 8,
            unused_files: 12,
            latest_snapshot_captured_at: Some(fresh_ts()),
            latest_activity_at: Some(fresh_ts()),
            latest_verification_at: Some(fresh_ts()),
        },
        &json!({"operation_states": [], "risk_level": "low", "is_dirty": false}),
        &[VerificationRun {
            id: 1,
            kind: "test".to_string(),
            status: "passed".to_string(),
            command: "cargo test".to_string(),
            exit_code: Some(0),
            summary: None,
            source: "cli".to_string(),
            started_at: None,
            finished_at: fresh_ts(),
        }],
    );

    assert_eq!(
        recommendation["recommended_next_action"],
        "review_unused_files"
    );
    assert!(recommendation["reason"]
        .as_str()
        .unwrap()
        .contains("unused"));
}

#[test]
fn recommend_project_action_uses_project_native_commands() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"demo\"\n",
    )
    .unwrap();

    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            id: "demo".to_string(),
            status: "monitoring".to_string(),
            root_path: dir.path().to_path_buf(),
            total_files: 10,
            accessed_files: 0,
            unused_files: 10,
            latest_snapshot_captured_at: Some(fresh_ts()),
            latest_activity_at: None,
            latest_verification_at: None,
        },
        &json!({"operation_states": [], "risk_level": "low", "is_dirty": false}),
        &[],
    );

    assert!(recommendation["suggested_commands"][0]
        .as_str()
        .unwrap()
        .contains("pytest"));
}
