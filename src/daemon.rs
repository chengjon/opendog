use tracing::{error, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::config;
use crate::control::{spawn_control_server, DaemonClient, MonitorController};
use crate::error::OpenDogError;

const PID_FILE: &str = "daemon.pid";
const DAEMON_READY_TIMEOUT_SECS: u64 = 5;

pub fn run() {
    init_logging();
    check_wsl();

    let rt = match build_daemon_runtime() {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to create tokio runtime: {}", e);
            std::process::exit(1);
        }
    };

    rt.block_on(async {
        if let Err(e) = run_daemon().await {
            error!("Daemon error: {}", e);
            std::process::exit(1);
        }
    });
}

fn build_daemon_runtime() -> crate::error::Result<tokio::runtime::Runtime> {
    Ok(tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?)
}

pub fn ensure_running_for_mcp() -> crate::error::Result<()> {
    match daemon_startup_state()? {
        DaemonStartupState::Ready => Ok(()),
        DaemonStartupState::Starting => {
            wait_for_daemon_ready(std::time::Duration::from_secs(DAEMON_READY_TIMEOUT_SECS))
        }
        DaemonStartupState::Unavailable => {
            spawn_background_daemon()?;
            wait_for_daemon_ready(std::time::Duration::from_secs(DAEMON_READY_TIMEOUT_SECS))
        }
    }
}

async fn run_daemon() -> crate::error::Result<()> {
    write_pid_file()?;
    let controller = std::sync::Arc::new(std::sync::Mutex::new(MonitorController::new()?));
    let projects = {
        let controller = controller.lock().map_err(|e| {
            crate::error::OpenDogError::LockPoisoned(format!("daemon controller: {}", e))
        })?;
        controller.list_projects()?
    };

    for project in projects {
        if config::is_windows_mount_path(&project.root_path) {
            warn!(
                project_id = %project.id,
                root_path = %project.root_path.display(),
                "Project root is under /mnt; inotify support on Windows-mounted filesystems is unreliable"
            );
        }

        let mut controller_guard = controller.lock().map_err(|e| {
            crate::error::OpenDogError::LockPoisoned(format!("daemon controller: {}", e))
        })?;
        match controller_guard.start_monitor(&project.id) {
            Ok(outcome) => {
                info!(
                    project_id = %project.id,
                    snapshot_taken = outcome.snapshot_taken,
                    already_running = outcome.already_running,
                    "Started background monitor"
                );
            }
            Err(e) => {
                warn!(project_id = %project.id, error = %e, "Failed to start background monitor");
            }
        }
    }

    let server_running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    #[cfg(unix)]
    let control_thread = Some(spawn_control_server(
        controller.clone(),
        server_running.clone(),
    )?);
    #[cfg(not(unix))]
    let control_thread: Option<std::thread::JoinHandle<()>> = None;

    info!("OPENDOG daemon starting");

    // Notify systemd we're ready (DAEM-02)
    if let Err(e) = sd_notify::notify(&[sd_notify::NotifyState::Ready]) {
        warn!("sd_notify failed (not running under systemd?): {}", e);
    }

    // DAEM-03: graceful shutdown on SIGTERM / SIGINT
    let signal = wait_for_shutdown_signal().await?;
    info!(signal = %signal, "Received shutdown signal");

    server_running.store(false, std::sync::atomic::Ordering::Relaxed);
    if let Some(thread) = control_thread {
        thread.thread().unpark();
        let _ = thread.join();
    }

    let monitor_ids = {
        let controller = controller.lock().map_err(|e| {
            crate::error::OpenDogError::LockPoisoned(format!("daemon controller: {}", e))
        })?;
        controller.monitor_ids()
    };
    {
        let mut controller = controller.lock().map_err(|e| {
            crate::error::OpenDogError::LockPoisoned(format!("daemon controller: {}", e))
        })?;
        for project_id in &monitor_ids {
            info!(project_id = %project_id, "Stopping background monitor");
        }
        controller.stop_all();
    }
    std::thread::sleep(std::time::Duration::from_millis(250));

    // Notify systemd we're stopping
    let _ = sd_notify::notify(&[sd_notify::NotifyState::Stopping]);
    remove_pid_file();
    info!("OPENDOG daemon stopped");
    Ok(())
}

/// DAEM-04: Initialize structured logging — journald if available, stderr otherwise.
fn init_logging() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("opendog=info"));

    // Try journald first (works under systemd)
    if std::env::var("JOURNAL_STREAM").is_ok() {
        #[cfg(target_os = "linux")]
        {
            if let Ok(subscriber) = tracing_journald::layer() {
                tracing_subscriber::registry()
                    .with(subscriber)
                    .with(filter)
                    .init();
                return;
            }
        }
    }

    // Fallback: structured JSON to stderr
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}

