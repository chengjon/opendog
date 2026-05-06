use super::*;

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
