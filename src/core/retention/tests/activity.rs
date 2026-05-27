use super::*;

#[test]
fn cleanup_dry_run_counts_old_activity_without_deleting_rows() {
    let db = test_db();
    let now = 2_000_000i64;

    db.execute(
        "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
        params!["src/old.rs", "codex", 10i64, (now - 10 * 86_400).to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
        params!["src/new.rs", "codex", 11i64, (now - 60).to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO file_events (file_path, event_type, event_time) VALUES (?1, 'modify', ?2)",
        params!["src/old.rs", (now - 9 * 86_400).to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO file_events (file_path, event_type, event_time) VALUES (?1, 'modify', ?2)",
        params!["src/new.rs", (now - 30).to_string()],
    )
    .unwrap();

    let result = cleanup_project_data_at(
        &db,
        &ProjectDataCleanupRequest {
            scope: CleanupScope::Activity,
            older_than_days: Some(7),
            keep_snapshot_runs: None,
            vacuum: false,
            dry_run: true,
        },
        now,
    )
    .unwrap();

    assert!(result.dry_run);
    assert!(!result.vacuum);
    assert_eq!(result.deleted.file_sightings, 1);
    assert_eq!(result.deleted.file_events, 1);
    assert_eq!(
        result.rolled_up,
        crate::storage::queries::ActivityRollupCounts::default()
    );
    assert!(result.storage_before.page_count >= 1);
    assert!(result.storage_before.approx_db_size_bytes >= result.storage_before.page_size);
    assert_eq!(result.storage_after, None);
    assert_eq!(result.maintenance, CleanupMaintenanceStatus::default());
    assert_eq!(count(&db, "file_sightings"), 2);
    assert_eq!(count(&db, "file_events"), 2);
}

#[test]
fn cleanup_activity_rolls_up_old_activity_before_deleting_raw_rows() {
    let db = test_db();
    let now = 2_000_000i64;

    db.execute(
        "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
        params!["src/old-a.rs", "codex", 10i64, (now - 10 * 86_400).to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
        params!["src/old-b.rs", "codex", 11i64, (now - 10 * 86_400 + 1).to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
        params!["src/new.rs", "codex", 12i64, (now - 60).to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO file_events (file_path, event_type, event_time) VALUES (?1, 'modify', ?2)",
        params!["src/old-a.rs", (now - 9 * 86_400).to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO file_events (file_path, event_type, event_time) VALUES (?1, 'modify', ?2)",
        params!["src/new.rs", (now - 30).to_string()],
    )
    .unwrap();

    let result = cleanup_project_data_at(
        &db,
        &ProjectDataCleanupRequest {
            scope: CleanupScope::Activity,
            older_than_days: Some(7),
            keep_snapshot_runs: None,
            vacuum: false,
            dry_run: false,
        },
        now,
    )
    .unwrap();

    assert_eq!(result.deleted.file_sightings, 2);
    assert_eq!(result.deleted.file_events, 1);
    assert_eq!(result.rolled_up.file_sightings, 2);
    assert_eq!(result.rolled_up.file_events, 1);
    assert_eq!(count(&db, "file_sightings"), 1);
    assert_eq!(count(&db, "file_events"), 1);
    assert_eq!(count(&db, "activity_daily_rollups"), 2);
    assert!(result
        .notes
        .iter()
        .any(|note| note.contains("rolls up daily activity counts")));
}

#[test]
fn cleanup_all_can_prune_old_history_without_touching_current_snapshot_or_stats() {
    let db = test_db();
    let now = 3_000_000i64;

    db.execute(
        "INSERT INTO snapshot (path, size, mtime, file_type, scan_timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        params!["src/live.rs", 10i64, 1i64, "rs", now.to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO file_stats (file_path, access_count, estimated_duration_ms, modification_count, first_seen_time, last_updated)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params!["src/live.rs", 5i64, 100i64, 1i64, "1", now.to_string()],
    )
    .unwrap();

    for (offset_days, run_id) in [(30, 1i64), (20, 2i64), (1, 3i64)] {
        db.execute(
            "INSERT INTO snapshot_runs (id, captured_at, file_count) VALUES (?1, ?2, 1)",
            params![run_id, (now - offset_days * 86_400).to_string()],
        )
        .unwrap();
        db.execute(
            "INSERT INTO snapshot_history (run_id, path, size, mtime, file_type) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![run_id, format!("src/run-{}.rs", run_id), 10i64, run_id, "rs"],
        )
        .unwrap();
    }

    db.execute(
        "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
        params!["src/old.rs", "codex", 10i64, (now - 15 * 86_400).to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
        params!["src/new.rs", "codex", 11i64, (now - 60).to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO file_events (file_path, event_type, event_time) VALUES (?1, 'modify', ?2)",
        params!["src/old.rs", (now - 12 * 86_400).to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO verification_runs (kind, status, command, source, finished_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params!["test", "passed", "cargo test", "cli", (now - 14 * 86_400).to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO verification_runs (kind, status, command, source, finished_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params!["lint", "passed", "cargo clippy", "cli", (now - 30).to_string()],
    )
    .unwrap();

    let result = cleanup_project_data_at(
        &db,
        &ProjectDataCleanupRequest {
            scope: CleanupScope::All,
            older_than_days: Some(7),
            keep_snapshot_runs: Some(1),
            vacuum: true,
            dry_run: false,
        },
        now,
    )
    .unwrap();

    assert!(!result.dry_run);
    assert!(result.vacuum);
    assert_eq!(result.deleted.file_sightings, 1);
    assert_eq!(result.deleted.file_events, 1);
    assert_eq!(result.rolled_up.file_sightings, 1);
    assert_eq!(result.rolled_up.file_events, 1);
    assert_eq!(result.deleted.verification_runs, 1);
    assert_eq!(result.deleted.snapshot_runs, 2);
    assert_eq!(result.deleted.snapshot_history, 2);
    assert!(result.maintenance.optimized);
    assert!(result.maintenance.vacuumed);
    let storage_after = result.storage_after.as_ref().unwrap();
    assert!(storage_after.page_count >= 1);
    assert!(storage_after.approx_reclaimable_bytes <= result.storage_before.approx_db_size_bytes);
    assert_eq!(count(&db, "snapshot"), 1);
    assert_eq!(count(&db, "file_stats"), 1);
    assert_eq!(count(&db, "snapshot_runs"), 1);
    assert_eq!(count(&db, "snapshot_history"), 1);
}
