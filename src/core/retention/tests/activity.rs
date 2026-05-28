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
