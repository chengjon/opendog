use super::*;

#[test]
fn stats_guidance_adapts_when_no_activity_exists() {
    let summary = ProjectSummary {
        total_files: 5,
        accessed_files: 0,
        unused_files: 5,
    };
    let value = stats_guidance(std::path::Path::new("/tmp/demo"), &summary, &[], &[]);

    assert!(value["summary"]
        .as_str()
        .unwrap()
        .contains("no file activity has been recorded"));
    assert_eq!(value["next_tools"][0], "start_monitor");
}

#[test]
fn stats_guidance_mentions_hottest_file_when_available() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join("target")).unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    let summary = ProjectSummary {
        total_files: 3,
        accessed_files: 2,
        unused_files: 1,
    };
    let entries = vec![StatsEntry {
        file_path: "src/main.rs".to_string(),
        size: 10,
        file_type: "rs".to_string(),
        access_count: 5,
        estimated_duration_ms: 1000,
        modification_count: 0,
        last_access_time: None,
        first_seen_time: None,
    }];
    let value = stats_guidance(dir.path(), &summary, &entries, &[]);

    assert!(value["summary"].as_str().unwrap().contains("src/main.rs"));
    assert_eq!(value["file_recommendations"][0]["file_path"], "src/main.rs");
    assert_eq!(
        value["layers"]["workspace_observation"]["analysis_state"],
        json!("ready")
    );
    assert_eq!(
        value["layers"]["project_toolchain"]["project_type"],
        json!("unknown")
    );
    assert!(value["layers"]["repo_status_risk"]["status"].is_string());
    assert!(
        value["layers"]["constraints_boundaries"]["generated_artifact_directories"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item == "target")
    );
    assert!(value["layers"]["constraints_boundaries"]["protected_paths"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == ".git"));
}

#[test]
fn stats_guidance_marks_observation_ready_when_summary_has_activity_but_entries_are_empty() {
    let summary = ProjectSummary {
        total_files: 3,
        accessed_files: 2,
        unused_files: 1,
    };
    let value = stats_guidance(std::path::Path::new("/tmp/demo"), &summary, &[], &[]);

    assert!(value["summary"]
        .as_str()
        .unwrap()
        .contains("Query unused files next"));
    assert_eq!(
        value["layers"]["workspace_observation"]["analysis_state"],
        json!("ready")
    );
    assert_eq!(
        value["layers"]["workspace_observation"]["activity_available"],
        json!(true)
    );
}
