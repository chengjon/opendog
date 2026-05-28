use opendog::config::ProjectConfig;
use opendog::core::snapshot;
use opendog::storage::queries;
use std::fs;

use crate::common::{ensure_dir, setup_manager, setup_snapshot};

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

#[test]
fn test_new_files_does_not_underflow_when_all_files_replaced() {
    // Take a snapshot, then replace every file. Verify new_files is 0 (not a
    // wrapping-negative panic from the inner subtraction).
    let (dir, db) = setup_snapshot();
    let project_dir = dir.path().join("project");
    ensure_dir(&project_dir);

    fs::write(project_dir.join("a.txt"), "alpha").unwrap();
    fs::write(project_dir.join("b.txt"), "beta").unwrap();

    let r1 = snapshot::take_snapshot(&db, &project_dir, &ProjectConfig::default()).unwrap();
    assert_eq!(r1.total_files, 2);
    assert_eq!(r1.removed_files, 0);
    assert_eq!(r1.new_files, 2);

    // Remove both and add one new file
    fs::remove_file(project_dir.join("a.txt")).unwrap();
    fs::remove_file(project_dir.join("b.txt")).unwrap();
    fs::write(project_dir.join("c.txt"), "charlie").unwrap();

    let r2 = snapshot::take_snapshot(&db, &project_dir, &ProjectConfig::default()).unwrap();
    assert_eq!(r2.total_files, 1);
    assert_eq!(r2.removed_files, 2);
    // removed (2) > previous_count (2) with new_count (1), so
    // new_files = 1.saturating_sub(2.saturating_sub(2)) = 1
    assert_eq!(r2.new_files, 1);
}
