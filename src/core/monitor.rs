use crate::config::ProjectConfig;
use crate::core::scanner::{ProcScanner, ScanResult};
use std::path::PathBuf;
use crate::error::Result;
use crate::storage::database::Database;
use crate::storage::queries;
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

const DEFAULT_SCAN_INTERVAL_SECS: u64 = 3;
const INOTIFY_MAX_WATCHES_PATH: &str = "/proc/sys/fs/inotify/max_user_watches";

pub struct MonitorHandle {
    running: Arc<AtomicBool>,
    #[allow(dead_code)]
    root_path: PathBuf,
}

impl MonitorHandle {
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

pub fn start_monitor(
    db_path: &Path,
    root_path: PathBuf,
    config: ProjectConfig,
) -> Result<MonitorHandle> {
    check_inotify_limits()?;

    let running = Arc::new(AtomicBool::new(true));
    let handle = MonitorHandle {
        running: running.clone(),
        root_path: root_path.clone(),
    };

    // Read snapshot paths from main connection, then each thread opens its own
    let db = Database::open_project(db_path)?;
    let snapshot_paths: HashSet<String> = queries::get_snapshot_paths(&db)?.into_iter().collect();
    let process_whitelist = config.process_whitelist.clone();
    drop(db);

    let scanner_db_path = db_path.to_path_buf();
    let scanner_running = running.clone();
    let scanner_root = root_path.clone();
    std::thread::spawn(move || {
        let db = match Database::open_project(&scanner_db_path) {
            Ok(d) => d,
            Err(e) => { error!(error = %e, "Scanner thread failed to open DB"); return; }
        };
        let scanner = ProcScanner::new(&scanner_root, &process_whitelist, snapshot_paths);
        let mut open_state: HashMap<(String, i32), u64> = HashMap::new();
        let mut last_scan_time = now_secs();

        info!("Monitor scanner started for {:?}", scanner_root);

        while scanner_running.load(Ordering::Relaxed) {
            let scan_result = scanner.scan();
            let current_time = now_secs();
            let scan_interval_ms = (current_time - last_scan_time) * 1000;

            debug!(sightings = scan_result.sightings.len(), pids = scan_result.pids_scanned, "Scan");

            if let Err(e) = process_scan_results(&db, &scan_result, &mut open_state, current_time, scan_interval_ms) {
                error!(error = %e, "Failed to process scan results");
            }

            last_scan_time = current_time;
            std::thread::sleep(Duration::from_secs(DEFAULT_SCAN_INTERVAL_SECS));
        }

        flush_open_durations(&db, &mut open_state, now_secs());
        info!("Monitor scanner stopped for {:?}", scanner_root);
    });

    let watcher_db_path = db_path.to_path_buf();
    let watcher_running = running.clone();
    let watcher_root = root_path.clone();
    std::thread::spawn(move || {
        let db = match Database::open_project(&watcher_db_path) {
            Ok(d) => d,
            Err(e) => { error!(error = %e, "Watcher thread failed to open DB"); return; }
        };
        start_file_watcher(&db, &watcher_root, &watcher_running);
    });

    Ok(handle)
}

fn process_scan_results(
    db: &Database,
    result: &ScanResult,
    open_state: &mut HashMap<(String, i32), u64>,
    current_time: u64,
    scan_interval_ms: u64,
) -> Result<()> {
    let current_sightings: HashSet<(String, i32)> = result
        .sightings
        .iter()
        .map(|s| (s.file_path.clone(), s.pid))
        .collect();

    // Record sightings and update stats
    let timestamp = current_time.to_string();
    for sighting in &result.sightings {
        // Record raw sighting
        db.execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
            params![sighting.file_path, sighting.process_name, sighting.pid, timestamp],
        )?;

        let key = (sighting.file_path.clone(), sighting.pid);
        if !open_state.contains_key(&key) {
            // Newly opened file
            open_state.insert(key, current_time);

            // Ensure file_stats entry exists and update access count
            upsert_file_stats(db, &sighting.file_path, current_time)?;
        }
    }

