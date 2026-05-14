use super::*;
use crate::core::file_classification::FilePathClassificationFilter;

fn stats_entry(path: &str, access_count: i64, modification_count: i64) -> StatsEntry {
    StatsEntry {
        file_path: path.to_string(),
        size: 42,
        file_type: path.rsplit('.').next().unwrap_or("").to_string(),
        access_count,
        estimated_duration_ms: 1000,
        modification_count,
        last_access_time: Some("1".to_string()),
        first_seen_time: None,
    }
}

#[test]
fn export_project_evidence_payload_has_versioned_contract() {
    let summary = ProjectSummary {
        total_files: 2,
        accessed_files: 1,
        unused_files: 1,
    };
    let artifact = crate::core::export::PortableProjectExport {
        schema_version: "portable".to_string(),
        project_id: "demo".to_string(),
        format: "json".to_string(),
        view: "stats".to_string(),
        generated_at: "1".to_string(),
        summary: summary.clone(),
        row_count: 2,
        rows: vec![],
    };
    let value = export_project_evidence_payload(
        MCP_EXPORT_PROJECT_EVIDENCE_V1,
        &artifact,
        "/tmp/demo.json",
        128,
        "{\"project_id\":\"demo\"}",
    );
    assert_eq!(value["schema_version"], MCP_EXPORT_PROJECT_EVIDENCE_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["format"], "json");
    assert_eq!(value["view"], "stats");
    assert_eq!(value["bytes_written"], 128);
    assert_eq!(value["row_count"], 2);
    assert_eq!(value["summary"]["total_files"], 2);
}

#[test]
fn stats_payload_has_versioned_contract() {
    let summary = ProjectSummary {
        total_files: 5,
        accessed_files: 2,
        unused_files: 3,
    };
    let entries = vec![StatsEntry {
        file_path: "src/main.rs".to_string(),
        size: 42,
        file_type: "rs".to_string(),
        access_count: 7,
        estimated_duration_ms: 12,
        modification_count: 1,
        last_access_time: Some("1".to_string()),
        first_seen_time: None,
    }];
    let value = stats_payload(
        "demo",
        &summary,
        &entries,
        std::path::Path::new("/tmp/demo"),
        &[],
    );
    assert_eq!(value["schema_version"], MCP_STATS_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["summary"]["total_files"], 5);
    assert_eq!(value["files"][0]["path"], "src/main.rs");
    assert_eq!(value["result_window"]["total_count"], 1);
    assert_eq!(value["result_window"]["returned_count"], 1);
    assert_eq!(value["result_window"]["truncated"], false);
}

#[test]
fn unused_files_payload_has_versioned_contract() {
    let unused = vec![StatsEntry {
        file_path: "old/module.rs".to_string(),
        size: 11,
        file_type: "rs".to_string(),
        access_count: 0,
        estimated_duration_ms: 0,
        modification_count: 0,
        last_access_time: None,
        first_seen_time: None,
    }];
    let value = unused_files_payload("demo", &unused, std::path::Path::new("/tmp/demo"), &[]);
    assert_eq!(value["schema_version"], MCP_UNUSED_FILES_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["unused_count"], 1);
    assert_eq!(value["files"][0]["path"], "old/module.rs");
    assert_eq!(value["result_window"]["total_count"], 1);
    assert_eq!(value["result_window"]["returned_count"], 1);
    assert_eq!(value["result_window"]["truncated"], false);
}

#[test]
fn stats_payload_is_bounded_by_default_for_large_result_sets() {
    let summary = ProjectSummary {
        total_files: 55,
        accessed_files: 55,
        unused_files: 0,
    };
    let entries: Vec<StatsEntry> = (0..55)
        .map(|idx| StatsEntry {
            file_path: format!("src/file_{idx}.rs"),
            size: 42,
            file_type: "rs".to_string(),
            access_count: 55 - idx,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: Some("1".to_string()),
            first_seen_time: None,
        })
        .collect();

    let value = stats_payload(
        "demo",
        &summary,
        &entries,
        std::path::Path::new("/tmp/demo"),
        &[],
    );

    assert_eq!(value["files"].as_array().unwrap().len(), 50);
    assert_eq!(value["result_window"]["total_count"], 55);
    assert_eq!(value["result_window"]["returned_count"], 50);
    assert_eq!(value["result_window"]["limit"], 50);
    assert_eq!(value["result_window"]["truncated"], true);
}

#[test]
fn unused_files_payload_is_bounded_by_default_for_large_result_sets() {
    let unused: Vec<StatsEntry> = (0..55)
        .map(|idx| StatsEntry {
            file_path: format!("old/file_{idx}.rs"),
            size: 11,
            file_type: "rs".to_string(),
            access_count: 0,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        })
        .collect();

    let value = unused_files_payload("demo", &unused, std::path::Path::new("/tmp/demo"), &[]);

    assert_eq!(value["unused_count"], 55);
    assert_eq!(value["files"].as_array().unwrap().len(), 50);
    assert_eq!(value["result_window"]["total_count"], 55);
    assert_eq!(value["result_window"]["returned_count"], 50);
    assert_eq!(value["result_window"]["limit"], 50);
    assert_eq!(value["result_window"]["truncated"], true);
}

