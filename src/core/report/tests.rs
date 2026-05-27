use super::*;
use crate::config::ProjectConfig;
use crate::core::snapshot;
use crate::storage::database::Database;
use rusqlite::params;

fn test_db() -> Database {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open_project(&db_path).unwrap();
    Box::leak(Box::new(dir));
    db
}

fn insert_sighting(db: &Database, path: &str, process_name: &str, pid: i64, seen_at: i64) {
    db.execute(
        "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
        params![path, process_name, pid, seen_at.to_string()],
    )
    .unwrap();
}

fn insert_modify_event(db: &Database, path: &str, event_time: i64) {
    db.execute(
        "INSERT INTO file_events (file_path, event_type, event_time) VALUES (?1, 'modify', ?2)",
        params![path, event_time.to_string()],
    )
    .unwrap();
}

#[test]
fn time_window_report_respects_24h_7d_and_30d_boundaries() {
    let db = test_db();
    let end_ts = 2_000_000i64;

    insert_sighting(&db, "src/main.rs", "codex", 10, end_ts - 3600);
    insert_sighting(&db, "src/main.rs", "codex", 10, end_ts - 1800);
    insert_sighting(&db, "src/lib.rs", "codex", 10, end_ts - 2 * 86_400);
    insert_sighting(&db, "src/legacy.rs", "codex", 10, end_ts - 10 * 86_400);

    insert_modify_event(&db, "src/main.rs", end_ts - 1200);
    insert_modify_event(&db, "src/lib.rs", end_ts - 3 * 86_400);
    insert_modify_event(&db, "src/legacy.rs", end_ts - 40 * 86_400);

    let report_24h = get_time_window_report_at(&db, ReportWindow::Hours24, end_ts, 10).unwrap();
    assert_eq!(report_24h.window, "24h");
    assert_eq!(report_24h.summary.total_sightings, 2);
    assert_eq!(report_24h.summary.unique_files_accessed, 1);
    assert_eq!(report_24h.summary.modification_events, 1);
    assert_eq!(report_24h.files.len(), 1);
    assert_eq!(report_24h.files[0].file_path, "src/main.rs");
    assert_eq!(report_24h.files[0].access_count, 2);
    assert_eq!(report_24h.files[0].modification_count, 1);

    let report_7d = get_time_window_report_at(&db, ReportWindow::Days7, end_ts, 10).unwrap();
    assert_eq!(report_7d.summary.total_sightings, 3);
    assert_eq!(report_7d.summary.unique_files_accessed, 2);
    assert_eq!(report_7d.summary.modification_events, 2);
    assert_eq!(report_7d.files.len(), 2);
    assert_eq!(report_7d.files[0].file_path, "src/main.rs");
    assert_eq!(report_7d.files[1].file_path, "src/lib.rs");

    let report_30d = get_time_window_report_at(&db, ReportWindow::Days30, end_ts, 10).unwrap();
    assert_eq!(report_30d.summary.total_sightings, 4);
    assert_eq!(report_30d.summary.unique_files_accessed, 3);
}

#[test]
fn snapshot_comparison_detects_added_removed_and_modified_files() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("project.db");
    let db = Database::open_project(&db_path).unwrap();
    let project_dir = dir.path().join("project");
    std::fs::create_dir_all(&project_dir).unwrap();

    std::fs::write(project_dir.join("a.txt"), "alpha").unwrap();
    std::fs::write(project_dir.join("b.txt"), "beta").unwrap();
    snapshot::take_snapshot(&db, &project_dir, &ProjectConfig::default()).unwrap();

    std::fs::remove_file(project_dir.join("a.txt")).unwrap();
    std::fs::write(project_dir.join("b.txt"), "beta-updated").unwrap();
    std::fs::write(project_dir.join("c.txt"), "charlie").unwrap();
    snapshot::take_snapshot(&db, &project_dir, &ProjectConfig::default()).unwrap();

    let comparison = compare_latest_snapshots(&db, 10).unwrap();
    assert_eq!(comparison.base_run.file_count, 2);
    assert_eq!(comparison.head_run.file_count, 2);
    assert_eq!(comparison.summary.added_files, 1);
    assert_eq!(comparison.summary.removed_files, 1);
    assert_eq!(comparison.summary.modified_files, 1);
    assert_eq!(comparison.summary.unchanged_files, 0);
    assert_eq!(comparison.changes.len(), 3);
    assert!(comparison
        .changes
        .iter()
        .any(|entry| entry.file_path == "a.txt" && entry.change_type == "removed"));
    assert!(comparison
        .changes
        .iter()
        .any(|entry| entry.file_path == "b.txt" && entry.change_type == "modified"));
    assert!(comparison
        .changes
        .iter()
        .any(|entry| entry.file_path == "c.txt" && entry.change_type == "added"));
}

