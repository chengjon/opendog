use crate::config::ProjectInfo;
use crate::core::snapshot::SnapshotResult;
use crate::error::{OpenDogError, Result};
use serde_json::Value;

use super::protocol::StartMonitorOutcome;

/// Narrow trait for project lifecycle operations: create, list, delete.
pub trait ProjectLifecycle {
    fn create_project(&self, id: &str, path: &str) -> Result<ProjectInfo>;
    fn list_projects(&self) -> Result<Vec<ProjectInfo>>;
    fn delete_project(&self, id: &str) -> Result<bool>;
}

/// Narrow trait for snapshot and monitor operations.
pub trait SnapshotMonitor {
    fn take_snapshot(&self, id: &str) -> Result<SnapshotResult>;
    fn start_monitor(&self, id: &str) -> Result<StartMonitorOutcome>;
    fn stop_monitor(&self, id: &str) -> Result<bool>;
}

/// Narrow trait for guidance operations.
pub trait Guidance {
    fn get_agent_guidance(&self, project: Option<&str>, top: usize) -> Result<Value>;
    fn get_decision_brief(
        &self,
        schema_version: &str,
        project: Option<&str>,
        top: usize,
    ) -> Result<Value>;
}

mod cli;
mod daemon;
mod direct;

pub use cli::CliProjectLifecycle;
pub use daemon::DaemonProjectLifecycle;
pub use direct::DirectProjectLifecycle;

// ---------------------------------------------------------------------------
// Fallback: try daemon first, fall back to direct on DaemonUnavailable.
// ---------------------------------------------------------------------------

pub struct FallbackLifecycle<D, L> {
    daemon: D,
    local: L,
}

impl<D, L> FallbackLifecycle<D, L> {
    pub fn new(daemon: D, local: L) -> Self {
        Self { daemon, local }
    }

    fn fallback<Fd, Fl, T>(&self, daemon_op: Fd, local_op: Fl) -> Result<T>
    where
        Fd: FnOnce(&D) -> Result<T>,
        Fl: FnOnce(&L) -> Result<T>,
    {
        match daemon_op(&self.daemon) {
            Ok(value) => Ok(value),
            Err(OpenDogError::DaemonUnavailable) => local_op(&self.local),
            Err(e) => Err(e),
        }
    }
}

impl<D: ProjectLifecycle, L: ProjectLifecycle> ProjectLifecycle for FallbackLifecycle<D, L> {
    fn create_project(&self, id: &str, path: &str) -> Result<ProjectInfo> {
        self.fallback(
            |svc| svc.create_project(id, path),
            |svc| svc.create_project(id, path),
        )
    }

    fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        self.fallback(|svc| svc.list_projects(), |svc| svc.list_projects())
    }

    fn delete_project(&self, id: &str) -> Result<bool> {
        self.fallback(|svc| svc.delete_project(id), |svc| svc.delete_project(id))
    }
}

impl<D: SnapshotMonitor, L: SnapshotMonitor> SnapshotMonitor for FallbackLifecycle<D, L> {
    fn take_snapshot(&self, id: &str) -> Result<SnapshotResult> {
        self.fallback(|svc| svc.take_snapshot(id), |svc| svc.take_snapshot(id))
    }

    fn start_monitor(&self, id: &str) -> Result<StartMonitorOutcome> {
        self.fallback(|svc| svc.start_monitor(id), |svc| svc.start_monitor(id))
    }

    fn stop_monitor(&self, id: &str) -> Result<bool> {
        self.fallback(|svc| svc.stop_monitor(id), |svc| svc.stop_monitor(id))
    }
}

impl<D: Guidance, L: Guidance> Guidance for FallbackLifecycle<D, L> {
    fn get_agent_guidance(&self, project: Option<&str>, top: usize) -> Result<Value> {
        self.fallback(
            |svc| svc.get_agent_guidance(project, top),
            |svc| svc.get_agent_guidance(project, top),
        )
    }

    fn get_decision_brief(
        &self,
        schema_version: &str,
        project: Option<&str>,
        top: usize,
    ) -> Result<Value> {
        self.fallback(
            |svc| svc.get_decision_brief(schema_version, project, top),
            |svc| svc.get_decision_brief(schema_version, project, top),
        )
    }
}

#[cfg(test)]
mod tests;
