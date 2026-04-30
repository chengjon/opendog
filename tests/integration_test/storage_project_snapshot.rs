use opendog::config::{ConfigPatch, ProjectConfig, ProjectConfigPatch};
use opendog::core::snapshot;
use opendog::storage::database::Database;
use opendog::storage::queries;
use std::fs;
use tempfile::TempDir;

use super::common::{ensure_dir, setup_manager, setup_snapshot};

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
fn test_effective_project_config_uses_global_defaults_and_project_overrides() {
    let (dir, mgr) = setup_manager();
    let project_dir = dir.path().join("myproject");
    ensure_dir(&project_dir);

    mgr.update_global_config(ConfigPatch {
        ignore_patterns: Some(vec!["global-cache".to_string(), "dist".to_string()]),
        process_whitelist: Some(vec!["codex".to_string(), "claude".to_string()]),
    })
    .unwrap();

    mgr.create("cfg-test", &project_dir).unwrap();

    let baseline = mgr.effective_project_config("cfg-test").unwrap();
    assert_eq!(
        baseline.ignore_patterns,
        vec!["global-cache".to_string(), "dist".to_string()]
    );
    assert_eq!(
        baseline.process_whitelist,
        vec!["codex".to_string(), "claude".to_string()]
    );

    let updated = mgr
        .update_project_config(
            "cfg-test",
            ProjectConfigPatch {
                ignore_patterns: Some(vec!["logs".to_string(), "tmp".to_string()]),
                process_whitelist: None,
                inherit_ignore_patterns: false,
                inherit_process_whitelist: false,
            },
        )
        .unwrap();
    assert_eq!(
        updated.project_overrides.ignore_patterns,
        Some(vec!["logs".to_string(), "tmp".to_string()])
    );
    assert_eq!(
        updated.effective.ignore_patterns,
        vec!["logs".to_string(), "tmp".to_string()]
    );
    assert_eq!(
        updated.effective.process_whitelist,
        vec!["codex".to_string(), "claude".to_string()]
    );

    let inherited = mgr
        .update_project_config(
            "cfg-test",
            ProjectConfigPatch {
                ignore_patterns: None,
                process_whitelist: None,
                inherit_ignore_patterns: true,
                inherit_process_whitelist: false,
            },
        )
        .unwrap();
    assert_eq!(inherited.project_overrides.ignore_patterns, None);
    assert_eq!(
        inherited.effective.ignore_patterns,
        vec!["global-cache".to_string(), "dist".to_string()]
    );
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

#[test]
fn test_snapshot_basic_scan() {
    let (dir, db) = setup_snapshot();
    let project_dir = dir.path().join("project");
    ensure_dir(&project_dir.join("src"));
    fs::write(project_dir.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(project_dir.join("README.md"), "# Test").unwrap();

    let result = snapshot::take_snapshot(&db, &project_dir, &ProjectConfig::default()).unwrap();
    assert_eq!(result.total_files, 2);
    assert!(result.total_files >= result.new_files);
}

#[test]
fn test_snapshot_filters_ignored_dirs() {
    let (dir, db) = setup_snapshot();
    let project_dir = dir.path().join("project");
    ensure_dir(&project_dir.join("src"));
    ensure_dir(&project_dir.join("node_modules/pkg"));
    ensure_dir(&project_dir.join(".git/objects"));
    ensure_dir(&project_dir.join("dist"));

    fs::write(project_dir.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(
        project_dir.join("node_modules/pkg/index.js"),
        "module.exports",
    )
    .unwrap();
    fs::write(project_dir.join(".git/objects/abc"), "git object").unwrap();
    fs::write(project_dir.join("dist/bundle.js"), "bundled").unwrap();

    let result = snapshot::take_snapshot(&db, &project_dir, &ProjectConfig::default()).unwrap();
    assert_eq!(result.total_files, 1);
}

#[test]
fn test_snapshot_handles_permission_errors() {
    let (dir, db) = setup_snapshot();
    let project_dir = dir.path().join("project");
    let restricted = project_dir.join("restricted");
    ensure_dir(&restricted);
    fs::write(project_dir.join("readable.txt"), "ok").unwrap();
    fs::write(restricted.join("secret.txt"), "nope").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&restricted).unwrap().permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&restricted, perms).unwrap();
    }

    let result = snapshot::take_snapshot(&db, &project_dir, &ProjectConfig::default()).unwrap();
    assert!(result.total_files >= 1);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&restricted).unwrap().permissions();
        perms.set_mode(0o755);
        let _ = fs::set_permissions(&restricted, perms);
    }
}

