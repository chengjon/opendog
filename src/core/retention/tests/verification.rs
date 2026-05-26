use super::*;

#[test]
fn cleanup_verification_scope_only_touches_verification_runs() {
    let db = test_db();
    let now = 3_000_000i64;

    // Seed verification runs
    db.execute(
        "INSERT INTO verification_runs (kind, status, command, source, finished_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params!["test", "passed", "cargo test", "cli", (now - 14 * 86_400).to_string()],
    ).unwrap();
    db.execute(
        "INSERT INTO verification_runs (kind, status, command, source, finished_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params!["lint", "passed", "cargo clippy", "cli", (now - 30).to_string()],
    ).unwrap();

    // Seed activity data — verification scope must NOT touch these
    db.execute(
        "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
        params!["src/old.rs", "codex", 10i64, (now - 10 * 86_400).to_string()],
    ).unwrap();

    assert_eq!(count(&db, "verification_runs"), 2);
    assert_eq!(count(&db, "file_sightings"), 1);

    let result = cleanup_project_data_at(
        &db,
        &ProjectDataCleanupRequest {
            scope: CleanupScope::Verification,
            older_than_days: Some(7),
            keep_snapshot_runs: None,
            vacuum: false,
            dry_run: false,
        },
        now,
    )
    .unwrap();

    assert_eq!(result.deleted.verification_runs, 1);
    assert_eq!(
        result.deleted.file_sightings, 0,
        "verification scope must not touch activity"
    );
    assert_eq!(result.deleted.file_events, 0);
    assert_eq!(count(&db, "verification_runs"), 1);
    assert_eq!(count(&db, "file_sightings"), 1, "activity data untouched");
}

#[test]
fn cleanup_dry_run_for_verification_scope_counts_but_preserves() {
    let db = test_db();
    let now = 3_000_000i64;

    db.execute(
        "INSERT INTO verification_runs (kind, status, command, source, finished_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params!["test", "passed", "cargo test", "cli", (now - 14 * 86_400).to_string()],
    ).unwrap();

    let result = cleanup_project_data_at(
        &db,
        &ProjectDataCleanupRequest {
            scope: CleanupScope::Verification,
            older_than_days: Some(7),
            keep_snapshot_runs: None,
            vacuum: false,
            dry_run: true,
        },
        now,
    )
    .unwrap();

    assert!(result.dry_run);
    assert_eq!(result.deleted.verification_runs, 1);
    assert_eq!(
        count(&db, "verification_runs"),
        1,
        "dry run must not delete"
    );
}

#[test]
fn cleanup_no_matching_rows_produces_no_match_note() {
    let db = test_db();
    let now = 3_000_000i64;

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

    assert_eq!(result.deleted.file_sightings, 0);
    assert_eq!(result.deleted.file_events, 0);
    assert!(result
        .notes
        .iter()
        .any(|n| n.contains("no matching retained rows")));
}

#[test]
fn cleanup_activity_scope_preserves_snapshots_and_verification() {
    let db = test_db();
    let now = 3_000_000i64;

    // Seed snapshot data
    db.execute(
        "INSERT INTO snapshot_runs (captured_at, file_count) VALUES (?1, 1)",
        params![(now - 86_400).to_string()],
    )
    .unwrap();

    // Seed verification data
    db.execute(
        "INSERT INTO verification_runs (kind, status, command, source, finished_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params!["test", "passed", "cargo test", "cli", (now - 86_400).to_string()],
    ).unwrap();

    // Seed activity to be cleaned
    db.execute(
        "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
        params!["src/old.rs", "codex", 10i64, (now - 10 * 86_400).to_string()],
    ).unwrap();

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

    assert_eq!(result.deleted.file_sightings, 1);
    assert_eq!(
        result.deleted.snapshot_runs, 0,
        "activity scope must not touch snapshots"
    );
    assert_eq!(
        result.deleted.verification_runs, 0,
        "activity scope must not touch verification"
    );
    assert_eq!(count(&db, "snapshot_runs"), 1);
    assert_eq!(count(&db, "verification_runs"), 1);
}
