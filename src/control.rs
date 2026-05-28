use crate::config::{ProjectConfig, ProjectConfigView, ProjectInfo};
use crate::core::monitor::{self, MonitorHandle};
use crate::core::project::ProjectManager;
use crate::core::snapshot;
use crate::core::snapshot::SnapshotResult;
use crate::error::{OpenDogError, Result};
use crate::storage::database::Database;
use std::collections::HashMap;

mod client;
mod config_reload;
mod controller_queries;
mod fallback;
mod protocol;
mod request_handler;
#[cfg(test)]
mod tests;
mod transport;

pub use self::client::DaemonClient;
pub use self::fallback::{
    CliProjectLifecycle, DaemonProjectLifecycle, DirectProjectLifecycle, FallbackLifecycle,
    Guidance, ProjectLifecycle, SnapshotMonitor,
};
pub use self::protocol::{ControlRequest, ControlResponse, StartMonitorOutcome};
#[cfg(unix)]
pub use self::transport::{spawn_control_server, spawn_control_server_at};

pub struct MonitorController {
    pm: ProjectManager,
    monitors: HashMap<String, MonitorHandle>,
}

impl MonitorController {
    pub fn new() -> Result<Self> {
        Ok(Self {
            pm: ProjectManager::new()?,
            monitors: HashMap::new(),
        })
    }

    pub fn with_project_manager(pm: ProjectManager) -> Self {
        Self {
            pm,
            monitors: HashMap::new(),
        }
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        let mut projects = self.pm.list()?;
        for project in &mut projects {
            if self.monitors.contains_key(&project.id) {
                project.status = "monitoring".to_string();
            }
        }
        Ok(projects)
    }

    pub fn project_manager(&self) -> &ProjectManager {
        &self.pm
    }

    pub fn create_project(&self, id: &str, path: &str) -> Result<ProjectInfo> {
        self.pm.create(id, std::path::Path::new(path))
    }

    pub fn global_config(&self) -> Result<ProjectConfig> {
        self.pm.global_config()
    }

    pub fn project_config_view(&self, id: &str) -> Result<ProjectConfigView> {
        self.pm.project_config_view(id)
    }

    pub fn start_monitor(&mut self, id: &str) -> Result<StartMonitorOutcome> {
        if self.monitors.contains_key(id) {
            return Ok(StartMonitorOutcome {
                already_running: true,
                snapshot_taken: false,
            });
        }

        let info = self
            .pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let effective_config = self.pm.resolve_project_config(&info)?;
        let db = self.pm.open_project_db(id)?;
        let snapshot_taken = if snapshot::get_snapshot_count(&db)? == 0 {
            snapshot::take_snapshot(&db, &info.root_path, &effective_config)?;
            true
        } else {
            false
        };
        let handle =
            monitor::start_monitor(&info.db_path, info.root_path.clone(), effective_config)?;
        self.monitors.insert(id.to_string(), handle);
        Ok(StartMonitorOutcome {
            already_running: false,
            snapshot_taken,
        })
    }

    pub fn stop_monitor(&mut self, id: &str) -> bool {
        match self.monitors.remove(id) {
            Some(handle) => {
                handle.stop();
                true
            }
            None => false,
        }
    }

    pub fn stop_all(&mut self) {
        for handle in self.monitors.drain().map(|(_, handle)| handle) {
            handle.stop();
        }
    }

    pub fn take_snapshot(&self, id: &str) -> Result<SnapshotResult> {
        let info = self
            .pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let effective_config = self.pm.resolve_project_config(&info)?;
        let db = self.pm.open_project_db(id)?;
        snapshot::take_snapshot(&db, &info.root_path, &effective_config)
    }

    pub fn delete_project(&mut self, id: &str) -> Result<bool> {
        self.stop_monitor(id);
        self.pm.delete(id)
    }

    pub fn monitor_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.monitors.keys().cloned().collect();
        ids.sort();
        ids
    }

    /// Validate that `id` maps to a known project, open its database, then
    /// call `f` with the connection.  Eliminates the repeated
    /// get→ok_or→open_project_db preamble shared by every query method.
    fn with_project_db<F, T>(&self, id: &str, f: F) -> crate::error::Result<T>
    where
        F: FnOnce(&Database) -> crate::error::Result<T>,
    {
        self.pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = self.pm.open_project_db(id)?;
        f(&db)
    }

    /// Like [`with_project_db`], but also passes the resolved [`ProjectInfo`]
    /// to the closure.  Use when the delegate needs the project root path.
    fn with_project_info_db<F, T>(&self, id: &str, f: F) -> crate::error::Result<T>
    where
        F: FnOnce(&ProjectInfo, &Database) -> crate::error::Result<T>,
    {
        let info = self
            .pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = self.pm.open_project_db(id)?;
        f(&info, &db)
    }
}
