use super::*;

#[test]
fn collect_storage_metrics_returns_valid_page_info() {
    let db = test_db();
    let metrics = collect_storage_metrics(&db).unwrap();
    assert!(metrics.page_size > 0);
    assert!(metrics.page_count >= 1);
    assert!(metrics.approx_db_size_bytes >= metrics.page_size);
    // Fresh DB should have minimal reclaimable space
    assert!(metrics.approx_reclaimable_bytes >= 0);
}

#[test]
fn collect_storage_metrics_size_equals_page_size_times_page_count() {
    let db = test_db();
    let metrics = collect_storage_metrics(&db).unwrap();
    assert_eq!(
        metrics.approx_db_size_bytes,
        metrics.page_size.saturating_mul(metrics.page_count)
    );
}

#[test]
fn collect_storage_metrics_reclaimable_equals_page_size_times_freelist() {
    let db = test_db();
    let metrics = collect_storage_metrics(&db).unwrap();
    assert_eq!(
        metrics.approx_reclaimable_bytes,
        metrics.page_size.saturating_mul(metrics.freelist_count)
    );
}

#[test]
fn collect_storage_evidence_counts_returns_table_counts() {
    let db = test_db();
    db.execute(
        "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES ('a.rs', 'codex', 1, '100')",
        rusqlite::params![],
    )
    .unwrap();
    db.execute(
        "INSERT INTO file_events (file_path, event_type, event_time) VALUES ('a.rs', 'modify', '100')",
        rusqlite::params![],
    )
    .unwrap();
    db.execute(
        "INSERT INTO activity_daily_rollups
         (day_start, source_table, activity, row_count, max_source_id, updated_at)
         VALUES (0, 'file_events', 'modify', 1, 1, '100')",
        rusqlite::params![],
    )
    .unwrap();
    db.execute(
        "INSERT INTO verification_runs (kind, status, command, source, finished_at) VALUES ('test', 'passed', 'cargo test', 'cli', '100')",
        rusqlite::params![],
    )
    .unwrap();
    db.execute(
        "INSERT INTO snapshot_runs (captured_at, file_count) VALUES ('100', 1)",
        rusqlite::params![],
    )
    .unwrap();

    let counts = collect_storage_evidence_counts(&db).unwrap();
    assert_eq!(counts.file_sightings, 1);
    assert_eq!(counts.file_events, 1);
    assert_eq!(counts.activity_daily_rollups, 1);
    assert_eq!(counts.verification_runs, 1);
    assert_eq!(counts.snapshot_runs, 1);
}