    // Detect closed files (were open, now absent)
    let mut closed_keys = Vec::new();
    for (key, open_time) in open_state.iter() {
        if !current_sightings.contains(key) {
            let duration_ms = (current_time - open_time) * 1000;
            closed_keys.push(key.clone());

            // Accumulate duration
            db.execute(
                "UPDATE file_stats SET estimated_duration_ms = estimated_duration_ms + ?1, last_updated = ?2 WHERE file_path = ?3",
                params![duration_ms as i64, current_time.to_string(), key.0],
            )?;

            debug!(
                file = %key.0,
                pid = key.1,
                duration_ms,
                "File closed (estimated)"
            );
        }
    }

    for key in closed_keys {
        open_state.remove(&key);
    }

    // For still-open files, accumulate interval duration
    for (key, _) in open_state.iter() {
        db.execute(
            "UPDATE file_stats SET estimated_duration_ms = estimated_duration_ms + ?1, last_updated = ?2 WHERE file_path = ?3",
            params![scan_interval_ms as i64, current_time.to_string(), key.0],
        )?;
    }

    Ok(())
}

fn flush_open_durations(
    db: &Database,
    open_state: &mut HashMap<(String, i32), u64>,
    current_time: u64,
) {
    for (key, open_time) in open_state.drain() {
        let duration_ms = (current_time - open_time) * 1000;
        let _ = db.execute(
            "UPDATE file_stats SET estimated_duration_ms = estimated_duration_ms + ?1, last_updated = ?2 WHERE file_path = ?3",
            params![duration_ms as i64, current_time.to_string(), key.0],
        );
    }
}

fn upsert_file_stats(db: &Database, file_path: &str, current_time: u64) -> Result<()> {
    let timestamp = current_time.to_string();
    db.execute(
        "INSERT INTO file_stats (file_path, access_count, first_seen_time, last_updated) VALUES (?1, 1, ?2, ?3)
         ON CONFLICT(file_path) DO UPDATE SET access_count = access_count + 1, last_access_time = ?2, last_updated = ?3",
        params![file_path, timestamp, timestamp],
    )?;
    Ok(())
}

fn start_file_watcher(db: &Database, root: &Path, running: &AtomicBool) {
    use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel::<Event>();

    let mut watcher = match RecommendedWatcher::new(
        move |res: std::result::Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            error!(error = %e, "Failed to create file watcher");
            return;
        }
    };

    if let Err(e) = watcher.watch(root, RecursiveMode::Recursive) {
        error!(error = %e, root = %root.display(), "Failed to start watching directory");
        return;
    }

    info!(root = %root.display(), "File watcher started");

    while running.load(Ordering::Relaxed) {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                // Skip events on database files
                let skip = event.paths.iter().any(|p| {
                    let name = p.file_name().unwrap_or_default().to_string_lossy();
                    name.ends_with(".db") || name.ends_with(".db-wal") || name.ends_with(".db-shm")
                });
                if skip {
                    continue;
                }
                if let Err(e) = record_file_event(db, &event) {
                    warn!(error = %e, "Failed to record file event");
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    let _ = watcher.unwatch(root);
    info!(root = %root.display(), "File watcher stopped");
}

fn record_file_event(db: &Database, event: &notify::Event) -> Result<()> {
    use notify::EventKind;
    let event_type = match event.kind {
        EventKind::Create(_) => "create",
        EventKind::Modify(_) => "modify",
        EventKind::Remove(_) => "remove",
        EventKind::Any => "any",
        _ => return Ok(()), // Skip other event types
    };

    let timestamp = now_secs().to_string();

    for path in &event.paths {
        let path_str = path.to_str().unwrap_or("");
        if path_str.is_empty() {
            continue;
        }

        db.execute(
            "INSERT INTO file_events (file_path, event_type, event_time) VALUES (?1, ?2, ?3)",
            params![path_str, event_type, timestamp],
        )?;

        // Update modification count for modify events
        if event_type == "modify" {
            db.execute(
                "INSERT INTO file_stats (file_path, modification_count, first_seen_time, last_updated) VALUES (?1, 1, ?2, ?2)
                 ON CONFLICT(file_path) DO UPDATE SET modification_count = modification_count + 1, last_updated = ?2",
                params![path_str, timestamp],
            )?;
        }
    }

    Ok(())
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
