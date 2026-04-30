use super::*;

#[test]
fn recommend_project_action_prefers_start_when_not_monitored() {
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            id: "demo".to_string(),
            status: "active".to_string(),
            root_path: std::path::PathBuf::from("/tmp/demo"),
            total_files: 0,
            accessed_files: 0,
            unused_files: 0,
            latest_snapshot_captured_at: None,
            latest_activity_at: None,
            latest_verification_at: None,
        },
        &json!({"operation_states": [], "risk_level": "low", "is_dirty": false}),
        &[],
    );

    assert_eq!(recommendation["recommended_next_action"], "start_monitor");
    assert!(recommendation["reason"]
        .as_str()
        .unwrap()
        .contains("not currently being monitored"));
    assert_eq!(
        recommendation["recommended_flow"][0],
        json!("Start monitoring because fresh activity evidence does not exist yet.")
    );
}

#[test]
fn recommend_project_action_prefers_generate_activity_when_monitored_but_idle() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\n",
    )
    .unwrap();

    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            id: "demo".to_string(),
            status: "monitoring".to_string(),
            root_path: dir.path().to_path_buf(),
            total_files: 12,
            accessed_files: 0,
            unused_files: 12,
            latest_snapshot_captured_at: Some(fresh_ts()),
            latest_activity_at: None,
            latest_verification_at: None,
        },
        &json!({"operation_states": [], "risk_level": "low", "is_dirty": false}),
        &[],
    );

    assert_eq!(
        recommendation["recommended_next_action"],
        "generate_activity_then_stats"
    );
    assert!(recommendation["suggested_commands"][0]
        .as_str()
        .unwrap()
        .contains("cargo test"));
}
