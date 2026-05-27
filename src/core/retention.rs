use crate::error::{OpenDogError, Result};
use crate::storage::{database::Database, queries};
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
pub struct StorageEvidenceCounts {
    pub file_sightings: i64,
    pub file_events: i64,
    pub activity_daily_rollups: i64,
    pub verification_runs: i64,
    pub snapshot_runs: i64,
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
    pub rolled_up: queries::ActivityRollupCounts,
    pub storage_before: StorageMetrics,
    pub storage_after: Option<StorageMetrics>,
    pub maintenance: CleanupMaintenanceStatus,
    pub notes: Vec<String>,
    pub estimate_mode: EstimateMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EstimateMode {
    Full,
    ScopeCountsOnly,
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

pub(crate) fn collect_storage_evidence_counts(db: &Database) -> Result<StorageEvidenceCounts> {
    Ok(StorageEvidenceCounts {
        file_sightings: queries::count_file_sightings(db)?,
        file_events: queries::count_file_events(db)?,
        activity_daily_rollups: queries::count_activity_daily_rollups(db)?,
        verification_runs: queries::count_verification_runs(db)?,
        snapshot_runs: queries::count_snapshot_runs(db)?,
    })
}

#[cfg(test)]
mod tests;
