use crate::config::should_ignore_path;
use crate::error::Result;
use crate::storage::database::Database;
use rusqlite::params;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

use super::{now_secs, thread_finished, MonitorState};

pub(super) fn start_file_watcher(db: &Database, root: &Path, state: &Arc<MonitorState>) {
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
            thread_finished(state);
            return;
        }
    };

    if let Err(e) = watcher.watch(root, RecursiveMode::Recursive) {
        error!(error = %e, root = %root.display(), "Failed to start watching directory");
        thread_finished(state);
        return;
    }

    info!(root = %root.display(), "File watcher started");

    while state.running.load(Ordering::Relaxed) {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                let skip = event.paths.iter().any(|p| {
                    let name = p.file_name().unwrap_or_default().to_string_lossy();
                    name.ends_with(".db") || name.ends_with(".db-wal") || name.ends_with(".db-shm")
                });
                if skip {
                    continue;
                }
                if let Err(e) = record_file_event(db, root, state, &event) {
                    warn!(error = %e, "Failed to record file event");
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    let _ = watcher.unwatch(root);
    info!(root = %root.display(), "File watcher stopped");
    thread_finished(state);
}

pub(super) fn record_file_event(
    db: &Database,
    root: &Path,
    state: &Arc<MonitorState>,
    event: &notify::Event,
) -> Result<()> {
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
        let live_config = state.config.read().unwrap().clone();
        if should_ignore_path(&rel_path, &live_config) {
            continue;
        }

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
