use crate::error::{OpenDogError, Result};
use crate::storage::database::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct FileStats {
    pub file_path: String,
    pub access_count: i64,
    pub estimated_duration_ms: i64,
    pub modification_count: i64,
    pub last_access_time: Option<String>,
    pub first_seen_time: String,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatsEntry {
    pub file_path: String,
    pub size: i64,
    pub file_type: String,
    pub access_count: i64,
    pub estimated_duration_ms: i64,
    pub modification_count: i64,
    pub last_access_time: Option<String>,
    pub first_seen_time: Option<String>,
}

pub fn get_all_stats(db: &Database) -> Result<Vec<FileStats>> {
    db.prepare_and_query(
        "SELECT file_path, access_count, estimated_duration_ms, modification_count, last_access_time, first_seen_time, last_updated FROM file_stats ORDER BY access_count DESC",
        params![],
        |row| {
            Ok(FileStats {
                file_path: row.get(0)?,
                access_count: row.get(1)?,
                estimated_duration_ms: row.get(2)?,
                modification_count: row.get(3)?,
                last_access_time: row.get(4)?,
                first_seen_time: row.get(5)?,
                last_updated: row.get(6)?,
            })
        },
    )
}

pub fn get_stats_with_snapshot(db: &Database) -> Result<Vec<StatsEntry>> {
    db.prepare_and_query(
        "SELECT s.path, s.size, s.file_type,
                COALESCE(fs.access_count, 0),
                COALESCE(fs.estimated_duration_ms, 0),
                COALESCE(fs.modification_count, 0),
                fs.last_access_time,
                fs.first_seen_time
         FROM snapshot s
         LEFT JOIN file_stats fs ON s.path = fs.file_path
         ORDER BY COALESCE(fs.access_count, 0) DESC, s.path",
        params![],
        |row| {
            Ok(StatsEntry {
                file_path: row.get(0)?,
                size: row.get(1)?,
                file_type: row.get(2)?,
                access_count: row.get(3)?,
                estimated_duration_ms: row.get(4)?,
                modification_count: row.get(5)?,
                last_access_time: row.get(6)?,
                first_seen_time: row.get(7)?,
            })
        },
    )
}

pub fn get_unused_files(db: &Database) -> Result<Vec<StatsEntry>> {
    db.prepare_and_query(
        "SELECT s.path, s.size, s.file_type,
                COALESCE(fs.access_count, 0),
                COALESCE(fs.estimated_duration_ms, 0),
                COALESCE(fs.modification_count, 0),
                fs.last_access_time,
                fs.first_seen_time
         FROM snapshot s
         LEFT JOIN file_stats fs ON s.path = fs.file_path
         WHERE fs.file_path IS NULL OR fs.access_count = 0
         ORDER BY s.path",
        params![],
        |row| {
            Ok(StatsEntry {
                file_path: row.get(0)?,
                size: row.get(1)?,
                file_type: row.get(2)?,
                access_count: row.get(3)?,
                estimated_duration_ms: row.get(4)?,
                modification_count: row.get(5)?,
                last_access_time: row.get(6)?,
                first_seen_time: row.get(7)?,
            })
        },
    )
}

pub fn get_core_files(db: &Database, min_access_count: i64) -> Result<Vec<StatsEntry>> {
    db.prepare_and_query(
        "SELECT s.path, s.size, s.file_type,
                fs.access_count,
                fs.estimated_duration_ms,
                fs.modification_count,
                fs.last_access_time,
                fs.first_seen_time
         FROM file_stats fs
         JOIN snapshot s ON fs.file_path = s.path
         WHERE fs.access_count >= ?1
         ORDER BY fs.access_count DESC",
        params![min_access_count],
        |row| {
            Ok(StatsEntry {
                file_path: row.get(0)?,
                size: row.get(1)?,
                file_type: row.get(2)?,
                access_count: row.get(3)?,
                estimated_duration_ms: row.get(4)?,
                modification_count: row.get(5)?,
                last_access_time: row.get(6)?,
                first_seen_time: row.get(7)?,
            })
        },
    )
}

pub fn get_file_detail(db: &Database, file_path: &str) -> Result<Option<StatsEntry>> {
    let result = db.query_row(
        "SELECT s.path, s.size, s.file_type,
                COALESCE(fs.access_count, 0),
                COALESCE(fs.estimated_duration_ms, 0),
                COALESCE(fs.modification_count, 0),
                fs.last_access_time,
                fs.first_seen_time
         FROM snapshot s
         LEFT JOIN file_stats fs ON s.path = fs.file_path
         WHERE s.path = ?1",
        params![file_path],
        |row| {
            Ok(StatsEntry {
                file_path: row.get(0)?,
                size: row.get(1)?,
                file_type: row.get(2)?,
                access_count: row.get(3)?,
                estimated_duration_ms: row.get(4)?,
                modification_count: row.get(5)?,
                last_access_time: row.get(6)?,
                first_seen_time: row.get(7)?,
            })
        },
    );
    match result {
        Ok(entry) => Ok(Some(entry)),
        Err(OpenDogError::Database(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn count_unused(db: &Database) -> Result<i64> {
    let count = db.query_row(
        "SELECT COUNT(*) FROM snapshot s
         LEFT JOIN file_stats fs ON s.path = fs.file_path
         WHERE fs.file_path IS NULL OR fs.access_count = 0",
        params![],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn count_accessed(db: &Database) -> Result<i64> {
    let count = db.query_row(
        "SELECT COUNT(*) FROM snapshot s
         JOIN file_stats fs ON s.path = fs.file_path
         WHERE fs.access_count > 0",
        params![],
        |row| row.get(0),
    )?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;
    use rusqlite::params;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("stats_test.db");
        let db = Database::open_project(&db_path).unwrap();
        Box::leak(Box::new(dir));
        db
    }

    fn seed_snapshot(db: &Database, path: &str, size: i64, file_type: &str) {
        db.execute(
            "INSERT INTO snapshot (path, size, mtime, file_type, scan_timestamp) VALUES (?1, ?2, 0, ?3, '0')",
            params![path, size, file_type],
        )
        .unwrap();
    }

    fn seed_stats(db: &Database, path: &str, access_count: i64, mod_count: i64) {
        db.execute(
            "INSERT INTO file_stats (file_path, access_count, estimated_duration_ms, modification_count, first_seen_time, last_updated)
             VALUES (?1, ?2, 100, ?3, '100', '200')",
            params![path, access_count, mod_count],
        )
        .unwrap();
    }

    #[test]
    fn get_all_stats_returns_rows() {
        let db = test_db();
        seed_stats(&db, "a.rs", 5, 1);
        seed_stats(&db, "b.rs", 0, 0);
        let all = get_all_stats(&db).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].file_path, "a.rs"); // ordered by access_count DESC
    }

    #[test]
    fn get_stats_with_snapshot_joins_correctly() {
        let db = test_db();
        seed_snapshot(&db, "a.rs", 100, "rs");
        seed_snapshot(&db, "b.rs", 200, "rs");
        seed_stats(&db, "a.rs", 10, 2);
        let entries = get_stats_with_snapshot(&db).unwrap();
        assert_eq!(entries.len(), 2);
        let a = entries.iter().find(|e| e.file_path == "a.rs").unwrap();
        assert_eq!(a.access_count, 10);
        assert_eq!(a.modification_count, 2);
        let b = entries.iter().find(|e| e.file_path == "b.rs").unwrap();
        assert_eq!(b.access_count, 0);
    }

    #[test]
    fn get_unused_files_returns_zero_access_or_missing_stats() {
        let db = test_db();
        seed_snapshot(&db, "used.rs", 10, "rs");
        seed_snapshot(&db, "unused.rs", 20, "rs");
        seed_snapshot(&db, "nostats.rs", 30, "rs");
        seed_stats(&db, "used.rs", 5, 0);
        seed_stats(&db, "unused.rs", 0, 0);
        let unused = get_unused_files(&db).unwrap();
        assert_eq!(unused.len(), 2);
        let paths: Vec<&str> = unused.iter().map(|e| e.file_path.as_str()).collect();
        assert!(paths.contains(&"unused.rs"));
        assert!(paths.contains(&"nostats.rs"));
    }

    #[test]
    fn get_core_files_filters_by_min_access() {
        let db = test_db();
        seed_snapshot(&db, "hot.rs", 10, "rs");
        seed_snapshot(&db, "warm.rs", 20, "rs");
        seed_stats(&db, "hot.rs", 10, 0);
        seed_stats(&db, "warm.rs", 2, 0);
        let core = get_core_files(&db, 5).unwrap();
        assert_eq!(core.len(), 1);
        assert_eq!(core[0].file_path, "hot.rs");
    }

    #[test]
    fn get_file_detail_found_and_missing() {
        let db = test_db();
        seed_snapshot(&db, "exists.rs", 42, "rs");
        seed_stats(&db, "exists.rs", 3, 1);
        let found = get_file_detail(&db, "exists.rs").unwrap().unwrap();
        assert_eq!(found.size, 42);
        assert_eq!(found.access_count, 3);
        assert!(get_file_detail(&db, "nope.rs").unwrap().is_none());
    }

    #[test]
    fn count_unused_and_accessed() {
        let db = test_db();
        seed_snapshot(&db, "a.rs", 10, "rs");
        seed_snapshot(&db, "b.rs", 20, "rs");
        seed_snapshot(&db, "c.rs", 30, "rs");
        seed_stats(&db, "a.rs", 5, 0);
        seed_stats(&db, "b.rs", 0, 0);
        // c.rs has no stats
        assert_eq!(count_unused(&db).unwrap(), 2);
        assert_eq!(count_accessed(&db).unwrap(), 1);
    }
}
