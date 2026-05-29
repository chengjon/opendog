use crate::error::{OpenDogError, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tracing::warn;

use super::MonitorState;

const INOTIFY_MAX_WATCHES_PATH: &str = "/proc/sys/fs/inotify/max_user_watches";

pub(super) fn monitor_lock_path(db_path: &Path) -> PathBuf {
    db_path.with_extension("monitor.lock")
}

pub(super) fn acquire_monitor_lock(lock_path: &Path) -> Result<()> {
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Atomic create-new eliminates the TOCTOU race between checking existence and writing.
    use std::fs::OpenOptions;
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(lock_path)
    {
        Ok(mut f) => {
            use std::io::Write;
            let _ = f.write_all(std::process::id().to_string().as_bytes());
            Ok(())
        }
        Err(_) => {
            // File exists: check whether the owner process is still alive.
            if let Ok(existing) = std::fs::read_to_string(lock_path) {
                let pid = existing.trim();
                if !pid.is_empty() && process_exists(pid) {
                    return Err(OpenDogError::MonitorAlreadyRunning(pid.to_string()));
                }
            }
            // Stale lock (owner gone). Remove and retry once.
            let _ = std::fs::remove_file(lock_path);
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(lock_path)
            {
                Ok(mut f) => {
                    use std::io::Write;
                    let _ = f.write_all(std::process::id().to_string().as_bytes());
                    Ok(())
                }
                Err(e) => Err(OpenDogError::Io(e)),
            }
        }
    }
}

fn release_monitor_lock(lock_path: &Path) {
    let _ = std::fs::remove_file(lock_path);
}

pub(super) fn thread_finished(state: &Arc<MonitorState>) {
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

pub(super) fn check_inotify_limits() -> Result<()> {
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

pub(super) fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