#[test]
fn stats_payload_honors_explicit_limit() {
    let summary = ProjectSummary {
        total_files: 5,
        accessed_files: 5,
        unused_files: 0,
    };
    let entries: Vec<StatsEntry> = (0..5)
        .map(|idx| StatsEntry {
            file_path: format!("src/file_{idx}.rs"),
            size: 42,
            file_type: "rs".to_string(),
            access_count: 5 - idx,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: Some("1".to_string()),
            first_seen_time: None,
        })
        .collect();

    let value = stats_payload_with_limit(
        "demo",
        &summary,
        &entries,
        std::path::Path::new("/tmp/demo"),
        &[],
        3,
        FilePathClassificationFilter::All,
    );

    assert_eq!(value["files"].as_array().unwrap().len(), 3);
    assert_eq!(value["result_window"]["total_count"], 5);
    assert_eq!(value["result_window"]["returned_count"], 3);
    assert_eq!(value["result_window"]["limit"], 3);
    assert_eq!(value["result_window"]["truncated"], true);
}

#[test]
fn unused_files_payload_honors_explicit_limit() {
    let unused: Vec<StatsEntry> = (0..5)
        .map(|idx| StatsEntry {
            file_path: format!("old/file_{idx}.rs"),
            size: 11,
            file_type: "rs".to_string(),
            access_count: 0,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        })
        .collect();

    let value = unused_files_payload_with_limit(
        "demo",
        &unused,
        std::path::Path::new("/tmp/demo"),
        &[],
        2,
        FilePathClassificationFilter::All,
    );

    assert_eq!(value["unused_count"], 5);
    assert_eq!(value["files"].as_array().unwrap().len(), 2);
    assert_eq!(value["result_window"]["total_count"], 5);
    assert_eq!(value["result_window"]["returned_count"], 2);
    assert_eq!(value["result_window"]["limit"], 2);
    assert_eq!(value["result_window"]["truncated"], true);
}

#[test]
fn stats_payload_classifies_infrastructure_and_source_files() {
    let summary = ProjectSummary {
        total_files: 8,
        accessed_files: 8,
        unused_files: 0,
    };
    let paths = [
        ".claude/settings.local.json",
        ".amazonq/rules.md",
        ".cursor/rules/project.mdc",
        ".agents/prompts/review.md",
        ".zread/wiki/current/index.md",
        "src/main.py",
        "web/frontend/src/App.vue",
        "src/main.py.bak",
    ];
    let entries: Vec<StatsEntry> = paths
        .iter()
        .enumerate()
        .map(|(idx, path)| StatsEntry {
            file_path: (*path).to_string(),
            size: 42,
            file_type: path.rsplit('.').next().unwrap_or("").to_string(),
            access_count: 8 - idx as i64,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: Some("1".to_string()),
            first_seen_time: None,
        })
        .collect();

    let value = stats_payload(
        "demo",
        &summary,
        &entries,
        std::path::Path::new("/tmp/demo"),
        &[],
    );

    assert_eq!(value["files"][0]["path_classification"], "infrastructure");
    assert_eq!(value["files"][5]["path_classification"], "source");
    assert_eq!(value["files"][6]["path_classification"], "source");
    assert_eq!(value["files"][7]["path_classification"], "backup");
    assert_eq!(value["classification_summary"]["infrastructure_files"], 5);
    assert_eq!(value["classification_summary"]["source_files"], 2);
    assert_eq!(value["classification_summary"]["backup_files"], 1);
}

#[test]
fn stats_payload_filters_rows_by_path_classification() {
    let summary = ProjectSummary {
        total_files: 4,
        accessed_files: 3,
        unused_files: 1,
    };
    let entries = vec![
        stats_entry("src/main.rs", 7, 1),
        stats_entry(".claude/settings.json", 99, 0),
        stats_entry("notes.txt", 1, 0),
        stats_entry("src/main.rs.bak", 0, 0),
    ];

    let source_value = stats_payload_with_limit(
        "demo",
        &summary,
        &entries,
        std::path::Path::new("/tmp/demo"),
        &[],
        50,
        FilePathClassificationFilter::Source,
    );

    assert_eq!(source_value["files"].as_array().unwrap().len(), 1);
    assert_eq!(source_value["files"][0]["path"], "src/main.rs");
    assert_eq!(source_value["result_window"]["total_count"], 1);
    assert_eq!(source_value["result_window"]["returned_count"], 1);
    assert_eq!(
        source_value["result_window"]["path_classification"],
        "source"
    );
    assert_eq!(source_value["classification_summary"]["source_files"], 1);
    assert_eq!(
        source_value["classification_summary"]["infrastructure_files"],
        1
    );
    assert_eq!(source_value["classification_summary"]["backup_files"], 1);
    assert_eq!(source_value["classification_summary"]["project_files"], 1);

    let infrastructure_value = stats_payload_with_limit(
        "demo",
        &summary,
        &entries,
        std::path::Path::new("/tmp/demo"),
        &[],
        50,
        FilePathClassificationFilter::Infrastructure,
    );

    assert_eq!(infrastructure_value["files"].as_array().unwrap().len(), 1);
    assert_eq!(
        infrastructure_value["files"][0]["path"],
        ".claude/settings.json"
    );
    assert_eq!(infrastructure_value["result_window"]["total_count"], 1);
}

