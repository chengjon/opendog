use super::*;
use rusqlite::params;

fn temp_db_path() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    (dir, db_path)
}

#[test]
fn open_creates_database_file() {
    let (dir, db_path) = temp_db_path();
    assert!(!db_path.exists());
    let _db = Database::open(&db_path).unwrap();
    assert!(db_path.exists());
    let _ = dir; // keep alive
}

#[test]
fn open_creates_parent_directories() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("a").join("b").join("test.db");
    let _db = Database::open(&nested).unwrap();
    assert!(nested.exists());
    let _ = dir;
}

#[test]
fn open_enables_wal_mode() {
    let (dir, db_path) = temp_db_path();
    let db = Database::open(&db_path).unwrap();
    let mode: String = db
        .query_row("PRAGMA journal_mode", params![], |row| row.get(0))
        .unwrap();
    assert_eq!(mode, "wal");
    let _ = dir;
}

#[test]
fn open_project_creates_snapshot_table() {
    let (dir, db_path) = temp_db_path();
    let db = Database::open_project(&db_path).unwrap();
    let count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='snapshot'",
            params![],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
    let _ = dir;
}

#[test]
fn open_registry_creates_projects_table() {
    let (dir, db_path) = temp_db_path();
    let db = Database::open_registry(&db_path).unwrap();
    let count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='projects'",
            params![],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
    let _ = dir;
}

#[test]
fn execute_inserts_and_queries_row() {
    let (dir, db_path) = temp_db_path();
    let db = Database::open(&db_path).unwrap();
    db.execute_batch("CREATE TABLE t (val TEXT)").unwrap();
    let rows = db
        .execute("INSERT INTO t (val) VALUES (?1)", params!["hello"])
        .unwrap();
    assert_eq!(rows, 1);
    let val: String = db
        .query_row("SELECT val FROM t", params![], |row| row.get(0))
        .unwrap();
    assert_eq!(val, "hello");
    let _ = dir;
}

#[test]
fn prepare_and_query_maps_multiple_rows() {
    let (dir, db_path) = temp_db_path();
    let db = Database::open(&db_path).unwrap();
    db.execute_batch("CREATE TABLE t (n INTEGER)").unwrap();
    db.execute("INSERT INTO t (n) VALUES (1)", params![])
        .unwrap();
    db.execute("INSERT INTO t (n) VALUES (2)", params![])
        .unwrap();
    db.execute("INSERT INTO t (n) VALUES (3)", params![])
        .unwrap();
    let nums: Vec<i64> = db
        .prepare_and_query("SELECT n FROM t ORDER BY n", params![], |row| row.get(0))
        .unwrap();
    assert_eq!(nums, vec![1, 2, 3]);
    let _ = dir;
}

#[test]
fn execute_batch_runs_multiple_statements() {
    let (dir, db_path) = temp_db_path();
    let db = Database::open(&db_path).unwrap();
    db.execute_batch("CREATE TABLE a (x INT); CREATE TABLE b (y INT);")
        .unwrap();
    let count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('a', 'b')",
            params![],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 2);
    let _ = dir;
}

#[test]
fn conn_returns_underlying_connection() {
    let (dir, db_path) = temp_db_path();
    let db = Database::open(&db_path).unwrap();
    // conn() should return a reference we can use
    let result: i64 = db
        .conn()
        .query_row("SELECT 1 + 2", params![], |row| row.get(0))
        .unwrap();
    assert_eq!(result, 3);
    let _ = dir;
}

#[test]
fn query_row_returns_error_for_missing_row() {
    let (dir, db_path) = temp_db_path();
    let db = Database::open(&db_path).unwrap();
    db.execute_batch("CREATE TABLE t (val TEXT)").unwrap();
    let result = db.query_row::<String, _, _>("SELECT val FROM t", params![], |row| row.get(0));
    assert!(result.is_err());
    let _ = dir;
}

#[test]
fn open_project_creates_all_expected_tables() {
    let (dir, db_path) = temp_db_path();
    let db = Database::open_project(&db_path).unwrap();
    let expected = [
        "snapshot",
        "file_stats",
        "file_sightings",
        "file_events",
        "snapshot_runs",
        "snapshot_history",
        "verification_runs",
        "activity_daily_rollups",
        "governance_lanes",
        "governance_nodes",
        "data_risk_cache",
    ];
    for table in &expected {
        let count: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                params![table],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "table '{}' should exist", table);
    }
    let _ = dir;
}
