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
mod tests {
    use super::*;
    use crate::storage::queries::SnapshotEntry;
    use rusqlite::params;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open_project(&db_path).unwrap();
        // Keep tempdir alive for the test duration
        Box::leak(Box::new(dir));
        db
    }

    fn insert_snapshot(db: &Database, paths: &[&str]) {
        let entries: Vec<SnapshotEntry> = paths
            .iter()
            .map(|&p| SnapshotEntry {
                path: p.to_string(),
                size: 100,
                mtime: 0,
                file_type: "rs".to_string(),
                scan_timestamp: "1000".to_string(),
            })
            .collect();
        queries::insert_snapshot_batch(db, &entries).unwrap();
    }

    fn insert_file_stat(
        db: &Database,
        path: &str,
        access_count: i64,
        duration_ms: i64,
        mod_count: i64,
    ) {
        db.execute(
            "INSERT INTO file_stats (file_path, access_count, estimated_duration_ms, modification_count, first_seen_time, last_updated)
             VALUES (?1, ?2, ?3, ?4, '1000', '2000')",
            params![path, access_count, duration_ms, mod_count],
        ).unwrap();
    }

    #[test]
    fn test_get_stats_empty() {
        let db = test_db();
        let stats = get_stats(&db).unwrap();
        assert!(stats.is_empty());
    }

    #[test]
    fn test_get_stats_with_data() {
        let db = test_db();
        insert_snapshot(&db, &["src/main.rs", "src/lib.rs", "README.md"]);
        insert_file_stat(&db, "src/main.rs", 10, 5000, 3);
        insert_file_stat(&db, "src/lib.rs", 2, 1000, 0);

        let stats = get_stats(&db).unwrap();
        assert_eq!(stats.len(), 3);

        // Ordered by access_count DESC
        assert_eq!(stats[0].file_path, "src/main.rs");
        assert_eq!(stats[0].access_count, 10);
        assert_eq!(stats[0].estimated_duration_ms, 5000);

        assert_eq!(stats[1].file_path, "src/lib.rs");
        assert_eq!(stats[1].access_count, 2);

        // README.md has no stats → access_count 0
        assert_eq!(stats[2].file_path, "README.md");
        assert_eq!(stats[2].access_count, 0);
    }

    #[test]
    fn test_get_unused_files() {
        let db = test_db();
        insert_snapshot(&db, &["used.rs", "unused.rs", "also_unused.rs"]);
        insert_file_stat(&db, "used.rs", 5, 1000, 0);

        let unused = get_unused_files(&db).unwrap();
        assert_eq!(unused.len(), 2);
        let paths: Vec<&str> = unused.iter().map(|e| e.file_path.as_str()).collect();
        assert!(paths.contains(&"unused.rs"));
        assert!(paths.contains(&"also_unused.rs"));
    }

    #[test]
    fn test_get_core_files() {
        let db = test_db();
        insert_snapshot(&db, &["core.rs", "moderate.rs", "rare.rs"]);
        insert_file_stat(&db, "core.rs", 50, 10000, 10);
        insert_file_stat(&db, "moderate.rs", 5, 500, 1);
        insert_file_stat(&db, "rare.rs", 1, 100, 0);

        let core = get_core_files(&db, 5).unwrap();
        assert_eq!(core.len(), 2);
        assert_eq!(core[0].file_path, "core.rs");
        assert_eq!(core[0].access_count, 50);
        assert_eq!(core[1].file_path, "moderate.rs");
    }

    #[test]
    fn test_get_file_detail() {
        let db = test_db();
        insert_snapshot(&db, &["src/main.rs"]);
        insert_file_stat(&db, "src/main.rs", 7, 3000, 2);

        let detail = get_file_detail(&db, "src/main.rs").unwrap().unwrap();
        assert_eq!(detail.access_count, 7);
        assert_eq!(detail.estimated_duration_ms, 3000);
        assert_eq!(detail.modification_count, 2);
        assert_eq!(detail.size, 100);
    }

    #[test]
    fn test_get_file_detail_missing() {
        let db = test_db();
        assert!(get_file_detail(&db, "nonexistent.rs").unwrap().is_none());
    }

    #[test]
    fn test_get_summary() {
        let db = test_db();
        insert_snapshot(&db, &["a.rs", "b.rs", "c.rs", "d.rs"]);
        insert_file_stat(&db, "a.rs", 10, 1000, 0);
        insert_file_stat(&db, "b.rs", 1, 100, 0);

        let summary = get_summary(&db).unwrap();
        assert_eq!(summary.total_files, 4);
        assert_eq!(summary.accessed_files, 2);
        assert_eq!(summary.unused_files, 2);
    }

    #[test]
    fn test_get_summary_empty() {
        let db = test_db();
        let summary = get_summary(&db).unwrap();
        assert_eq!(summary.total_files, 0);
        assert_eq!(summary.accessed_files, 0);
        assert_eq!(summary.unused_files, 0);
    }
}
