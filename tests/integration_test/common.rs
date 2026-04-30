use opendog::core::project::ProjectManager;
use opendog::storage::database::Database;
use std::fs;
use tempfile::TempDir;

pub(crate) fn setup_manager() -> (TempDir, ProjectManager) {
    let dir = TempDir::new().unwrap();
    let mgr = ProjectManager::with_data_dir(dir.path()).unwrap();
    (dir, mgr)
}

pub(crate) fn setup_snapshot() -> (TempDir, Database) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open_project(&db_path).unwrap();
    (dir, db)
}

pub(crate) fn ensure_dir(path: &std::path::Path) {
    fs::create_dir_all(path).unwrap();
}

#[cfg(unix)]
pub(crate) fn run_cli(home: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_opendog"))
        .env("HOME", home)
        .args(args)
        .output()
        .unwrap()
}
