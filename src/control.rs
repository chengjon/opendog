mod client;
mod config_reload;
mod controller;
mod controller_queries;
mod fallback;
mod protocol;
mod request_handler;
#[cfg(test)]
mod tests;
mod transport;

#[cfg(test)]
use crate::core::project::ProjectManager;

pub use self::client::DaemonClient;
pub use self::controller::MonitorController;
pub use self::fallback::{
    CliProjectLifecycle, DaemonProjectLifecycle, DirectProjectLifecycle, FallbackLifecycle,
    Guidance, ProjectLifecycle, SnapshotMonitor,
};
pub use self::protocol::{ControlRequest, ControlResponse, StartMonitorOutcome};
#[cfg(unix)]
pub use self::transport::{spawn_control_server, spawn_control_server_at};
