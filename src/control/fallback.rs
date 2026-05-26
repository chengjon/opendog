use crate::config::ProjectInfo;
use crate::control::DaemonClient;
use crate::core::snapshot::{self, SnapshotResult};
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

impl SnapshotMonitor for DaemonProjectLifecycle<'_> {
    fn take_snapshot(&self, id: &str) -> Result<SnapshotResult> {
        self.client.take_snapshot(id)
    }

    fn start_monitor(&self, id: &str) -> Result<StartMonitorOutcome> {
        self.client.start_monitor(id)
    }

    fn stop_monitor(&self, id: &str) -> Result<bool> {
        self.client.stop_monitor(id)
    }
}

impl Guidance for DaemonProjectLifecycle<'_> {
    fn get_agent_guidance(&self, project: Option<&str>, top: usize) -> Result<Value> {
        self.client.get_agent_guidance(project, top)
    }

    fn get_decision_brief(
        &self,
        schema_version: &str,
        project: Option<&str>,
        top: usize,
    ) -> Result<Value> {
        self.client.get_decision_brief(project, top, schema_version)
    }
}

// ---------------------------------------------------------------------------
// Direct (ProjectManager-backed) implementation for MCP server
// ---------------------------------------------------------------------------

use crate::control::MonitorController;
use std::sync::{Mutex, MutexGuard};

pub struct DirectProjectLifecycle<'a> {
    controller: &'a Mutex<MonitorController>,
}

impl<'a> DirectProjectLifecycle<'a> {
    pub fn new(controller: &'a Mutex<MonitorController>) -> Self {
        Self { controller }
    }

    fn controller(&self) -> Result<MutexGuard<'_, MonitorController>> {
        self.controller
            .lock()
            .map_err(|e| OpenDogError::LockPoisoned(format!("MonitorController: {}", e)))
    }
}

impl ProjectLifecycle for DirectProjectLifecycle<'_> {
    fn create_project(&self, id: &str, path: &str) -> Result<ProjectInfo> {
        let inner = self.controller()?;
        inner
            .project_manager()
            .create(id, std::path::Path::new(path))
    }

    fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        let inner = self.controller()?;
        inner.list_projects()
    }

    fn delete_project(&self, id: &str) -> Result<bool> {
        let mut inner = self.controller()?;
        inner.delete_project(id)
    }
}

impl SnapshotMonitor for DirectProjectLifecycle<'_> {
    fn take_snapshot(&self, id: &str) -> Result<SnapshotResult> {
        let inner = self.controller()?;
        inner.take_snapshot(id)
    }

    fn start_monitor(&self, id: &str) -> Result<StartMonitorOutcome> {
        let mut inner = self.controller()?;
        inner.start_monitor(id)
    }

    fn stop_monitor(&self, id: &str) -> Result<bool> {
        let mut inner = self.controller()?;
        Ok(inner.stop_monitor(id))
    }
}

impl Guidance for DirectProjectLifecycle<'_> {
    fn get_agent_guidance(&self, project: Option<&str>, top: usize) -> Result<Value> {
        let inner = self.controller()?;
        inner.get_agent_guidance(project, top)
    }

    fn get_decision_brief(
        &self,
        schema_version: &str,
        project: Option<&str>,
        top: usize,
    ) -> Result<Value> {
        let inner = self.controller()?;
        inner.get_decision_brief(schema_version, project, top)
    }
}

// ---------------------------------------------------------------------------
// Direct (ProjectManager-backed) implementation for CLI
// ---------------------------------------------------------------------------

use crate::core::project::ProjectManager;
use crate::core::stats;
use crate::guidance::{
    build_agent_guidance_for_projects, build_decision_brief_for_projects,
    load_project_guidance_data,
};

pub struct CliProjectLifecycle<'a> {
    pm: &'a ProjectManager,
}

impl<'a> CliProjectLifecycle<'a> {
    pub fn new(pm: &'a ProjectManager) -> Self {
        Self { pm }
    }

    fn guidance_projects(&self, project: Option<&str>) -> Result<Vec<ProjectInfo>> {
        let mut projects = self.pm.list()?;
        if let Some(project_id) = project {
            projects.retain(|p| p.id == project_id);
            if projects.is_empty() {
                return Err(OpenDogError::ProjectNotFound(project_id.to_string()));
            }
        }
        Ok(projects)
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

impl SnapshotMonitor for CliProjectLifecycle<'_> {
    fn take_snapshot(&self, id: &str) -> Result<SnapshotResult> {
        let info = self
            .pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let effective_config = self.pm.resolve_project_config(&info)?;
        let db = self.pm.open_project_db(id)?;
        snapshot::take_snapshot(&db, &info.root_path, &effective_config)
    }

    fn start_monitor(&self, _id: &str) -> Result<StartMonitorOutcome> {
        Err(OpenDogError::RemoteControl(
            "CLI direct start_monitor not supported through SnapshotMonitor trait".into(),
        ))
    }

    fn stop_monitor(&self, _id: &str) -> Result<bool> {
        Err(OpenDogError::RemoteControl(
            "CLI direct stop_monitor not supported through SnapshotMonitor trait".into(),
        ))
    }
}

impl Guidance for CliProjectLifecycle<'_> {
    fn get_agent_guidance(&self, project: Option<&str>, top: usize) -> Result<Value> {
        let projects = self.guidance_projects(project)?;
        Ok(build_agent_guidance_for_projects(
            self.pm,
            &projects,
            top.max(1),
            |p| load_project_guidance_data(self.pm, p),
        ))
    }

