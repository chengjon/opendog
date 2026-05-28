use opendog::storage::database::Database;
use opendog::storage::queries;
use tempfile::TempDir;

use super::common::{ensure_dir, setup_manager};

#[path = "storage_project_snapshot/project_config.rs"]
mod project_config;
#[path = "storage_project_snapshot/snapshot_cases.rs"]
mod snapshot_cases;

#[test]
fn test_registry_creates_schema() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("registry.db");
    let db = Database::open_registry(&path).unwrap();

    db.execute(
        "INSERT INTO projects (id, root_path, db_path, config, created_at, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params!["test-project", "/tmp/test", "/tmp/test.db", "{}", "12345", "active"],
    ).unwrap();

    let count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM projects",
            rusqlite::params![],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_project_db_creates_tables() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("project.db");
    let db = Database::open_project(&path).unwrap();

    db.execute(
        "INSERT INTO snapshot (path, size, mtime, file_type, scan_timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["test.rs", 100i64, 12345i64, "rs", "99999"],
    ).unwrap();

    let count = queries::count_snapshot(&db).unwrap();
    assert_eq!(count, 1);

    let mut indexes = db
        .prepare_and_query(
            "SELECT name FROM sqlite_master WHERE type = 'index' AND name LIKE 'idx_%' ORDER BY name",
            rusqlite::params![],
            |row| row.get::<_, String>(0),
        )
        .unwrap();
    indexes.sort();

    for expected in [
        "idx_snapshot_runs_time_int",
        "idx_file_sightings_time_int",
        "idx_file_events_modify_time_int",
        "idx_verification_runs_finished_time_int",
    ] {
        assert!(
            indexes.iter().any(|name| name == expected),
            "missing expected index {expected}; got {indexes:?}"
        );
    }
}

#[test]
fn test_wal_mode_enabled() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.db");
    let _db = Database::open(&path).unwrap();

    let db2 = Database::open(&path).unwrap();
    let mode: String = db2
        .query_row("PRAGMA journal_mode", rusqlite::params![], |row| row.get(0))
        .unwrap();
    assert_eq!(mode, "wal");
}

#[test]
fn test_create_project() {
    let (dir, mgr) = setup_manager();
    let project_dir = dir.path().join("myproject");
    ensure_dir(&project_dir);

    let info = mgr.create("test-proj", &project_dir).unwrap();
    assert_eq!(info.id, "test-proj");
    assert_eq!(info.root_path, project_dir);
    assert_eq!(info.status, "active");
    assert!(info.db_path.to_str().unwrap().ends_with("test-proj.db"));
}

#[test]
fn test_create_duplicate_project_fails() {
    let (dir, mgr) = setup_manager();
    let project_dir = dir.path().join("myproject");
    ensure_dir(&project_dir);

    mgr.create("dup", &project_dir).unwrap();
    let result = mgr.create("dup", &project_dir);
    assert!(result.is_err());
}

#[test]
fn test_invalid_project_id_rejected() {
    let (dir, mgr) = setup_manager();
    let project_dir = dir.path().join("myproject");
    ensure_dir(&project_dir);

    assert!(mgr.create("", &project_dir).is_err());
    assert!(mgr.create("has space", &project_dir).is_err());
    assert!(mgr.create("has/slash", &project_dir).is_err());
}

#[test]
fn test_list_projects() {
    let (dir, mgr) = setup_manager();
    let p1 = dir.path().join("proj1");
    let p2 = dir.path().join("proj2");
    ensure_dir(&p1);
    ensure_dir(&p2);

    mgr.create("proj-1", &p1).unwrap();
    mgr.create("proj-2", &p2).unwrap();

    let list = mgr.list().unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn test_delete_project() {
    let (dir, mgr) = setup_manager();
    let project_dir = dir.path().join("myproject");
    ensure_dir(&project_dir);

    mgr.create("to-delete", &project_dir).unwrap();
    assert!(mgr.get("to-delete").unwrap().is_some());

    let deleted = mgr.delete("to-delete").unwrap();
    assert!(deleted);

    let list = mgr.list().unwrap();
    assert!(list.is_empty());
}

#[test]
fn test_delete_nonexistent_project() {
    let (_dir, mgr) = setup_manager();
    let deleted = mgr.delete("no-such").unwrap();
    assert!(!deleted);
}

#[test]
fn test_open_project_db() {
    let (dir, mgr) = setup_manager();
    let project_dir = dir.path().join("myproject");
    ensure_dir(&project_dir);

    mgr.create("db-test", &project_dir).unwrap();
    let db = mgr.open_project_db("db-test").unwrap();

    db.execute(
        "INSERT INTO snapshot (path, size, mtime, file_type, scan_timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["foo.rs", 10i64, 0i64, "rs", "0"],
    ).unwrap();
}
