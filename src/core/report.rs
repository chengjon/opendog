use crate::error::{OpenDogError, Result};
use serde::{Deserialize, Serialize};

mod snapshot_compare;
mod time_window;
mod usage_trend;

pub use self::snapshot_compare::{compare_latest_snapshots, compare_snapshot_runs};
pub use self::time_window::{get_time_window_report, get_time_window_report_at};
pub use self::usage_trend::{get_usage_trend_report, get_usage_trend_report_at};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportWindow {
    Hours24,
    Days7,
    Days30,
}

impl ReportWindow {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "24h" => Ok(Self::Hours24),
            "7d" => Ok(Self::Days7),
            "30d" => Ok(Self::Days30),
            _ => Err(OpenDogError::InvalidInput(format!(
                "window must be one of: 24h, 7d, 30d; got '{}'",
                value
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hours24 => "24h",
            Self::Days7 => "7d",
            Self::Days30 => "30d",
        }
    }

    fn duration_secs(self) -> i64 {
        match self {
            Self::Hours24 => 24 * 60 * 60,
            Self::Days7 => 7 * 24 * 60 * 60,
            Self::Days30 => 30 * 24 * 60 * 60,
        }
    }

    fn bucket_size_secs(self) -> i64 {
        match self {
            Self::Hours24 => 60 * 60,
            Self::Days7 | Self::Days30 => 24 * 60 * 60,
        }
    }

    fn bucket_size_label(self) -> &'static str {
        match self {
            Self::Hours24 => "1h",
            Self::Days7 | Self::Days30 => "1d",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeWindowSummary {
    pub total_sightings: i64,
    pub unique_files_accessed: i64,
    pub unique_processes: i64,
    pub modification_events: i64,
    pub modified_files: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeWindowFile {
    pub file_path: String,
    pub access_count: i64,
    pub modification_count: i64,
    pub last_seen_at: Option<String>,
    pub last_modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeWindowReport {
    pub window: String,
    pub start_time: String,
    pub end_time: String,
    pub summary: TimeWindowSummary,
    pub files: Vec<TimeWindowFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotRunInfo {
    pub run_id: i64,
    pub captured_at: String,
    pub file_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotFileVersion {
    pub size: i64,
    pub mtime: i64,
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotDiffEntry {
    pub file_path: String,
    pub change_type: String,
    pub before: Option<SnapshotFileVersion>,
    pub after: Option<SnapshotFileVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotComparisonSummary {
    pub added_files: i64,
    pub removed_files: i64,
    pub modified_files: i64,
    pub unchanged_files: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotComparison {
    pub base_run: SnapshotRunInfo,
    pub head_run: SnapshotRunInfo,
    pub summary: SnapshotComparisonSummary,
    pub changes: Vec<SnapshotDiffEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrendBucket {
    pub bucket_start: String,
    pub access_count: i64,
    pub modification_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileTrend {
    pub file_path: String,
    pub total_access_count: i64,
    pub total_modification_count: i64,
    pub current_bucket_access_count: i64,
    pub previous_bucket_access_count: i64,
    pub delta_access_count: i64,
    pub buckets: Vec<TrendBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrendSummary {
    pub bucket_size: String,
    pub bucket_count: usize,
    pub total_access_count: i64,
    pub total_modification_count: i64,
    pub tracked_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UsageTrendReport {
    pub window: String,
    pub start_time: String,
    pub end_time: String,
    pub summary: TrendSummary,
    pub files: Vec<FileTrend>,
}

pub(super) fn window_bounds(window: ReportWindow, end_ts: i64) -> (i64, i64) {
    let duration = window.duration_secs();
    let start_ts = end_ts.saturating_sub(duration).saturating_add(1);
    (start_ts, end_ts)
}

pub(super) fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
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
    fn report_window_parse_rejects_unknown_values() {
        let error = ReportWindow::parse("90d").unwrap_err();
        assert!(error.to_string().contains("window must be one of"));
    }
}
