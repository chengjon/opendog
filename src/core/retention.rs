use crate::error::{OpenDogError, Result};
use crate::storage::database::Database;
use serde::{Deserialize, Serialize};

mod executor;
mod validation;

pub use self::executor::{cleanup_project_data, cleanup_project_data_at};
use self::validation::validate_request;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CleanupScope {
    Activity,
    Snapshots,
    Verification,
    All,
}

impl CleanupScope {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "activity" => Ok(Self::Activity),
            "snapshots" => Ok(Self::Snapshots),
            "verification" => Ok(Self::Verification),
            "all" => Ok(Self::All),
            _ => Err(OpenDogError::InvalidInput(format!(
                "cleanup scope must be one of: activity, snapshots, verification, all; got '{}'",
                value
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Activity => "activity",
            Self::Snapshots => "snapshots",
            Self::Verification => "verification",
            Self::All => "all",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectDataCleanupRequest {
    pub scope: CleanupScope,
    pub older_than_days: Option<i64>,
    pub keep_snapshot_runs: Option<usize>,
    pub vacuum: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CleanupCountBreakdown {
    pub file_sightings: i64,
    pub file_events: i64,
    pub verification_runs: i64,
    pub snapshot_runs: i64,
    pub snapshot_history: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageMetrics {
    pub page_size: i64,
    pub page_count: i64,
    pub freelist_count: i64,
    pub approx_db_size_bytes: i64,
    pub approx_reclaimable_bytes: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CleanupMaintenanceStatus {
    pub optimized: bool,
    pub vacuumed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectDataCleanupResult {
    pub scope: String,
    pub dry_run: bool,
    pub older_than_days: Option<i64>,
    pub keep_snapshot_runs: Option<usize>,
    pub vacuum: bool,
    pub deleted: CleanupCountBreakdown,
    pub storage_before: StorageMetrics,
    pub storage_after: Option<StorageMetrics>,
    pub maintenance: CleanupMaintenanceStatus,
    pub notes: Vec<String>,
}

pub(super) fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub(crate) fn collect_storage_metrics(db: &Database) -> Result<StorageMetrics> {
    let page_size: i64 = db.query_row("PRAGMA page_size", rusqlite::params![], |row| row.get(0))?;
    let page_count: i64 =
        db.query_row("PRAGMA page_count", rusqlite::params![], |row| row.get(0))?;
    let freelist_count: i64 =
        db.query_row("PRAGMA freelist_count", rusqlite::params![], |row| {
            row.get(0)
        })?;

    Ok(StorageMetrics {
        page_size,
        page_count,
        freelist_count,
        approx_db_size_bytes: page_size.saturating_mul(page_count),
        approx_reclaimable_bytes: page_size.saturating_mul(freelist_count),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open_project(&db_path).unwrap();
        Box::leak(Box::new(dir));
        db
    }

    fn count(db: &Database, table: &str) -> i64 {
        db.query_row(
            &format!("SELECT COUNT(*) FROM {}", table),
            params![],
            |row| row.get(0),
        )
        .unwrap()
    }

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
        assert!(result.storage_before.page_count >= 1);
        assert!(result.storage_before.approx_db_size_bytes >= result.storage_before.page_size);
        assert_eq!(result.storage_after, None);
        assert_eq!(result.maintenance, CleanupMaintenanceStatus::default());
        assert_eq!(count(&db, "file_sightings"), 2);
        assert_eq!(count(&db, "file_events"), 2);
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
        assert_eq!(result.deleted.verification_runs, 1);
        assert_eq!(result.deleted.snapshot_runs, 2);
        assert_eq!(result.deleted.snapshot_history, 2);
        assert!(result.maintenance.optimized);
        assert!(result.maintenance.vacuumed);
        let storage_after = result.storage_after.as_ref().unwrap();
        assert!(storage_after.page_count >= 1);
        assert!(
            storage_after.approx_reclaimable_bytes <= result.storage_before.approx_db_size_bytes
        );
        assert_eq!(count(&db, "snapshot"), 1);
        assert_eq!(count(&db, "file_stats"), 1);
        assert_eq!(count(&db, "snapshot_runs"), 1);
        assert_eq!(count(&db, "snapshot_history"), 1);
    }

    #[test]
    fn cleanup_validation_requires_explicit_action_parameters() {
        let db = test_db();

        let error = cleanup_project_data_at(
            &db,
            &ProjectDataCleanupRequest {
                scope: CleanupScope::All,
                older_than_days: None,
                keep_snapshot_runs: None,
                vacuum: false,
                dry_run: true,
            },
            1,
        )
        .unwrap_err();

        assert!(error.to_string().contains("cleanup"));
    }

    #[test]
    fn cleanup_validation_rejects_vacuum_in_dry_run_mode() {
        let db = test_db();

        let error = cleanup_project_data_at(
            &db,
            &ProjectDataCleanupRequest {
                scope: CleanupScope::Activity,
                older_than_days: Some(1),
                keep_snapshot_runs: None,
                vacuum: true,
                dry_run: true,
            },
            1,
        )
        .unwrap_err();

        assert!(error.to_string().contains("vacuum"));
    }

    #[test]
    fn cleanup_scope_parse_rejects_invalid_value() {
        let err = CleanupScope::parse("everything").unwrap_err();
        assert!(err.to_string().contains("cleanup scope must be one of: activity, snapshots, verification, all"));
        assert!(err.to_string().contains("everything"));
    }

    #[test]
    fn cleanup_scope_parse_activity() {
        assert_eq!(CleanupScope::parse("activity").unwrap(), CleanupScope::Activity);
    }

    #[test]
    fn cleanup_scope_parse_snapshots() {
        assert_eq!(CleanupScope::parse("snapshots").unwrap(), CleanupScope::Snapshots);
    }

    #[test]
    fn cleanup_scope_parse_verification() {
        assert_eq!(CleanupScope::parse("verification").unwrap(), CleanupScope::Verification);
    }

    #[test]
    fn cleanup_scope_parse_all() {
        assert_eq!(CleanupScope::parse("all").unwrap(), CleanupScope::All);
    }

    #[test]
    fn cleanup_scope_parse_empty_string_is_error() {
        assert!(CleanupScope::parse("").is_err());
    }

    #[test]
    fn cleanup_scope_as_str_activity() {
        assert_eq!(CleanupScope::Activity.as_str(), "activity");
    }

    #[test]
    fn cleanup_scope_as_str_snapshots() {
        assert_eq!(CleanupScope::Snapshots.as_str(), "snapshots");
    }

    #[test]
    fn cleanup_scope_as_str_verification() {
        assert_eq!(CleanupScope::Verification.as_str(), "verification");
    }

    #[test]
    fn cleanup_scope_as_str_all() {
        assert_eq!(CleanupScope::All.as_str(), "all");
    }

    #[test]
    fn cleanup_scope_roundtrip_parse_as_str() {
        for (label, scope) in [
            ("activity", CleanupScope::Activity),
            ("snapshots", CleanupScope::Snapshots),
            ("verification", CleanupScope::Verification),
            ("all", CleanupScope::All),
        ] {
            assert_eq!(CleanupScope::parse(label).unwrap(), scope);
            assert_eq!(scope.as_str(), label);
        }
    }

    // -----------------------------------------------------------------------
    // Snapshot-only cleanup integration tests
    // -----------------------------------------------------------------------

    fn seed_snapshot_runs(db: &Database, timestamps: &[&str], files_per_run: &[&[(&str, i64)]]) {
        use rusqlite::params;
        for (i, ts) in timestamps.iter().enumerate() {
            db.execute(
                "INSERT INTO snapshot_runs (captured_at, file_count) VALUES (?1, ?2)",
                params![ts, files_per_run[i].len() as i64],
            )
            .unwrap();
            let run_id: i64 = db
                .query_row(
                    "SELECT last_insert_rowid()",
                    params![],
                    |row| row.get(0),
                )
                .unwrap();
            for (path, size) in files_per_run[i] {
                db.execute(
                    "INSERT INTO snapshot_history (run_id, path, size, mtime, file_type) VALUES (?1, ?2, ?3, 1, 'rs')",
                    params![run_id, path, size],
                )
                .unwrap();
            }
        }
    }

    /// Snapshots-only cleanup with keep_snapshot_runs=2 prunes older runs.
    #[test]
    fn snapshots_only_cleanup_keeps_latest_two_runs() {
        let db = test_db();
        let now = 3_000_000i64;

        seed_snapshot_runs(
            &db,
            &[
                "100",
                "200",
                "300",
                "400",
            ],
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
        assert_eq!(result.deleted.file_sightings, 0, "snapshots-only must not touch activity");
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
            result.notes.iter().any(|n| n.contains("fewer than 2 snapshot runs")),
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
        assert_eq!(result.deleted.snapshot_runs, 2, "dry run should count but not delete");
        assert_eq!(result.deleted.snapshot_history, 2);

        // Nothing actually deleted.
        assert_eq!(count(&db, "snapshot_runs"), 3);
        assert_eq!(count(&db, "snapshot_history"), 3);
    }
}
