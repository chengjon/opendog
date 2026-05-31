use std::collections::HashSet;

use tracing::warn;

use crate::config::ProjectConfig;

use super::MonitorState;

pub(super) fn read_config_snapshot(state: &MonitorState) -> ProjectConfig {
    match state.config.read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => {
            warn!("Monitor config lock was poisoned; recovering current config snapshot");
            poisoned.into_inner().clone()
        }
    }
}

pub(super) fn replace_config_snapshot(state: &MonitorState, config: ProjectConfig) {
    match state.config.write() {
        Ok(mut guard) => *guard = config,
        Err(poisoned) => {
            warn!("Monitor config lock was poisoned; replacing recovered config snapshot");
            *poisoned.into_inner() = config;
        }
    }
}

pub(super) fn read_snapshot_paths_snapshot(state: &MonitorState) -> HashSet<String> {
    match state.snapshot_paths.read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => {
            warn!("Monitor snapshot path lock was poisoned; recovering current path snapshot");
            poisoned.into_inner().clone()
        }
    }
}

pub(super) fn replace_snapshot_paths_snapshot(
    state: &MonitorState,
    snapshot_paths: HashSet<String>,
) {
    match state.snapshot_paths.write() {
        Ok(mut guard) => *guard = snapshot_paths,
        Err(poisoned) => {
            warn!("Monitor snapshot path lock was poisoned; replacing recovered path snapshot");
            *poisoned.into_inner() = snapshot_paths;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    use std::sync::{Arc, Condvar, Mutex};

    use super::*;

    fn monitor_state(config: ProjectConfig, snapshot_paths: HashSet<String>) -> Arc<MonitorState> {
        Arc::new(MonitorState {
            running: AtomicBool::new(true),
            active_threads: AtomicUsize::new(0),
            lock_path: PathBuf::from("monitor.lock"),
            scan_wait: Mutex::new(()),
            scan_wake: Condvar::new(),
            config: std::sync::RwLock::new(config),
            snapshot_paths: std::sync::RwLock::new(snapshot_paths),
        })
    }

    #[test]
    fn read_config_snapshot_recovers_from_poisoned_lock() {
        let state = monitor_state(ProjectConfig::default(), HashSet::new());
        let poisoned = state.clone();
        let _ = std::thread::spawn(move || {
            let _guard = poisoned.config.write().unwrap();
            panic!("poison config lock");
        })
        .join();

        let config = read_config_snapshot(&state);

        assert_eq!(config, ProjectConfig::default());
    }

    #[test]
    fn read_snapshot_paths_snapshot_recovers_from_poisoned_lock() {
        let state = monitor_state(
            ProjectConfig::default(),
            HashSet::from(["src/main.rs".to_string()]),
        );
        let poisoned = state.clone();
        let _ = std::thread::spawn(move || {
            let _guard = poisoned.snapshot_paths.write().unwrap();
            panic!("poison snapshot paths lock");
        })
        .join();

        let paths = read_snapshot_paths_snapshot(&state);

        assert!(paths.contains("src/main.rs"));
    }
}
