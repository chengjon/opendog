use crate::config::ProjectInfo;
use crate::error::{OpenDogError, Result};
use crate::storage::database::Database;
use rusqlite::params;
use std::path::Path;

// --- Registry queries ---

pub fn insert_project(db: &Database, info: &ProjectInfo) -> Result<()> {
    let config_json = serde_json::to_string(&info.config)?;
    db.execute(
        "INSERT INTO projects (id, root_path, db_path, config, created_at, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![info.id, info.root_path.to_str(), info.db_path.to_str(), config_json, info.created_at, info.status],
    )?;
    Ok(())
}

pub fn get_project(db: &Database, id: &str) -> Result<Option<ProjectInfo>> {
    let result = db.query_row(
        "SELECT id, root_path, db_path, config, created_at, status FROM projects WHERE id = ?1",
        params![id],
        |row| {
            let config_str: String = row.get(3)?;
            let config = serde_json::from_str(&config_str).unwrap_or_default();
            Ok(ProjectInfo {
                id: row.get(0)?,
                root_path: Path::new(&row.get::<_, String>(1)?).to_path_buf(),
                db_path: Path::new(&row.get::<_, String>(2)?).to_path_buf(),
                config,
                created_at: row.get(4)?,
                status: row.get(5)?,
            })
        },
    );
    match result {
        Ok(info) => Ok(Some(info)),
        Err(OpenDogError::Database(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn list_projects(db: &Database) -> Result<Vec<ProjectInfo>> {
    db.prepare_and_query(
        "SELECT id, root_path, db_path, config, created_at, status FROM projects WHERE status = 'active' ORDER BY created_at",
        params![],
        |row| {
            let config_str: String = row.get(3)?;
            let config = serde_json::from_str(&config_str).unwrap_or_default();
            Ok(ProjectInfo {
                id: row.get(0)?,
                root_path: Path::new(&row.get::<_, String>(1)?).to_path_buf(),
                db_path: Path::new(&row.get::<_, String>(2)?).to_path_buf(),
                config,
                created_at: row.get(4)?,
                status: row.get(5)?,
            })
        },
    )
}

pub fn delete_project(db: &Database, id: &str) -> Result<bool> {
    let rows = db.execute(
        "UPDATE projects SET status = 'deleted' WHERE id = ?1 AND status = 'active'",
        params![id],
    )?;
    Ok(rows > 0)
}

// --- Snapshot queries ---

#[derive(Debug, Clone)]
pub struct SnapshotEntry {
    pub path: String,
    pub size: i64,
    pub mtime: i64,
    pub file_type: String,
    pub scan_timestamp: String,
}

pub fn insert_snapshot_batch(db: &Database, entries: &[SnapshotEntry]) -> Result<usize> {
    let tx = db.conn().unchecked_transaction()?;
    let mut count = 0usize;
    for entry in entries {
        tx.execute(
            "INSERT OR REPLACE INTO snapshot (path, size, mtime, file_type, scan_timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![entry.path, entry.size, entry.mtime, entry.file_type, entry.scan_timestamp],
        )?;
        count += 1;
    }
    tx.commit()?;
    Ok(count)
}

pub fn delete_stale_snapshot(db: &Database, existing_paths: &[String], scan_timestamp: &str) -> Result<usize> {
    if existing_paths.is_empty() {
        return Ok(0);
    }

    let placeholders: Vec<String> = (1..=existing_paths.len()).map(|i| format!("?{}", i)).collect();
    let sql = format!(
        "DELETE FROM snapshot WHERE scan_timestamp < ?{} AND path NOT IN ({})",
        existing_paths.len() + 1,
        placeholders.join(",")
    );

    let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = existing_paths
        .iter()
        .map(|p| Box::new(p.clone()) as Box<dyn rusqlite::types::ToSql>)
        .collect();
    params_vec.push(Box::new(scan_timestamp.to_string()));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let rows = db.conn().execute(&sql, param_refs.as_slice())?;
    Ok(rows)
}

pub fn count_snapshot(db: &Database) -> Result<i64> {
    let count = db.query_row("SELECT COUNT(*) FROM snapshot", params![], |row| row.get(0))?;
    Ok(count)
}

pub fn get_snapshot_paths(db: &Database) -> Result<Vec<String>> {
    db.prepare_and_query("SELECT path FROM snapshot", params![], |row| row.get(0))
}

pub fn clear_snapshot(db: &Database) -> Result<usize> {
    db.execute("DELETE FROM snapshot", params![])
}

// --- File stats queries (defined for Phase 3, written now for schema completeness) ---

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

// --- Statistics queries (Phase 3: STAT-05..08) ---

/// Enriched file stats joined with snapshot metadata.
#[derive(Debug, Clone)]
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

/// STAT-06: All file statistics enriched with snapshot metadata (size, type).
/// Returns files from snapshot LEFT JOINed with file_stats, ordered by access_count DESC.
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

/// STAT-07: Never-accessed files (unused file candidates).
/// Files in snapshot that have no file_stats entry or zero access_count.
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

/// STAT-08: High-frequency files (core file candidates).
/// Files with access_count >= min_access_count, ordered by access_count DESC.
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

/// Get statistics for a single file (used by detail queries).
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

/// Count of files in snapshot that have never been accessed (STAT-05 support).
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

/// Count of accessed files (STAT-05 support).
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