#[test]
fn test_snapshot_incremental_update() {
    let (dir, db) = setup_snapshot();
    let project_dir = dir.path().join("project");
    ensure_dir(&project_dir);
    fs::write(project_dir.join("file1.txt"), "first").unwrap();

    let r1 = snapshot::take_snapshot(&db, &project_dir, &ProjectConfig::default()).unwrap();
    assert_eq!(r1.total_files, 1);

    fs::write(project_dir.join("file2.txt"), "second").unwrap();

    let r2 = snapshot::take_snapshot(&db, &project_dir, &ProjectConfig::default()).unwrap();
    assert_eq!(r2.total_files, 2);
    assert_eq!(r2.removed_files, 0);

    fs::remove_file(project_dir.join("file1.txt")).unwrap();

    let r3 = snapshot::take_snapshot(&db, &project_dir, &ProjectConfig::default()).unwrap();
    assert_eq!(r3.total_files, 1);
    assert_eq!(r3.removed_files, 1);
}

#[test]
fn test_snapshot_records_metadata() {
    let (dir, db) = setup_snapshot();
    let project_dir = dir.path().join("project");
    ensure_dir(&project_dir);
    fs::write(
        project_dir.join("code.rs"),
        "fn main() { println!(\"hi\"); }",
    )
    .unwrap();

    snapshot::take_snapshot(&db, &project_dir, &ProjectConfig::default()).unwrap();

    let paths = queries::get_snapshot_paths(&db).unwrap();
    assert!(paths.contains(&"code.rs".to_string()));

    let entries = db
        .prepare_and_query(
            "SELECT path, size, file_type FROM snapshot WHERE path = 'code.rs'",
            rusqlite::params![],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, "code.rs");
    assert!(entries[0].1 > 0);
    assert_eq!(entries[0].2, "rs");
}

#[test]
fn test_snapshot_file_type_filter() {
    let (dir, db) = setup_snapshot();
    let project_dir = dir.path().join("project");
    ensure_dir(&project_dir.join("__pycache__"));
    ensure_dir(&project_dir.join("src"));

    fs::write(project_dir.join("main.py"), "print('hi')").unwrap();
    fs::write(
        project_dir.join("__pycache__/main.cpython-311.pyc"),
        "bytecode",
    )
    .unwrap();
    fs::write(project_dir.join("src/app.py"), "app").unwrap();

    let result = snapshot::take_snapshot(&db, &project_dir, &ProjectConfig::default()).unwrap();
    assert_eq!(result.total_files, 2);
}

#[test]
fn test_project_isolation() {
    let (dir, mgr) = setup_manager();

    let p1 = dir.path().join("proj1");
    let p2 = dir.path().join("proj2");
    ensure_dir(&p1);
    ensure_dir(&p2);

    fs::write(p1.join("a.txt"), "project1").unwrap();
    fs::write(p2.join("b.txt"), "project2").unwrap();

    mgr.create("iso-1", &p1).unwrap();
    mgr.create("iso-2", &p2).unwrap();

    let db1 = mgr.open_project_db("iso-1").unwrap();
    let db2 = mgr.open_project_db("iso-2").unwrap();

    snapshot::take_snapshot(&db1, &p1, &ProjectConfig::default()).unwrap();
    snapshot::take_snapshot(&db2, &p2, &ProjectConfig::default()).unwrap();

    assert_eq!(queries::count_snapshot(&db1).unwrap(), 1);
    assert_eq!(queries::count_snapshot(&db2).unwrap(), 1);

    let paths1 = queries::get_snapshot_paths(&db1).unwrap();
    let paths2 = queries::get_snapshot_paths(&db2).unwrap();
    assert!(paths1.contains(&"a.txt".to_string()));
    assert!(paths2.contains(&"b.txt".to_string()));
    assert!(!paths1.contains(&"b.txt".to_string()));
    assert!(!paths2.contains(&"a.txt".to_string()));
}
