use crate::error::{OpenDogError, Result};
use crate::storage::database::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SnapshotEntry {
    pub path: String,
    pub size: i64,
    pub mtime: i64,
    pub file_type: String,
    pub scan_timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotRunRecord {
    pub id: i64,
    pub captured_at: String,
    pub file_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoricalSnapshotEntry {
    pub path: String,
    pub size: i64,
    pub mtime: i64,
    pub file_type: String,
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

pub fn insert_snapshot_history(
    db: &Database,
    captured_at: &str,
    entries: &[SnapshotEntry],
) -> Result<SnapshotRunRecord> {
    let tx = db.conn().unchecked_transaction()?;
    tx.execute(
        "INSERT INTO snapshot_runs (captured_at, file_count) VALUES (?1, ?2)",
        params![captured_at, entries.len() as i64],
    )?;
    let run_id = tx.last_insert_rowid();
    for entry in entries {
        tx.execute(
            "INSERT INTO snapshot_history (run_id, path, size, mtime, file_type) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![run_id, entry.path, entry.size, entry.mtime, entry.file_type],
        )?;
    }
    tx.commit()?;
    Ok(SnapshotRunRecord {
        id: run_id,
        captured_at: captured_at.to_string(),
        file_count: entries.len() as i64,
    })
}

pub fn list_snapshot_runs(db: &Database, limit: usize) -> Result<Vec<SnapshotRunRecord>> {
    db.prepare_and_query(
        "SELECT id, captured_at, file_count
         FROM snapshot_runs
         ORDER BY CAST(captured_at AS INTEGER) DESC, id DESC
         LIMIT ?1",
        params![limit.max(1) as i64],
        |row| {
            Ok(SnapshotRunRecord {
                id: row.get(0)?,
                captured_at: row.get(1)?,
                file_count: row.get(2)?,
            })
        },
    )
}

pub fn get_snapshot_run(db: &Database, run_id: i64) -> Result<Option<SnapshotRunRecord>> {
    let result = db.query_row(
        "SELECT id, captured_at, file_count FROM snapshot_runs WHERE id = ?1",
        params![run_id],
        |row| {
            Ok(SnapshotRunRecord {
                id: row.get(0)?,
                captured_at: row.get(1)?,
                file_count: row.get(2)?,
            })
        },
    );
    match result {
        Ok(record) => Ok(Some(record)),
        Err(OpenDogError::Database(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn get_snapshot_history_entries(
    db: &Database,
    run_id: i64,
) -> Result<Vec<HistoricalSnapshotEntry>> {
    db.prepare_and_query(
        "SELECT path, size, mtime, file_type
         FROM snapshot_history
         WHERE run_id = ?1
         ORDER BY path",
        params![run_id],
        |row| {
            Ok(HistoricalSnapshotEntry {
                path: row.get(0)?,
                size: row.get(1)?,
                mtime: row.get(2)?,
                file_type: row.get(3)?,
            })
        },
    )
}

pub fn delete_stale_snapshot(
    db: &Database,
    existing_paths: &[String],
    scan_timestamp: &str,
) -> Result<usize> {
    if existing_paths.is_empty() {
        return Ok(0);
    }

    let placeholders: Vec<String> = (1..=existing_paths.len())
        .map(|i| format!("?{}", i))
        .collect();
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

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params_vec.iter().map(|p| p.as_ref()).collect();

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
