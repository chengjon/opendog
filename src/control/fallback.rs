use crate::config::ProjectInfo;
use crate::control::DaemonClient;
use crate::error::{OpenDogError, Result};

/// Narrow trait for project lifecycle operations: create, list, delete.
/// Implementations may talk to the daemon over IPC or hit ProjectManager
/// directly — callers never know which path is active.
pub trait ProjectLifecycle {
    fn create_project(&self, id: &str, path: &str) -> Result<ProjectInfo>;
    fn list_projects(&self) -> Result<Vec<ProjectInfo>>;
    fn delete_project(&self, id: &str) -> Result<bool>;
}

// ---------------------------------------------------------------------------
// Daemon-backed implementation
// ---------------------------------------------------------------------------

pub struct DaemonProjectLifecycle<'a> {
    client: &'a DaemonClient,
}

impl<'a> DaemonProjectLifecycle<'a> {
    pub fn new(client: &'a DaemonClient) -> Self {
        Self { client }
    }
}

impl ProjectLifecycle for DaemonProjectLifecycle<'_> {
    fn create_project(&self, id: &str, path: &str) -> Result<ProjectInfo> {
        self.client.create_project(id, path)
    }

    fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        self.client.list_projects()
    }

    fn delete_project(&self, id: &str) -> Result<bool> {
        self.client.delete_project(id)
    }
}

// ---------------------------------------------------------------------------
// Direct (ProjectManager-backed) implementation for MCP server
// ---------------------------------------------------------------------------

use crate::control::MonitorController;
use std::sync::Mutex;

pub struct DirectProjectLifecycle<'a> {
    controller: &'a Mutex<MonitorController>,
}

impl<'a> DirectProjectLifecycle<'a> {
    pub fn new(controller: &'a Mutex<MonitorController>) -> Self {
        Self { controller }
    }
}

impl ProjectLifecycle for DirectProjectLifecycle<'_> {
    fn create_project(&self, id: &str, path: &str) -> Result<ProjectInfo> {
        let inner = self.controller.lock().unwrap();
        inner.project_manager().create(id, std::path::Path::new(path))
    }

    fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        let inner = self.controller.lock().unwrap();
        inner.list_projects()
    }

    fn delete_project(&self, id: &str) -> Result<bool> {
        let mut inner = self.controller.lock().unwrap();
        // delete_project on MonitorController already calls stop_monitor internally
        inner.delete_project(id)
    }
}

// ---------------------------------------------------------------------------
// Direct (ProjectManager-backed) implementation for CLI
// ---------------------------------------------------------------------------

use crate::core::project::ProjectManager;

pub struct CliProjectLifecycle<'a> {
    pm: &'a ProjectManager,
}

impl<'a> CliProjectLifecycle<'a> {
    pub fn new(pm: &'a ProjectManager) -> Self {
        Self { pm }
    }
}

impl ProjectLifecycle for CliProjectLifecycle<'_> {
    fn create_project(&self, id: &str, path: &str) -> Result<ProjectInfo> {
        self.pm.create(id, std::path::Path::new(path))
    }

    fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        self.pm.list()
    }

    fn delete_project(&self, id: &str) -> Result<bool> {
        self.pm.delete(id)
    }
}

// ---------------------------------------------------------------------------
// Fallback: try daemon first, fall back to direct on DaemonUnavailable.
// Other daemon errors propagate directly.
// ---------------------------------------------------------------------------

pub struct FallbackLifecycle<D: ProjectLifecycle, L: ProjectLifecycle> {
    daemon: D,
    local: L,
}

impl<D: ProjectLifecycle, L: ProjectLifecycle> FallbackLifecycle<D, L> {
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
        self.fallback(
            |svc| svc.list_projects(),
            |svc| svc.list_projects(),
        )
    }

    fn delete_project(&self, id: &str) -> Result<bool> {
        self.fallback(
            |svc| svc.delete_project(id),
            |svc| svc.delete_project(id),
        )
    }
}
