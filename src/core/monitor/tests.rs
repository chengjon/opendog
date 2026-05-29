use super::*;
use crate::core::scanner::FileSighting;
use crate::error::OpenDogError;
use notify::{event::ModifyKind, Event, EventKind};
use tempfile::TempDir;

fn test_db() -> (TempDir, Database) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("project.db");
    let db = Database::open_project(&db_path).unwrap();
    (dir, db)
}

fn single_sighting(path: &str, pid: i32) -> ScanResult {
    ScanResult {
        sightings: vec![FileSighting {
            file_path: path.to_string(),
            process_name: "claude".to_string(),
            pid,
        }],
        scan_duration_ms: 0,
        pids_scanned: 1,
    }
}

#[test]
fn process_scan_results_counts_every_sighting_and_avoids_duration_double_count() {
    let (_dir, db) = test_db();
    let mut open_state = HashMap::new();

    process_scan_results(
        &db,
        &single_sighting("src/main.rs", 42),
        &mut open_state,
        100,
    )
    .unwrap();
    process_scan_results(
        &db,
        &single_sighting("src/main.rs", 42),
        &mut open_state,
        103,
    )
    .unwrap();

    let stats: (i64, i64, Option<String>) = db
        .query_row(
            "SELECT access_count, estimated_duration_ms, last_access_time FROM file_stats WHERE file_path = ?1",
            params!["src/main.rs"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(stats.0, 2);
    assert_eq!(stats.1, 3000);
    assert_eq!(stats.2.as_deref(), Some("103"));

    process_scan_results(
        &db,
        &ScanResult {
            sightings: vec![],
            scan_duration_ms: 0,
            pids_scanned: 0,
        },
        &mut open_state,
        106,
    )
    .unwrap();

    let final_duration: i64 = db
        .query_row(
            "SELECT estimated_duration_ms FROM file_stats WHERE file_path = ?1",
            params!["src/main.rs"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(final_duration, 3000);
}

#[test]
fn record_file_event_stores_project_relative_paths() {
    let (dir, db) = test_db();
    let root = dir.path().join("project");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    let state = Arc::new(MonitorState {
        running: AtomicBool::new(true),
        active_threads: AtomicUsize::new(0),
        lock_path: dir.path().join("monitor.lock"),
        config: std::sync::RwLock::new(ProjectConfig::default()),
        snapshot_paths: std::sync::RwLock::new(HashSet::new()),
    });

    let event = Event::new(EventKind::Modify(ModifyKind::Any)).add_path(root.join("src/main.rs"));
    record_file_event(&db, &root, &state, &event).unwrap();

    let stored_path: String = db
        .query_row(
            "SELECT file_path FROM file_events LIMIT 1",
            params![],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored_path, "src/main.rs");
}

#[test]
fn acquire_monitor_lock_creates_lock_file() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("project.monitor.lock");
    assert!(acquire_monitor_lock(&lock_path).is_ok());
    let pid = std::fs::read_to_string(&lock_path).unwrap();
    assert_eq!(pid, std::process::id().to_string());
}

#[test]
fn acquire_monitor_lock_rejects_duplicate_for_current_pid() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("project.monitor.lock");
    acquire_monitor_lock(&lock_path).unwrap();
    let err = acquire_monitor_lock(&lock_path).unwrap_err();
    match err {
        OpenDogError::MonitorAlreadyRunning(pid) => {
            assert_eq!(pid, std::process::id().to_string());
        }
        other => panic!("expected MonitorAlreadyRunning, got {:?}", other),
    }
}

#[test]
fn acquire_monitor_lock_replaces_stale_lock() {
    let dir = tempfile::tempdir().unwrap();
    let lock_path = dir.path().join("project.monitor.lock");
    // Write a PID that doesn't exist
    std::fs::write(&lock_path, "999999999").unwrap();
    assert!(acquire_monitor_lock(&lock_path).is_ok());
    let pid = std::fs::read_to_string(&lock_path).unwrap();
    assert_eq!(pid, std::process::id().to_string());
}

#[test]
fn flush_open_durations_writes_accumulated_duration() {
    let (_dir, db) = test_db();
    // Pre-populate a file_stats row so the UPDATE has a target
    db.execute(
        "INSERT INTO file_stats (file_path, access_count, estimated_duration_ms, last_updated) VALUES (?1, 1, 0, ?2)",
        params!["src/lib.rs", "100"],
    )
    .unwrap();

    let mut open_state = HashMap::new();
    open_state.insert(
        ("src/lib.rs".to_string(), 42),
        OpenObservation { last_seen_at: 95 },
    );
    flush_open_durations(&db, &mut open_state, 100);

    let duration: i64 = db
        .query_row(
            "SELECT estimated_duration_ms FROM file_stats WHERE file_path = ?1",
            params!["src/lib.rs"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(duration, 5000); // (100 - 95) * 1000
}
