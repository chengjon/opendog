use tracing::{error, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::mcp::OpenDogServer;

const PID_FILE: &str = "daemon.pid";

pub fn run() {
    init_logging();
    check_wsl();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    rt.block_on(async {
        if let Err(e) = run_daemon().await {
            error!("Daemon error: {}", e);
            std::process::exit(1);
        }
    });
}

async fn run_daemon() -> crate::error::Result<()> {
    use rmcp::ServiceExt;

    write_pid_file();

    let server = OpenDogServer::new()?;
    let transport = (tokio::io::stdin(), tokio::io::stdout());

    info!("OPENDOG daemon starting");

    // Notify systemd we're ready (DAEM-02)
    if let Err(e) = sd_notify::notify(&[sd_notify::NotifyState::Ready]) {
        warn!("sd_notify failed (not running under systemd?): {}", e);
    }

    // DAEM-03: graceful shutdown on SIGTERM
    tokio::select! {
        result = server.serve(transport) => {
            if let Err(e) = result {
                error!("Server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down");
        }
    }

    // Notify systemd we're stopping
    let _ = sd_notify::notify(&[sd_notify::NotifyState::Stopping]);
    remove_pid_file();
    info!("OPENDOG daemon stopped");
    Ok(())
}

/// DAEM-04: Initialize structured logging — journald if available, stderr otherwise.
fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("opendog=info"));

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

fn write_pid_file() {
    let path = pid_file_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let pid = std::process::id();
    let _ = std::fs::write(&path, pid.to_string());
}

fn remove_pid_file() {
    let _ = std::fs::remove_file(pid_file_path());
}
