use crate::error::Result;
use crate::storage::database::Database;
use crate::storage::queries::{self, StatsEntry};
use serde::{Deserialize, Serialize};

/// Aggregate project statistics summary (STAT-05).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectSummary {
    pub total_files: i64,
    pub accessed_files: i64,
    pub unused_files: i64,
}

/// STAT-06: Query per-file statistics enriched with snapshot metadata.
pub fn get_stats(db: &Database) -> Result<Vec<StatsEntry>> {
    queries::get_stats_with_snapshot(db)
}

/// STAT-07: Query never-accessed files (unused file candidates).
pub fn get_unused_files(db: &Database) -> Result<Vec<StatsEntry>> {
    queries::get_unused_files(db)
}

/// STAT-08: Query high-frequency files (core file candidates).
/// Returns files with access_count >= `min_access_count` (default 5).
pub fn get_core_files(db: &Database, min_access_count: i64) -> Result<Vec<StatsEntry>> {
    queries::get_core_files(db, min_access_count.max(1))
}

/// Get detailed stats for a single file.
pub fn get_file_detail(db: &Database, file_path: &str) -> Result<Option<StatsEntry>> {
    queries::get_file_detail(db, file_path)
}

/// STAT-05: Get aggregate summary — total/accessed/unused file counts.
pub fn get_summary(db: &Database) -> Result<ProjectSummary> {
    let total = queries::count_snapshot(db)?;
    let accessed = queries::count_accessed(db)?;
    let unused = queries::count_unused(db)?;
    Ok(ProjectSummary {
        total_files: total,
        accessed_files: accessed,
        unused_files: unused,
    })
}

#[cfg(test)]
mod tests;
