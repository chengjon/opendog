use crate::config::ProjectConfig;
use crate::core::scanner::{ProcScanner, ScanResult};
use crate::error::{OpenDogError, Result};
use crate::storage::database::Database;
use crate::storage::queries;
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

mod watcher;

#[cfg(test)]
use self::watcher::record_file_event;
use self::watcher::start_file_watcher;

const DEFAULT_SCAN_INTERVAL_SECS: u64 = 3;
const INOTIFY_MAX_WATCHES_PATH: &str = "/proc/sys/fs/inotify/max_user_watches";

#[derive(Debug, Clone)]
struct OpenObservation {
    last_seen_at: u64,
}

struct MonitorState {
    running: AtomicBool,
    active_threads: AtomicUsize,
    lock_path: PathBuf,
    config: std::sync::RwLock<ProjectConfig>,
    snapshot_paths: std::sync::RwLock<HashSet<String>>,
}

pub struct MonitorHandle {
    state: Arc<MonitorState>,
    #[allow(dead_code)]
    root_path: PathBuf,
}

impl MonitorHandle {
    pub fn is_running(&self) -> bool {
        self.state.running.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.state.running.store(false, Ordering::Relaxed);
    }

    pub fn current_config(&self) -> ProjectConfig {
        self.state.config.read().unwrap().clone()
    }

    pub fn reload_config(&self, config: ProjectConfig, snapshot_paths: Option<HashSet<String>>) {
        *self.state.config.write().unwrap() = config;
        if let Some(snapshot_paths) = snapshot_paths {
            *self.state.snapshot_paths.write().unwrap() = snapshot_paths;
        }
    }
}

pub fn start_monitor(
    db_path: &Path,
    root_path: PathBuf,
    config: ProjectConfig,
) -> Result<MonitorHandle> {
    check_inotify_limits()?;

    let lock_path = monitor_lock_path(db_path);
    acquire_monitor_lock(&lock_path)?;

    let db = Database::open_project(db_path)?;
    let snapshot_paths: HashSet<String> = queries::get_snapshot_paths(&db)?.into_iter().collect();
    drop(db);

    let state = Arc::new(MonitorState {
        running: AtomicBool::new(true),
        active_threads: AtomicUsize::new(2),
        lock_path: lock_path.clone(),
        config: std::sync::RwLock::new(config),
        snapshot_paths: std::sync::RwLock::new(snapshot_paths),
    });
    let handle = MonitorHandle {
        state: state.clone(),
        root_path: root_path.clone(),
    };

    let scanner_db_path = db_path.to_path_buf();
    let scanner_state = state.clone();
    let scanner_root = root_path.clone();
    std::thread::spawn(move || {
        let db = match Database::open_project(&scanner_db_path) {
            Ok(d) => d,
            Err(e) => {
                error!(error = %e, "Scanner thread failed to open DB");
                thread_finished(&scanner_state);
                return;
            }
        };
        let mut open_state: HashMap<(String, i32), OpenObservation> = HashMap::new();

        info!("Monitor scanner started for {:?}", scanner_root);

        while scanner_state.running.load(Ordering::Relaxed) {
            let live_config = scanner_state.config.read().unwrap().clone();
            let snapshot_paths = scanner_state.snapshot_paths.read().unwrap().clone();
            let scanner = ProcScanner::new(
                &scanner_root,
                &live_config.process_whitelist,
                snapshot_paths,
            );
            let scan_result = scanner.scan();
            let current_time = now_secs();

            debug!(
                sightings = scan_result.sightings.len(),
                pids = scan_result.pids_scanned,
                "Scan"
            );

            if let Err(e) = process_scan_results(&db, &scan_result, &mut open_state, current_time) {
                error!(error = %e, "Failed to process scan results");
            }

            std::thread::sleep(Duration::from_secs(DEFAULT_SCAN_INTERVAL_SECS));
        }

        flush_open_durations(&db, &mut open_state, now_secs());
        info!("Monitor scanner stopped for {:?}", scanner_root);
        thread_finished(&scanner_state);
    });

    let watcher_db_path = db_path.to_path_buf();
    let watcher_state = state.clone();
    let watcher_root = root_path.clone();
    std::thread::spawn(move || {
        let db = match Database::open_project(&watcher_db_path) {
            Ok(d) => d,
            Err(e) => {
                error!(error = %e, "Watcher thread failed to open DB");
                thread_finished(&watcher_state);
                return;
            }
        };
        start_file_watcher(&db, &watcher_root, &watcher_state);
    });

    Ok(handle)
}