#[test]
fn usage_trend_report_builds_bucketed_deltas() {
    let db = test_db();
    let end_ts = 3_000_000i64;

    insert_sighting(&db, "src/hot.rs", "codex", 11, end_ts - 86_400 - 100);
    insert_sighting(&db, "src/hot.rs", "codex", 11, end_ts - 300);
    insert_sighting(&db, "src/hot.rs", "codex", 11, end_ts - 200);
    insert_sighting(&db, "src/hot.rs", "codex", 11, end_ts - 100);

    insert_sighting(&db, "src/cool.rs", "codex", 12, end_ts - 86_400 - 200);
    insert_sighting(&db, "src/cool.rs", "codex", 12, end_ts - 86_400 - 100);
    insert_sighting(&db, "src/cool.rs", "codex", 12, end_ts - 100);

    insert_modify_event(&db, "src/hot.rs", end_ts - 250);
    insert_modify_event(&db, "src/cool.rs", end_ts - 86_400 - 150);

    let report = get_usage_trend_report_at(&db, ReportWindow::Days7, end_ts, 10).unwrap();
    assert_eq!(report.window, "7d");
    assert_eq!(report.summary.bucket_size, "1d");
    assert_eq!(report.summary.bucket_count, 7);

    let hot = report
        .files
        .iter()
        .find(|entry| entry.file_path == "src/hot.rs")
        .unwrap();
    assert_eq!(hot.total_access_count, 4);
    assert_eq!(hot.total_modification_count, 1);
    assert_eq!(hot.current_bucket_access_count, 3);
    assert_eq!(hot.previous_bucket_access_count, 1);
    assert_eq!(hot.delta_access_count, 2);

    let cool = report
        .files
        .iter()
        .find(|entry| entry.file_path == "src/cool.rs")
        .unwrap();
    assert_eq!(cool.current_bucket_access_count, 1);
    assert_eq!(cool.previous_bucket_access_count, 2);
    assert_eq!(cool.delta_access_count, -1);
}

