use super::*;
use rusqlite::params;

fn test_db() -> Database {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open_project(&db_path).unwrap();
    Box::leak(Box::new(dir));
    db
}

fn count(db: &Database, table: &str) -> i64 {
    db.query_row(
        &format!("SELECT COUNT(*) FROM {}", table),
        params![],
        |row| row.get(0),
    )
    .unwrap()
}

fn seed_snapshot_runs(db: &Database, timestamps: &[&str], files_per_run: &[&[(&str, i64)]]) {
    use rusqlite::params;
    for (i, ts) in timestamps.iter().enumerate() {
        db.execute(
            "INSERT INTO snapshot_runs (captured_at, file_count) VALUES (?1, ?2)",
            params![ts, files_per_run[i].len() as i64],
        )
        .unwrap();
        let run_id: i64 = db
            .query_row("SELECT last_insert_rowid()", params![], |row| row.get(0))
            .unwrap();
        for (path, size) in files_per_run[i] {
            db.execute(
                "INSERT INTO snapshot_history (run_id, path, size, mtime, file_type) VALUES (?1, ?2, ?3, 1, 'rs')",
                params![run_id, path, size],
            )
            .unwrap();
        }
    }
}

/// Snapshots-only cleanup with keep_snapshot_runs=2 prunes older runs.
mod activity;
mod estimate;
mod scope;
mod snapshots;
mod storage;
mod verification;