#[test]
fn stats_payload_returns_clear_empty_window_for_filter_without_matches() {
    let summary = ProjectSummary {
        total_files: 1,
        accessed_files: 1,
        unused_files: 0,
    };
    let entries = vec![stats_entry("README.md", 1, 0)];

    let value = stats_payload_with_limit(
        "demo",
        &summary,
        &entries,
        std::path::Path::new("/tmp/demo"),
        &[],
        50,
        FilePathClassificationFilter::Source,
    );

    assert_eq!(value["files"].as_array().unwrap().len(), 0);
    assert_eq!(value["result_window"]["total_count"], 0);
    assert_eq!(value["result_window"]["returned_count"], 0);
    assert_eq!(value["result_window"]["truncated"], false);
    assert_eq!(value["result_window"]["path_classification"], "source");
}

#[test]
fn unused_payload_preserves_total_count_and_adds_filtered_count() {
    let unused = vec![
        stats_entry("src/old.rs", 0, 0),
        stats_entry(".claude/settings.json", 0, 0),
        stats_entry("notes.txt", 0, 0),
    ];

    let value = unused_files_payload_with_limit(
        "demo",
        &unused,
        std::path::Path::new("/tmp/demo"),
        &[],
        50,
        FilePathClassificationFilter::Source,
    );

    assert_eq!(value["unused_count"], 3);
    assert_eq!(value["filtered_unused_count"], 1);
    assert_eq!(value["files"].as_array().unwrap().len(), 1);
    assert_eq!(value["files"][0]["path"], "src/old.rs");
    assert_eq!(value["result_window"]["total_count"], 1);
    assert_eq!(value["result_window"]["path_classification"], "source");

    let all_value = unused_files_payload_with_limit(
        "demo",
        &unused,
        std::path::Path::new("/tmp/demo"),
        &[],
        50,
        FilePathClassificationFilter::All,
    );
    assert!(all_value.get("filtered_unused_count").is_none());
}

#[test]
fn unused_payload_prefers_source_candidates_over_infrastructure_noise() {
    let paths = [
        ".claude/agents/review.md",
        ".amazonq/config.json",
        ".cursor/rules/project.mdc",
        ".agents/prompts/review.md",
        ".zread/wiki/current/index.md",
        "src/live_service.py",
        "web/frontend/src/App.vue",
        "notes/tmp.txt~",
    ];
    let unused: Vec<StatsEntry> = paths
        .iter()
        .map(|path| StatsEntry {
            file_path: (*path).to_string(),
            size: 11,
            file_type: path.rsplit('.').next().unwrap_or("").to_string(),
            access_count: 0,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        })
        .collect();

    let value = unused_files_payload("demo", &unused, std::path::Path::new("/tmp/demo"), &[]);

    assert_eq!(value["files"][0]["path_classification"], "infrastructure");
    assert_eq!(value["files"][5]["path_classification"], "source");
    assert_eq!(value["files"][7]["path_classification"], "backup");
    assert_eq!(value["classification_summary"]["infrastructure_files"], 5);
    assert_eq!(value["classification_summary"]["source_files"], 2);
    assert_eq!(value["classification_summary"]["backup_files"], 1);
    assert_eq!(
        value["guidance"]["file_recommendations"][0]["file_path"],
        "src/live_service.py"
    );
    assert_eq!(
        value["guidance"]["layers"]["workspace_observation"]["infrastructure_candidates"],
        5
    );
}

#[test]
fn cleanup_project_data_payload_has_versioned_contract() {
    let value = cleanup_project_data_payload(
        MCP_CLEANUP_PROJECT_DATA_V1,
        "demo",
        &ProjectDataCleanupResult {
            scope: "activity".to_string(),
            dry_run: true,
            older_than_days: Some(14),
            keep_snapshot_runs: Some(2),
            vacuum: false,
            deleted: CleanupCountBreakdown {
                file_sightings: 4,
                file_events: 2,
                verification_runs: 0,
                snapshot_runs: 0,
                snapshot_history: 0,
            },
            storage_before: StorageMetrics {
                page_size: 4096,
                page_count: 10,
                freelist_count: 1,
                approx_db_size_bytes: 40_960,
                approx_reclaimable_bytes: 4_096,
            },
            storage_after: None,
            maintenance: CleanupMaintenanceStatus {
                optimized: false,
                vacuumed: false,
            },
            notes: vec!["dry run".to_string()],
        },
    );

    assert_eq!(value["schema_version"], MCP_CLEANUP_PROJECT_DATA_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["scope"], "activity");
    assert_eq!(value["deleted"]["file_sightings"], 4);
    assert_eq!(value["guidance"]["schema_version"], MCP_GUIDANCE_V1);
}
