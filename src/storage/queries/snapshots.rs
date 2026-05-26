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

pub fn count_snapshot(db: &Database) -> Result<i64> {
    let count = db.query_row("SELECT COUNT(*) FROM snapshot", params![], |row| row.get(0))?;
    Ok(count)
}

pub fn get_snapshot_paths(db: &Database) -> Result<Vec<String>> {
    db.prepare_and_query("SELECT path FROM snapshot", params![], |row| row.get(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("snapshots_test.db");
        let db = Database::open_project(&db_path).unwrap();
        Box::leak(Box::new(dir));
        db
    }

    fn entry(path: &str, size: i64) -> SnapshotEntry {
        SnapshotEntry {
            path: path.to_string(),
            size,
            mtime: 1,
            file_type: "rs".to_string(),
            scan_timestamp: "1000".to_string(),
        }
    }

    #[test]
    fn insert_batch_and_query() {
        let db = test_db();
        let count = insert_snapshot_batch(&db, &[entry("a.rs", 10), entry("b.rs", 20)]).unwrap();
        assert_eq!(count, 2);
        assert_eq!(count_snapshot(&db).unwrap(), 2);
        let paths = get_snapshot_paths(&db).unwrap();
        assert_eq!(paths, vec!["a.rs", "b.rs"]);
    }

    #[test]
    fn insert_batch_replaces_existing() {
        let db = test_db();
        insert_snapshot_batch(&db, &[entry("a.rs", 10)]).unwrap();
        insert_snapshot_batch(&db, &[entry("a.rs", 99)]).unwrap();
        assert_eq!(count_snapshot(&db).unwrap(), 1);
    }

    #[test]
    fn insert_history_and_query_runs() {
        let db = test_db();
        let run1 = insert_snapshot_history(&db, "100", &[entry("x.rs", 5)]).unwrap();
        let run2 =
            insert_snapshot_history(&db, "200", &[entry("y.rs", 6), entry("z.rs", 7)]).unwrap();
        assert_ne!(run1.id, run2.id);
        assert_eq!(run1.file_count, 1);
        assert_eq!(run2.file_count, 2);

        // List runs ordered by captured_at DESC
        let runs = list_snapshot_runs(&db, 10).unwrap();
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].id, run2.id);
        assert_eq!(runs[1].id, run1.id);
    }

    #[test]
    fn get_snapshot_run_found_and_missing() {
        let db = test_db();
        let run = insert_snapshot_history(&db, "300", &[entry("w.rs", 1)]).unwrap();
        let found = get_snapshot_run(&db, run.id).unwrap().unwrap();
        assert_eq!(found.captured_at, "300");
        assert!(get_snapshot_run(&db, 99999).unwrap().is_none());
    }

    #[test]
    fn get_history_entries_for_run() {
        let db = test_db();
        let run =
            insert_snapshot_history(&db, "400", &[entry("p.rs", 1), entry("q.rs", 2)]).unwrap();
        let entries = get_snapshot_history_entries(&db, run.id).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, "p.rs"); // ORDER BY path
        assert_eq!(entries[1].path, "q.rs");
    }
}
