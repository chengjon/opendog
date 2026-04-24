use crate::config::ProjectConfig;
use crate::core::scanner::{ProcScanner, ScanResult};
use crate::error::Result;
use crate::storage::database::Database;
use crate::storage::queries;
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

const DEFAULT_SCAN_INTERVAL_SECS: u64 = 3;
const INOTIFY_MAX_WATCHES_PATH: &str = "/proc/sys/fs/inotify/max_user_watches";

#[derive(Debug, Clone)]
struct OpenObservation {
    last_seen_at: u64,
}

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
            Err(e) => {
                error!(error = %e, "Scanner thread failed to open DB");
                return;
            }
        };
        let scanner = ProcScanner::new(&scanner_root, &process_whitelist, snapshot_paths);
        let mut open_state: HashMap<(String, i32), OpenObservation> = HashMap::new();

        info!("Monitor scanner started for {:?}", scanner_root);

        while scanner_running.load(Ordering::Relaxed) {
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
    });

    let watcher_db_path = db_path.to_path_buf();
    let watcher_running = running.clone();
    let watcher_root = root_path.clone();
    std::thread::spawn(move || {
        let db = match Database::open_project(&watcher_db_path) {
            Ok(d) => d,
            Err(e) => {
                error!(error = %e, "Watcher thread failed to open DB");
                return;
            }
        };
        start_file_watcher(&db, &watcher_root, &watcher_running);
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
                let skip = event.paths.iter().any(|p| {
                    let name = p.file_name().unwrap_or_default().to_string_lossy();
                    name.ends_with(".db") || name.ends_with(".db-wal") || name.ends_with(".db-shm")
                });
                if skip {
                    continue;
                }
                if let Err(e) = record_file_event(db, root, &event) {
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

fn record_file_event(db: &Database, root: &Path, event: &notify::Event) -> Result<()> {
    use notify::EventKind;

    let event_type = match event.kind {
        EventKind::Create(_) => "create",
        EventKind::Modify(_) => "modify",
        EventKind::Remove(_) => "remove",
        EventKind::Any => "any",
        _ => return Ok(()),
    };

    let timestamp = now_secs().to_string();

    for path in &event.paths {
        let Some(rel_path) = normalize_event_path(root, path) else {
            continue;
        };

        db.execute(
            "INSERT INTO file_events (file_path, event_type, event_time) VALUES (?1, ?2, ?3)",
            params![rel_path, event_type, timestamp],
        )?;

        if event_type == "modify" {
            db.execute(
                "INSERT INTO file_stats (file_path, modification_count, first_seen_time, last_updated) VALUES (?1, 1, ?2, ?2)
                 ON CONFLICT(file_path) DO UPDATE SET modification_count = modification_count + 1, last_updated = ?2",
                params![rel_path, timestamp],
            )?;
        }
    }

    Ok(())
}

fn normalize_event_path(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root)
        .ok()
        .and_then(|p| p.to_str())
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
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

        process_scan_results(&db, &single_sighting("src/main.rs", 42), &mut open_state, 100)
            .unwrap();
        process_scan_results(&db, &single_sighting("src/main.rs", 42), &mut open_state, 103)
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

        let event = Event::new(EventKind::Modify(ModifyKind::Any)).add_path(root.join("src/main.rs"));
        record_file_event(&db, &root, &event).unwrap();

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
