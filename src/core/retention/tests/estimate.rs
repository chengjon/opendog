use super::*;

#[test]
fn estimate_mode_full_when_below_threshold() {
    let db = test_db();
    let now = 3_000_000i64;

    // Seed fewer than 100 runs
    for i in 0..5 {
        db.execute(
            "INSERT INTO snapshot_runs (captured_at, file_count) VALUES (?1, 1)",
            params![(now - (i as i64 + 1) * 86_400).to_string()],
        )
        .unwrap();
    }

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

    assert_eq!(result.estimate_mode, EstimateMode::Full);
    assert_eq!(result.deleted.snapshot_runs, 4);
    assert!(result.deleted.snapshot_history == 0);
}

#[test]
fn estimate_mode_scope_counts_only_above_threshold() {
    let db = test_db();
    let now = 3_000_000i64;

    // Seed 105 runs (above threshold of 100)
    for i in 0..105 {
        db.execute(
            "INSERT INTO snapshot_runs (captured_at, file_count) VALUES (?1, 1)",
            params![(now - (i as i64 + 1) * 60).to_string()],
        )
        .unwrap();
    }

    assert_eq!(count(&db, "snapshot_runs"), 105);

    let result = cleanup_project_data_at(
        &db,
        &ProjectDataCleanupRequest {
            scope: CleanupScope::Snapshots,
            older_than_days: None,
            keep_snapshot_runs: Some(5),
            vacuum: false,
            dry_run: true,
        },
        now,
    )
    .unwrap();

    assert_eq!(result.estimate_mode, EstimateMode::ScopeCountsOnly);
    assert_eq!(
        result.deleted.snapshot_runs, 100,
        "prunable = total - keep_latest"
    );
    assert_eq!(
        result.deleted.snapshot_history, 0,
        "history skipped in estimate mode"
    );
    assert!(result
        .notes
        .iter()
        .any(|n| n.contains("estimate-only mode")));
    // Nothing deleted
    assert_eq!(count(&db, "snapshot_runs"), 105);
}

#[test]
fn estimate_mode_not_used_for_real_cleanup() {
    let db = test_db();
    let now = 3_000_000i64;

    // Seed 105 runs — above threshold, but NOT a dry run
    for i in 0..105 {
        db.execute(
            "INSERT INTO snapshot_runs (captured_at, file_count) VALUES (?1, 1)",
            params![(now - (i as i64 + 1) * 60).to_string()],
        )
        .unwrap();
    }

    let result = cleanup_project_data_at(
        &db,
        &ProjectDataCleanupRequest {
            scope: CleanupScope::Snapshots,
            older_than_days: None,
            keep_snapshot_runs: Some(5),
            vacuum: false,
            dry_run: false,
        },
        now,
    )
    .unwrap();

    assert_eq!(
        result.estimate_mode,
        EstimateMode::Full,
        "real cleanup always uses full mode"
    );
    assert_eq!(result.deleted.snapshot_runs, 100);
    assert_eq!(count(&db, "snapshot_runs"), 5);
}
