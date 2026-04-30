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
