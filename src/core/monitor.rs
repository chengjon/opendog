use crate::config::ProjectConfig;
use crate::core::scanner::{ProcScanner, ScanResult};
use crate::error::Result;
use crate::storage::database::Database;
use crate::storage::queries;
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tracing::{debug, error, info};

mod lock_snapshots;
mod runtime;
mod watcher;
use self::runtime::thread_finished;
use self::runtime::{acquire_monitor_lock, check_inotify_limits, monitor_lock_path, now_secs};
#[cfg(test)]
use self::watcher::record_file_event;
use self::watcher::start_file_watcher;

const DEFAULT_SCAN_INTERVAL_SECS: u64 = 3;

#[derive(Debug, Clone)]
struct OpenObservation {
    last_seen_at: u64,
}

struct MonitorState {
    running: AtomicBool,
    active_threads: AtomicUsize,
    lock_path: PathBuf,
    scan_wait: Mutex<()>,
    scan_wake: Condvar,
    config: std::sync::RwLock<ProjectConfig>,
    snapshot_paths: std::sync::RwLock<HashSet<String>>,
}

pub struct MonitorHandle {
    state: Arc<MonitorState>,
    watcher_tx: std::sync::mpsc::Sender<watcher::WatcherMessage>,
    scanner_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    watcher_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl MonitorHandle {
    pub fn is_running(&self) -> bool {
        self.state.running.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        {
            let _guard = match self.state.scan_wait.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            self.state.running.store(false, Ordering::Release);
            self.state.scan_wake.notify_all();
        }
        let _ = self.watcher_tx.send(watcher::WatcherMessage::Stop);
        Self::join_thread(&self.scanner_thread);
        Self::join_thread(&self.watcher_thread);
    }

    pub fn current_config(&self) -> ProjectConfig {
        lock_snapshots::read_config_snapshot(&self.state)
    }

    pub fn reload_config(&self, config: ProjectConfig, snapshot_paths: Option<HashSet<String>>) {
        lock_snapshots::replace_config_snapshot(&self.state, config);
        if let Some(snapshot_paths) = snapshot_paths {
            lock_snapshots::replace_snapshot_paths_snapshot(&self.state, snapshot_paths);
        }
    }

    fn join_thread(thread: &Mutex<Option<std::thread::JoinHandle<()>>>) {
        let thread = match thread.lock() {
            Ok(mut guard) => guard.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        };
        if let Some(thread) = thread {
            let _ = thread.join();
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
        scan_wait: Mutex::new(()),
        scan_wake: Condvar::new(),
        config: std::sync::RwLock::new(config),
        snapshot_paths: std::sync::RwLock::new(snapshot_paths),
    });

    let scanner_db_path = db_path.to_path_buf();
    let scanner_state = state.clone();
    let scanner_root = root_path.clone();
    let scanner_thread = std::thread::spawn(move || {
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
            let live_config = lock_snapshots::read_config_snapshot(&scanner_state);
            let snapshot_paths = lock_snapshots::read_snapshot_paths_snapshot(&scanner_state);
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

            let guard = match scanner_state.scan_wait.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            if !scanner_state.running.load(Ordering::Acquire) {
                break;
            }
            match scanner_state
                .scan_wake
                .wait_timeout(guard, Duration::from_secs(DEFAULT_SCAN_INTERVAL_SECS))
            {
                Ok((_guard, _timeout)) => {}
                Err(poisoned) => {
                    drop(poisoned.into_inner());
                }
            }
        }

        flush_open_durations(&db, &mut open_state, now_secs());
        info!("Monitor scanner stopped for {:?}", scanner_root);
        thread_finished(&scanner_state);
    });

    let watcher_db_path = db_path.to_path_buf();
    let watcher_state = state.clone();
    let watcher_root = root_path.clone();
    let (watcher_tx, watcher_rx) = std::sync::mpsc::channel::<watcher::WatcherMessage>();
    let watcher_event_tx = watcher_tx.clone();
    let watcher_thread = std::thread::spawn(move || {
        let db = match Database::open_project(&watcher_db_path) {
            Ok(d) => d,
            Err(e) => {
                error!(error = %e, "Watcher thread failed to open DB");
                thread_finished(&watcher_state);
                return;
            }
        };
        start_file_watcher(
            &db,
            &watcher_root,
            &watcher_state,
            watcher_rx,
            watcher_event_tx,
        );
    });

    Ok(MonitorHandle {
        state,
        watcher_tx,
        scanner_thread: Mutex::new(Some(scanner_thread)),
        watcher_thread: Mutex::new(Some(watcher_thread)),
    })
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
            if let Err(e) = db.execute(
                "UPDATE file_stats SET estimated_duration_ms = estimated_duration_ms + ?1, last_updated = ?2 WHERE file_path = ?3",
                params![duration_ms as i64, current_time.to_string(), key.0],
            ) {
                tracing::warn!("failed to flush open duration for {}: {}", key.0, e);
            }
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

#[cfg(test)]
mod tests;