    fn get_decision_brief(
        &self,
        schema_version: &str,
        project: Option<&str>,
        top: usize,
    ) -> Result<Value> {
        let projects = self.guidance_projects(project)?;
        Ok(build_decision_brief_for_projects(
            self.pm,
            schema_version,
            if project.is_some() {
                "project"
            } else {
                "workspace"
            },
            project,
            &projects,
            top.max(1),
            |p| load_project_guidance_data(self.pm, p),
            |p| {
                self.pm
                    .open_project_db(&p.id)
                    .ok()
                    .and_then(|db| stats::get_stats(&db).ok())
                    .unwrap_or_default()
            },
        ))
    }
}

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
mod tests {
    use super::*;
    use std::cell::Cell;

    struct StubDaemon {
        succeed: bool,
        called: Cell<bool>,
    }

    struct StubLocal {
        called: Cell<bool>,
    }

    impl ProjectLifecycle for StubDaemon {
        fn create_project(&self, _id: &str, _path: &str) -> Result<ProjectInfo> {
            self.called.set(true);
            if self.succeed {
                Ok(ProjectInfo {
                    id: "test".into(),
                    root_path: "/tmp".into(),
                    db_path: "/tmp/test.db".into(),
                    config: crate::config::ProjectConfigOverrides::default(),
                    created_at: "0".into(),
                    status: "active".into(),
                })
            } else {
                Err(OpenDogError::DaemonUnavailable)
            }
        }
        fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
            self.called.set(true);
            if self.succeed {
                Ok(vec![])
            } else {
                Err(OpenDogError::DaemonUnavailable)
            }
        }
        fn delete_project(&self, _id: &str) -> Result<bool> {
            self.called.set(true);
            if self.succeed {
                Ok(true)
            } else {
                Err(OpenDogError::DaemonUnavailable)
            }
        }
    }

    impl ProjectLifecycle for StubLocal {
        fn create_project(&self, _id: &str, _path: &str) -> Result<ProjectInfo> {
            self.called.set(true);
            Ok(ProjectInfo {
                id: "test".into(),
                root_path: "/tmp".into(),
                db_path: "/tmp/test.db".into(),
                config: crate::config::ProjectConfigOverrides::default(),
                created_at: "0".into(),
                status: "active".into(),
            })
        }
        fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
            self.called.set(true);
            Ok(vec![])
        }
        fn delete_project(&self, _id: &str) -> Result<bool> {
            self.called.set(true);
            Ok(true)
        }
    }

    #[test]
    fn daemon_success_skips_local_fallback() {
        let daemon = StubDaemon {
            succeed: true,
            called: Cell::new(false),
        };
        let local = StubLocal {
            called: Cell::new(false),
        };
        let lifecycle = FallbackLifecycle::new(daemon, local);

        let result = lifecycle.create_project("p1", "/tmp/p1");
        assert!(result.is_ok());
        assert!(lifecycle.daemon.called.get());
        assert!(!lifecycle.local.called.get());
    }

    #[test]
    fn daemon_unavailable_cascades_to_local() {
        let daemon = StubDaemon {
            succeed: false,
            called: Cell::new(false),
        };
        let local = StubLocal {
            called: Cell::new(false),
        };
        let lifecycle = FallbackLifecycle::new(daemon, local);

        let result = lifecycle.list_projects();
        assert!(result.is_ok());
        assert!(lifecycle.daemon.called.get());
        assert!(lifecycle.local.called.get());
    }

    #[test]
    fn non_daemon_unavailable_error_propagates() {
        struct ErrorDaemon;
        impl ProjectLifecycle for ErrorDaemon {
            fn create_project(&self, _id: &str, _path: &str) -> Result<ProjectInfo> {
                Err(OpenDogError::ProjectNotFound("nope".into()))
            }
            fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
                Ok(vec![])
            }
            fn delete_project(&self, _id: &str) -> Result<bool> {
                Ok(true)
            }
        }
        struct NeverLocal;
        impl ProjectLifecycle for NeverLocal {
            fn create_project(&self, _id: &str, _path: &str) -> Result<ProjectInfo> {
                panic!("local should not be called for non-DaemonUnavailable errors");
            }
            fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
                Ok(vec![])
            }
            fn delete_project(&self, _id: &str) -> Result<bool> {
                Ok(true)
            }
        }

        let lifecycle = FallbackLifecycle::new(ErrorDaemon, NeverLocal);
        let err = lifecycle.create_project("x", "/x").unwrap_err();
        assert!(matches!(err, OpenDogError::ProjectNotFound(_)));
    }

    #[test]
    fn delete_project_cascades_on_daemon_unavailable() {
        let daemon = StubDaemon {
            succeed: false,
            called: Cell::new(false),
        };
        let local = StubLocal {
            called: Cell::new(false),
        };
        let lifecycle = FallbackLifecycle::new(daemon, local);

        let result = lifecycle.delete_project("p1");
        assert!(result.is_ok());
        assert!(lifecycle.daemon.called.get());
        assert!(lifecycle.local.called.get());
    }
}
