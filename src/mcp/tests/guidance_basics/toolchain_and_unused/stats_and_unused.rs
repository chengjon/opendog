use super::*;

fn stats_entry(path: &str, access_count: i64, modification_count: i64) -> StatsEntry {
    StatsEntry {
        file_path: path.to_string(),
        size: 10,
        file_type: "rs".to_string(),
        access_count,
        estimated_duration_ms: 1000,
        modification_count,
        last_access_time: None,
        first_seen_time: None,
    }
}

fn summary_with_activity() -> ProjectSummary {
    ProjectSummary {
        total_files: 4,
        accessed_files: 2,
        unused_files: 2,
    }
}

fn passing_verification_runs() -> Vec<VerificationRun> {
    vec![VerificationRun {
        id: 1,
        kind: "test".to_string(),
        status: "passed".to_string(),
        command: "cargo test".to_string(),
        exit_code: Some(0),
        summary: None,
        source: "cli".to_string(),
        started_at: None,
        finished_at: fresh_ts(),
    }]
}

#[test]
fn stats_guidance_marks_hot_file_candidate_as_primary_with_basis() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/main.rs"),
        "const MOCK_USER: &str = \"demo data\";\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("src/old.rs"), "pub fn old() {}\n").unwrap();

    let value = stats_guidance(
        dir.path(),
        &summary_with_activity(),
        &[
            stats_entry("src/main.rs", 12, 3),
            stats_entry("src/old.rs", 0, 0),
        ],
        &passing_verification_runs(),
    );

    assert_eq!(
        value["file_recommendations"][0]["candidate_priority"],
        "primary"
    );
    assert_eq!(
        value["file_recommendations"][0]["candidate_basis"],
        json!([
            "highest_access_activity",
            "activity_present",
            "mock_data_overlap"
        ])
    );
    assert_eq!(
        value["file_recommendations"][0]["candidate_risk_hints"],
        json!([])
    );
}

#[test]
fn stats_guidance_marks_companion_unused_candidate_as_secondary_with_basis() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(
        dir.path().join("src/old.rs"),
        "const CUSTOMER_EMAIL: &str = \"demo@example.com\";\nconst CUSTOMER_STREET: &str = \"1 Demo Street\";\n",
    )
    .unwrap();

    let value = stats_guidance(
        dir.path(),
        &summary_with_activity(),
        &[
            stats_entry("src/main.rs", 12, 3),
            stats_entry("src/old.rs", 0, 0),
        ],
        &passing_verification_runs(),
    );

    assert_eq!(
        value["file_recommendations"][1]["candidate_priority"],
        "secondary"
    );
    assert_eq!(
        value["file_recommendations"][1]["candidate_basis"],
        json!([
            "zero_recorded_access",
            "snapshot_present",
            "hardcoded_data_overlap"
        ])
    );
    assert_eq!(
        value["file_recommendations"][1]["candidate_risk_hints"],
        json!([])
    );
}

#[test]
fn stats_guidance_surfaces_refactor_gate_level_in_execution_strategy() {
    let dir = TempDir::new().unwrap();
    let summary = ProjectSummary {
        total_files: 2,
        accessed_files: 1,
        unused_files: 1,
    };
    let entries = vec![
        StatsEntry {
            file_path: "src/main.rs".to_string(),
            size: 10,
            file_type: "rs".to_string(),
            access_count: 5,
            estimated_duration_ms: 1000,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        },
        StatsEntry {
            file_path: "src/old.rs".to_string(),
            size: 10,
            file_type: "rs".to_string(),
            access_count: 0,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        },
    ];

    let value = stats_guidance(
        dir.path(),
        &summary,
        &entries,
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
        value["layers"]["verification_evidence"]["gate_assessment"]["cleanup"]["level"],
        json!("caution")
    );
    assert_eq!(
        value["layers"]["execution_strategy"]["cleanup_gate_level"],
        json!("caution")
    );
    assert_eq!(
        value["layers"]["execution_strategy"]["refactor_gate_level"],
        json!("blocked")
    );
}

#[test]
fn unused_guidance_surfaces_candidate_files() {
    let dir = TempDir::new().unwrap();
    let entries = vec![
        StatsEntry {
            file_path: "src/old.rs".to_string(),
            size: 10,
            file_type: "rs".to_string(),
            access_count: 0,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        },
        StatsEntry {
            file_path: "src/older.rs".to_string(),
            size: 10,
            file_type: "rs".to_string(),
            access_count: 0,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        },
    ];

    let value = unused_guidance(dir.path(), &entries, &[]);
    assert_eq!(
        value["layers"]["workspace_observation"]["unused_candidates"],
        json!(2)
    );
    assert_eq!(value["file_recommendations"][0]["file_path"], "src/old.rs");
    assert!(value["summary"]
        .as_str()
        .unwrap()
        .contains("Verify with shell search or tests"));
}

#[test]
fn unused_guidance_reuses_shared_cleanup_reason() {
    let dir = TempDir::new().unwrap();
    let entries = vec![StatsEntry {
        file_path: "src/old.rs".to_string(),
        size: 10,
        file_type: "rs".to_string(),
        access_count: 0,
        estimated_duration_ms: 0,
        modification_count: 0,
        last_access_time: None,
        first_seen_time: None,
    }];

    let value = unused_guidance(
        dir.path(),
        &entries,
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
        value["layers"]["cleanup_refactor_candidates"]["safe_for_cleanup"],
        json!(true)
    );
    assert_eq!(
        value["layers"]["cleanup_refactor_candidates"]["cleanup_blockers"],
        json!([])
    );
}
