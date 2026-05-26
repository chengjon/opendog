use super::*;

#[test]
fn snapshots_only_cleanup_keeps_latest_two_runs() {
    let db = test_db();
    let now = 3_000_000i64;

    seed_snapshot_runs(
        &db,
        &["100", "200", "300", "400"],
        &[
            &[("src/a.rs", 10)],
            &[("src/b.rs", 20)],
            &[("src/c.rs", 30)],
            &[("src/d.rs", 40)],
        ],
    );

    // Seed some activity data — snapshots-only scope should NOT touch these.
    db.execute(
        "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
        params!["src/old.rs", "codex", 10i64, (now - 10 * 86_400).to_string()],
    )
    .unwrap();

    assert_eq!(count(&db, "snapshot_runs"), 4);
    assert_eq!(count(&db, "snapshot_history"), 4);
    assert_eq!(count(&db, "file_sightings"), 1);

    let result = cleanup_project_data_at(
        &db,
        &ProjectDataCleanupRequest {
            scope: CleanupScope::Snapshots,
            older_than_days: None,
            keep_snapshot_runs: Some(2),
            vacuum: false,
            dry_run: false,
        },
        now,
    )
    .unwrap();

    // Two oldest runs should be pruned.
    assert_eq!(result.deleted.snapshot_runs, 2);
    assert_eq!(result.deleted.snapshot_history, 2);
    assert_eq!(
        result.deleted.file_sightings, 0,
        "snapshots-only must not touch activity"
    );
    assert_eq!(result.deleted.file_events, 0);

    // Two most recent runs remain.
    assert_eq!(count(&db, "snapshot_runs"), 2);
    assert_eq!(count(&db, "snapshot_history"), 2);
    assert_eq!(count(&db, "file_sightings"), 1, "activity data untouched");
}

/// Snapshots-only cleanup with keep_snapshot_runs=1 (the <2 warning note path).
#[test]
fn snapshots_only_cleanup_keep_one_produces_comparison_warning() {
    let db = test_db();
    let now = 3_000_000i64;

    seed_snapshot_runs(
        &db,
        &["100", "200", "300"],
        &[
            &[("src/a.rs", 10)],
            &[("src/b.rs", 20)],
            &[("src/c.rs", 30)],
        ],
    );

    assert_eq!(count(&db, "snapshot_runs"), 3);

    let result = cleanup_project_data_at(
        &db,
        &ProjectDataCleanupRequest {
            scope: CleanupScope::Snapshots,
            older_than_days: None,
            keep_snapshot_runs: Some(1),
            vacuum: false,
            dry_run: false,
        },
        now,
    )
    .unwrap();

    // Two oldest runs pruned, only the latest kept.
    assert_eq!(result.deleted.snapshot_runs, 2);
    assert_eq!(result.deleted.snapshot_history, 2);
    assert_eq!(count(&db, "snapshot_runs"), 1);
    assert_eq!(count(&db, "snapshot_history"), 1);

    // The notes should include the comparison warning for keep < 2.
    assert!(
        result
            .notes
            .iter()
            .any(|n| n.contains("fewer than 2 snapshot runs")),
        "should warn about keeping fewer than 2 snapshot runs, got notes: {:?}",
        result.notes,
    );
}

/// Snapshots-only cleanup as dry_run should not delete anything.
#[test]
fn snapshots_only_dry_run_does_not_delete() {
    let db = test_db();
    let now = 3_000_000i64;

    seed_snapshot_runs(
        &db,
        &["100", "200", "300"],
        &[
            &[("src/a.rs", 10)],
            &[("src/b.rs", 20)],
            &[("src/c.rs", 30)],
        ],
    );

    let result = cleanup_project_data_at(
        &db,
        &ProjectDataCleanupRequest {
            scope: CleanupScope::Snapshots,
            older_than_days: None,
            keep_snapshot_runs: Some(1),
            vacuum: false,
            dry_run: true,
        },
        now,
    )
    .unwrap();

    assert!(result.dry_run);
    assert_eq!(
        result.deleted.snapshot_runs, 2,
        "dry run should count but not delete"
    );
    assert_eq!(result.deleted.snapshot_history, 2);

    // Nothing actually deleted.
    assert_eq!(count(&db, "snapshot_runs"), 3);
    assert_eq!(count(&db, "snapshot_history"), 3);
}

// -----------------------------------------------------------------------
// Storage metrics and scope-isolation tests
// -----------------------------------------------------------------------
