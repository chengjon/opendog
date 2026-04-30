use crate::error::{OpenDogError, Result};
use std::io::{Read, Write};
use std::path::PathBuf;

use super::transport;
use super::{ControlRequest, ControlResponse};

mod config_ops;
mod guidance_ops;
mod project_ops;
mod report_ops;
mod verification_ops;

pub struct DaemonClient {
    socket_path: PathBuf,
}

impl Default for DaemonClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DaemonClient {
    pub fn new() -> Self {
        Self::with_socket_path(crate::config::daemon_socket_path())
    }

    pub fn with_socket_path(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    pub fn ping(&self) -> Result<()> {
        match self.send(ControlRequest::Ping)? {
            ControlResponse::Pong => Ok(()),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon ping response: {:?}",
                response
            ))),
        }
    }

    pub fn list_monitors(&self) -> Result<Vec<String>> {
        match self.send(ControlRequest::ListMonitors)? {
            ControlResponse::Monitors { ids } => Ok(ids),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon list response: {:?}",
                response
            ))),
        }
    }

    fn send(&self, request: ControlRequest) -> Result<ControlResponse> {
        #[cfg(unix)]
        {
            use std::os::unix::net::UnixStream;

            let mut stream =
                UnixStream::connect(&self.socket_path).map_err(transport::map_connect_error)?;
            let payload = serde_json::to_vec(&request)?;
            stream.write_all(&payload)?;
            stream.shutdown(std::net::Shutdown::Write)?;

            let mut response = Vec::new();
            stream.read_to_end(&mut response)?;
            Ok(serde_json::from_slice(&response)?)
        }

        #[cfg(not(unix))]
        {
            let _ = request;
            Err(OpenDogError::RemoteControl(
                "Daemon IPC is only supported on unix platforms".to_string(),
            ))
        }
    }
}
