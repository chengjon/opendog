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
}
