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
    pub truncated: bool,
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
    pub bucket_start: i64,
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
mod tests;
