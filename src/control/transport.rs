use crate::error::{OpenDogError, Result};

use super::{ControlRequest, ControlResponse, MonitorController};
use std::io::{Read, Write};
use std::path::PathBuf;

#[cfg(unix)]
pub fn spawn_control_server(
    controller: std::sync::Arc<std::sync::Mutex<MonitorController>>,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<std::thread::JoinHandle<()>> {
    spawn_control_server_at(crate::config::daemon_socket_path(), controller, running)
}

#[cfg(unix)]
pub fn spawn_control_server_at(
    socket_path: PathBuf,
    controller: std::sync::Arc<std::sync::Mutex<MonitorController>>,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<std::thread::JoinHandle<()>> {
    use std::os::unix::net::UnixListener;
    use std::sync::atomic::Ordering;
    use std::time::Duration;
    use tracing::{info, warn};

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    let listener = UnixListener::bind(&socket_path)?;
    listener.set_nonblocking(true)?;
    info!(socket_path = %socket_path.display(), "Daemon control socket listening");

    let handle = std::thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _addr)) => {
                    let mut request_bytes = Vec::new();
                    if let Err(e) = stream.read_to_end(&mut request_bytes) {
                        warn!(error = %e, "Failed reading daemon control request");
                        continue;
                    }

                    let response = match serde_json::from_slice::<ControlRequest>(&request_bytes) {
                        Ok(request) => {
                            let mut controller = controller.lock().unwrap();
                            controller.handle_request(request)
                        }
                        Err(e) => ControlResponse::Error {
                            message: format!("Invalid control request: {}", e),
                        },
                    };

                    match serde_json::to_vec(&response) {
                        Ok(payload) => {
                            if let Err(e) = stream.write_all(&payload) {
                                warn!(error = %e, "Failed writing daemon control response");
                            }
                        }
                        Err(e) => warn!(error = %e, "Failed serializing daemon control response"),
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    warn!(error = %e, "Daemon control socket accept failed");
                    std::thread::sleep(Duration::from_millis(200));
                }
            }
        }

        let _ = std::fs::remove_file(&socket_path);
    });

    Ok(handle)
}

pub(super) fn map_connect_error(error: std::io::Error) -> OpenDogError {
    map_connect_error_with_liveness(error, crate::config::daemon_pid_is_live())
}

pub(super) fn map_connect_error_with_liveness(
    error: std::io::Error,
    daemon_pid_is_live: bool,
) -> OpenDogError {
    match error.kind() {
        std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused => {
            if daemon_pid_is_live {
                OpenDogError::DaemonControlUnavailable
            } else {
                OpenDogError::DaemonUnavailable
            }
        }
        _ => OpenDogError::Io(error),
    }
}