/// DAEM-05: Detect WSL version and warn about limitations.
fn check_wsl() {
    if let Ok(version) = std::fs::read_to_string("/proc/version") {
        let version_lower = version.to_lowercase();
        if version_lower.contains("microsoft") {
            if version_lower.contains("wsl2") || version_lower.contains("microsoft-standard-wsl2") {
                info!("Detected WSL2 environment");
            } else {
                warn!(
                    "Detected WSL1 — inotify has poor support. \
                     Recommend upgrading to WSL2 for reliable file monitoring."
                );
            }
        }
    }
}

fn pid_file_path() -> std::path::PathBuf {
    crate::config::data_dir().join(PID_FILE)
}

fn write_pid_file() -> crate::error::Result<()> {
    let path = pid_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if path.exists() {
        if let Ok(existing) = std::fs::read_to_string(&path) {
            let existing = existing.trim();
            if !existing.is_empty() && process_exists(existing) {
                return Err(crate::error::OpenDogError::DaemonAlreadyRunning(
                    existing.to_string(),
                ));
            }
        }
    }

    let pid = std::process::id();
    std::fs::write(&path, pid.to_string())?;
    Ok(())
}

fn remove_pid_file() {
    let _ = std::fs::remove_file(pid_file_path());
}

enum DaemonStartupState {
    Ready,
    Starting,
    Unavailable,
}

fn daemon_startup_state() -> crate::error::Result<DaemonStartupState> {
    match DaemonClient::new().ping() {
        Ok(()) => Ok(DaemonStartupState::Ready),
        Err(OpenDogError::DaemonControlUnavailable) => Ok(DaemonStartupState::Starting),
        Err(OpenDogError::DaemonUnavailable) => Ok(DaemonStartupState::Unavailable),
        Err(e) => Err(e),
    }
}

fn spawn_background_daemon() -> crate::error::Result<()> {
    let current_exe = std::env::current_exe()?;
    std::process::Command::new(current_exe)
        .arg("daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    Ok(())
}

fn wait_for_daemon_ready(timeout: std::time::Duration) -> crate::error::Result<()> {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;

    let socket_path = config::daemon_socket_path();
    let (event_tx, event_rx) = mpsc::channel::<()>();
    let callback_tx = event_tx.clone();
    let mut _event_tx_keepalive = Some(event_tx);
    let mut _watcher = None;
    if let Some(parent) = socket_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            warn!(error = %e, "Failed to prepare daemon socket watch directory");
        } else {
            let watched_socket_path = socket_path.clone();
            match RecommendedWatcher::new(
                move |res: std::result::Result<notify::Event, notify::Error>| {
                    if let Ok(event) = res {
                        if event.paths.iter().any(|path| path == &watched_socket_path) {
                            let _ = callback_tx.send(());
                        }
                    }
                },
                Config::default(),
            ) {
                Ok(mut watcher) => {
                    if let Err(e) = watcher.watch(parent, RecursiveMode::NonRecursive) {
                        warn!(error = %e, "Failed to watch daemon socket directory");
                    } else {
                        _event_tx_keepalive = None;
                        _watcher = Some(watcher);
                    }
                }
                Err(e) => warn!(error = %e, "Failed to create daemon socket watcher"),
            }
        }
    }

    let deadline = std::time::Instant::now() + timeout;
    loop {
        match DaemonClient::new().ping() {
            Ok(()) => return Ok(()),
            Err(e @ (OpenDogError::DaemonUnavailable | OpenDogError::DaemonControlUnavailable)) => {
                let now = std::time::Instant::now();
                if now >= deadline {
                    return Err(e);
                }
                let remaining = deadline.saturating_duration_since(now);
                let wait_for = if socket_path.exists() {
                    remaining.min(std::time::Duration::from_millis(100))
                } else {
                    remaining
                };
                let _ = event_rx.recv_timeout(wait_for);
            }
            Err(e) => return Err(e),
        }
    }
}

fn process_exists(pid: &str) -> bool {
    #[cfg(unix)]
    {
        std::path::Path::new("/proc").join(pid).exists()
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}

async fn wait_for_shutdown_signal() -> crate::error::Result<&'static str> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm = signal(SignalKind::terminate())?;
        let signal = tokio::select! {
            result = tokio::signal::ctrl_c() => {
                result?;
                "SIGINT"
            },
            _ = sigterm.recv() => "SIGTERM",
        };
        Ok(signal)
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await?;
        Ok("SIGINT")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pid_file_path_ends_with_pid_filename() {
        let path = pid_file_path();
        assert_eq!(
            path.file_name().unwrap().to_str().unwrap(),
            PID_FILE,
            "pid file should end with '{}'",
            PID_FILE
        );
    }

    #[test]
    fn pid_file_path_lives_under_opendog_data_dir() {
        let path = pid_file_path();
        let path_str = path.to_str().unwrap();
        // data_dir() ends with "data", which lives under the .opendog root
        assert!(
            path_str.contains(".opendog"),
            "pid file path should be under .opendog directory, got: {}",
            path_str
        );
        assert!(
            path_str.contains("data"),
            "pid file path should be under data subdirectory, got: {}",
            path_str
        );
    }

    #[test]
    fn pid_file_constant_value() {
        // Document the expected constant value
        assert_eq!(PID_FILE, "daemon.pid");
    }
}