#[test]
fn activity_rollup_report_reads_daily_rollups() {
    let db = test_db();
    let end_ts = 3_000_000i64;
    let today = (end_ts / 86_400) * 86_400;
    let yesterday = today - 86_400;

    db.execute(
        "INSERT INTO activity_daily_rollups
         (day_start, source_table, activity, row_count, max_source_id, updated_at)
         VALUES (?1, 'file_sightings', 'seen', 8, 10, ?2)",
        params![yesterday, end_ts.to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO activity_daily_rollups
         (day_start, source_table, activity, row_count, max_source_id, updated_at)
         VALUES (?1, 'file_events', 'modify', 3, 20, ?2)",
        params![yesterday, end_ts.to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO activity_daily_rollups
         (day_start, source_table, activity, row_count, max_source_id, updated_at)
         VALUES (?1, 'file_sightings', 'seen', 2, 12, ?2)",
        params![today, end_ts.to_string()],
    )
    .unwrap();

    let report = get_activity_rollup_report_at(&db, ReportWindow::Days7, end_ts, 10).unwrap();

    assert_eq!(report.window, "7d");
    assert_eq!(report.summary.bucket_size, "1d");
    assert_eq!(report.summary.total_access_count, 10);
    assert_eq!(report.summary.total_modification_count, 3);
    assert_eq!(report.summary.rollup_days, 2);
    assert_eq!(report.summary.returned_days, 2);
    assert!(!report.summary.truncated);
    assert_eq!(report.days[0].day_start, yesterday);
    assert_eq!(report.days[0].access_count, 8);
    assert_eq!(report.days[0].modification_count, 3);
    assert_eq!(report.days[1].day_start, today);
    assert_eq!(report.days[1].access_count, 2);
}

#[test]
fn activity_rollup_report_summary_counts_all_days_when_limited() {
    let db = test_db();
    let end_ts = 3_000_000i64;
    let today = (end_ts / 86_400) * 86_400;

    for (index, day) in [today - 2 * 86_400, today - 86_400, today]
        .iter()
        .enumerate()
    {
        db.execute(
            "INSERT INTO activity_daily_rollups
             (day_start, source_table, activity, row_count, max_source_id, updated_at)
             VALUES (?1, 'file_sightings', 'seen', ?2, ?3, ?4)",
            params![
                day,
                (index + 1) as i64,
                (index + 10) as i64,
                end_ts.to_string()
            ],
        )
        .unwrap();
    }

    let report = get_activity_rollup_report_at(&db, ReportWindow::Days7, end_ts, 2).unwrap();

    assert_eq!(report.summary.total_access_count, 6);
    assert_eq!(report.summary.rollup_days, 3);
    assert_eq!(report.summary.returned_days, 2);
    assert!(report.summary.truncated);
    assert_eq!(report.days.len(), 2);
}

#[test]
fn report_window_parse_rejects_unknown_values() {
    let error = ReportWindow::parse("90d").unwrap_err();
    assert!(error.to_string().contains("window must be one of"));
}

#[test]
fn compare_latest_snapshots_rejects_fewer_than_two_runs() {
    let db = test_db();
    let err = compare_latest_snapshots(&db, 10).unwrap_err();
    assert!(
        err.to_string().contains("at least two snapshot runs"),
        "expected insufficient-runs error, got: {err}"
    );
}

#[test]
fn compare_snapshot_runs_rejects_equal_run_ids() {
    let db = test_db();
    let err = compare_snapshot_runs(&db, 42, 42, 10).unwrap_err();
    assert!(
        err.to_string().contains("must differ"),
        "expected equal-ids error, got: {err}"
    );
}

#[test]
fn compare_snapshot_runs_rejects_missing_run() {
    let db = test_db();
    let err = compare_snapshot_runs(&db, 999, 998, 10).unwrap_err();
    assert!(
        err.to_string().contains("not found"),
        "expected missing-run error, got: {err}"
    );
}

#[test]
fn time_window_empty_db_returns_zero_counts() {
    let db = test_db();
    let report = get_time_window_report_at(&db, ReportWindow::Days7, 1_000_000, 10).unwrap();
    assert_eq!(report.summary.total_sightings, 0);
    assert_eq!(report.summary.unique_files_accessed, 0);
    assert_eq!(report.files.len(), 0);
}

#[test]
fn time_window_single_file_at_exact_boundary() {
    let db = test_db();
    let end_ts = 2_000_000i64;
    let start_ts = end_ts - 24 * 60 * 60 + 1; // window_bounds offset
                                              // Insert exactly at start boundary — should be included
    insert_sighting(&db, "boundary.rs", "codex", 1, start_ts);
    let report = get_time_window_report_at(&db, ReportWindow::Hours24, end_ts, 10).unwrap();
    assert_eq!(report.summary.total_sightings, 1);
}

#[test]
fn usage_trend_empty_db_returns_empty_files() {
    let db = test_db();
    let report = get_usage_trend_report_at(&db, ReportWindow::Days7, 1_000_000, 10).unwrap();
    assert_eq!(report.summary.tracked_files, 0);
    assert!(report.files.is_empty());
}

#[test]
fn window_bounds_calculates_correct_range() {
    let (start, end) = window_bounds(ReportWindow::Hours24, 100_000);
    assert_eq!(start, 100_000 - 24 * 60 * 60 + 1);
    assert_eq!(end, 100_000);
}

#[test]
fn report_window_as_str_roundtrip() {
    assert_eq!(ReportWindow::Hours24.as_str(), "24h");
    assert_eq!(ReportWindow::Days7.as_str(), "7d");
    assert_eq!(ReportWindow::Days30.as_str(), "30d");
}

#[test]
fn report_window_duration_secs_matches_window_name() {
    assert_eq!(ReportWindow::Hours24.duration_secs(), 24 * 60 * 60);
    assert_eq!(ReportWindow::Days7.duration_secs(), 7 * 24 * 60 * 60);
    assert_eq!(ReportWindow::Days30.duration_secs(), 30 * 24 * 60 * 60);
}

#[test]
fn report_window_bucket_size_secs_returns_hourly_or_daily() {
    assert_eq!(ReportWindow::Hours24.bucket_size_secs(), 60 * 60);
    assert_eq!(ReportWindow::Days7.bucket_size_secs(), 24 * 60 * 60);
    assert_eq!(ReportWindow::Days30.bucket_size_secs(), 24 * 60 * 60);
}

#[test]
fn report_window_bucket_size_label_returns_hour_or_day() {
    assert_eq!(ReportWindow::Hours24.bucket_size_label(), "1h");
    assert_eq!(ReportWindow::Days7.bucket_size_label(), "1d");
    assert_eq!(ReportWindow::Days30.bucket_size_label(), "1d");
}

#[test]
fn report_window_parse_as_str_roundtrip() {
    for input in &["24h", "7d", "30d"] {
        let window = ReportWindow::parse(input).unwrap();
        assert_eq!(window.as_str(), *input);
    }
}

#[test]
fn time_window_report_sql_limit_truncates_files() {
    let db = test_db();
    let end_ts = 2_000_000i64;
    for i in 0..5 {
        insert_sighting(&db, &format!("file{}.rs", i), "claude", 1, end_ts - 100 + i);
    }
    let report = get_time_window_report_at(&db, ReportWindow::Hours24, end_ts, 2).unwrap();
    assert!(report.files.len() <= 2, "should respect limit at SQL level");
    assert!(
        report.truncated,
        "truncated should be true when more files exist than the returned limit"
    );
    assert_eq!(
        report.summary.total_sightings, 5,
        "summary should still count all"
    );
}