fn process_scan_results(
    db: &Database,
    result: &ScanResult,
    open_state: &mut HashMap<(String, i32), OpenObservation>,
    current_time: u64,
) -> Result<()> {
    let timestamp = current_time.to_string();
    let mut processed_this_scan: HashSet<(String, i32)> = HashSet::new();

    for sighting in &result.sightings {
        let key = (sighting.file_path.clone(), sighting.pid);
        if !processed_this_scan.insert(key.clone()) {
            continue;
        }

        db.execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
            params![sighting.file_path, sighting.process_name, sighting.pid, timestamp],
        )?;

        upsert_file_stats(db, &sighting.file_path, current_time)?;

        match open_state.get_mut(&key) {
            Some(state) => {
                let delta_ms = (current_time.saturating_sub(state.last_seen_at)) * 1000;
                if delta_ms > 0 {
                    db.execute(
                        "UPDATE file_stats SET estimated_duration_ms = estimated_duration_ms + ?1, last_updated = ?2 WHERE file_path = ?3",
                        params![delta_ms as i64, current_time.to_string(), key.0],
                    )?;
                }
                state.last_seen_at = current_time;
            }
            None => {
                open_state.insert(
                    key,
                    OpenObservation {
                        last_seen_at: current_time,
                    },
                );
            }
        }
    }

    let mut closed_keys = Vec::new();
    for key in open_state.keys() {
        if !processed_this_scan.contains(key) {
            closed_keys.push(key.clone());
        }
    }

    for key in closed_keys {
        open_state.remove(&key);
    }

    Ok(())
}

fn flush_open_durations(
    db: &Database,
    open_state: &mut HashMap<(String, i32), OpenObservation>,
    current_time: u64,
) {
    for (key, state) in open_state.drain() {
        let duration_ms = (current_time.saturating_sub(state.last_seen_at)) * 1000;
        if duration_ms > 0 {
            let _ = db.execute(
                "UPDATE file_stats SET estimated_duration_ms = estimated_duration_ms + ?1, last_updated = ?2 WHERE file_path = ?3",
                params![duration_ms as i64, current_time.to_string(), key.0],
            );
        }
    }
}

fn upsert_file_stats(db: &Database, file_path: &str, current_time: u64) -> Result<()> {
    let timestamp = current_time.to_string();
    db.execute(
        "INSERT INTO file_stats (file_path, access_count, last_access_time, first_seen_time, last_updated) VALUES (?1, 1, ?2, ?2, ?3)
         ON CONFLICT(file_path) DO UPDATE SET access_count = access_count + 1, last_access_time = ?2, last_updated = ?3",
        params![file_path, timestamp, timestamp],
    )?;
    Ok(())
}

fn monitor_lock_path(db_path: &Path) -> PathBuf {
    db_path.with_extension("monitor.lock")
}

fn acquire_monitor_lock(lock_path: &Path) -> Result<()> {
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if lock_path.exists() {
        if let Ok(existing) = std::fs::read_to_string(lock_path) {
            let pid = existing.trim();
            if !pid.is_empty() && process_exists(pid) {
                return Err(OpenDogError::MonitorAlreadyRunning(pid.to_string()));
            }
        }
    }

    std::fs::write(lock_path, std::process::id().to_string())?;
    Ok(())
}

fn release_monitor_lock(lock_path: &Path) {
    let _ = std::fs::remove_file(lock_path);
}

fn thread_finished(state: &Arc<MonitorState>) {
    if state.active_threads.fetch_sub(1, Ordering::AcqRel) == 1 {
        release_monitor_lock(&state.lock_path);
    }
}

fn process_exists(pid: &str) -> bool {
    #[cfg(unix)]
    {
        Path::new("/proc").join(pid).exists()
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}

fn check_inotify_limits() -> Result<()> {
    match std::fs::read_to_string(INOTIFY_MAX_WATCHES_PATH) {
        Ok(content) => {
            let max: u64 = content.trim().parse().unwrap_or(0);
            if max < 524288 {
                warn!(
                    max_watches = max,
                    "inotify max_user_watches is low. Consider increasing: sysctl fs.inotify.max_user_watches=524288"
                );
            }
            Ok(())
        }
        Err(_) => {
            warn!("Could not read inotify max_user_watches");
            Ok(())
        }
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::scanner::FileSighting;
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

        let event =
            Event::new(EventKind::Modify(ModifyKind::Any)).add_path(root.join("src/main.rs"));
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
}
