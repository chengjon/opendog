use crate::config::{
    changed_config_fields, ConfigPatch, GlobalConfigUpdateResult, ProjectConfig,
    ProjectConfigPatch, ProjectConfigReload, ProjectConfigUpdateResult, ProjectConfigView,
    ProjectInfo, ProjectReloadStatus,
};
use crate::core::monitor::{self, MonitorHandle};
use crate::core::project::ProjectManager;
use crate::core::snapshot;
use crate::core::snapshot::SnapshotResult;
use crate::error::{OpenDogError, Result};
use std::collections::HashMap;

mod client;
mod controller_queries;
mod protocol;
mod request_handler;
#[cfg(test)]
mod tests;
mod transport;

pub use self::client::DaemonClient;
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

    pub fn update_project_config(
        &mut self,
        id: &str,
        patch: ProjectConfigPatch,
    ) -> Result<ProjectConfigUpdateResult> {
        let mut result = self.pm.update_project_config(id, patch)?;
        let reload = self.reload_project_runtime(id, &result.effective)?;
        result.reload = reload;
        Ok(result)
    }

    pub fn update_global_config(&mut self, patch: ConfigPatch) -> Result<GlobalConfigUpdateResult> {
        let before_effective = self
            .pm
            .list()?
            .into_iter()
            .map(|project| {
                let effective = self.pm.resolve_project_config(&project)?;
                Ok((project.id, effective))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let global_defaults = self.pm.update_global_config(patch)?;
        let mut reloaded_projects = Vec::new();

        for project in self.pm.list()? {
            let before = before_effective
                .get(&project.id)
                .cloned()
                .unwrap_or_else(ProjectConfig::default);
            let after = self.pm.resolve_project_config(&project)?;
            let changed_fields = changed_config_fields(&before, &after);
            if changed_fields.is_empty() {
                continue;
            }

            let reload =
                self.reload_project_runtime_with_changes(&project.id, &after, changed_fields)?;
            reloaded_projects.push(ProjectReloadStatus {
                project_id: project.id.clone(),
                monitor_running: reload.monitor_running,
                runtime_reloaded: reload.runtime_reloaded,
                snapshot_refreshed: reload.snapshot_refreshed,
                changed_fields: reload.changed_fields,
                skipped_fields: reload.skipped_fields,
            });
        }

        Ok(GlobalConfigUpdateResult {
            global_defaults,
            reloaded_projects,
        })
    }

    pub fn reload_project_config(&mut self, id: &str) -> Result<ProjectConfigReload> {
        let effective = self.pm.effective_project_config(id)?;
        self.reload_project_runtime(id, &effective)
    }

    fn reload_project_runtime(
        &mut self,
        id: &str,
        effective: &ProjectConfig,
    ) -> Result<ProjectConfigReload> {
        let previous = self
            .monitors
            .get(id)
            .map(|handle| handle.current_config())
            .unwrap_or_else(|| effective.clone());
        let changed_fields = changed_config_fields(&previous, effective);
        self.reload_project_runtime_with_changes(id, effective, changed_fields)
    }

    fn reload_project_runtime_with_changes(
        &mut self,
        id: &str,
        effective: &ProjectConfig,
        changed_fields: Vec<String>,
    ) -> Result<ProjectConfigReload> {
        let monitor_running = self.monitors.contains_key(id);
        let mut reload = ProjectConfigReload {
            monitor_running,
            runtime_reloaded: false,
            snapshot_refreshed: false,
            changed_fields,
            skipped_fields: Vec::new(),
        };

        if reload.changed_fields.is_empty() {
            reload
                .skipped_fields
                .push("effective configuration unchanged".to_string());
            return Ok(reload);
        }

        if !monitor_running {
            reload.skipped_fields.push(
                "monitor is not running; persisted configuration will apply on next start"
                    .to_string(),
            );
            return Ok(reload);
        }

        let info = self
            .pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let mut snapshot_paths = None;
        if reload
            .changed_fields
            .iter()
            .any(|field| field == "ignore_patterns")
        {
            let db = self.pm.open_project_db(id)?;
            snapshot::take_snapshot(&db, &info.root_path, effective)?;
            snapshot_paths = Some(
                snapshot::get_snapshot_paths(&db)?
                    .into_iter()
                    .collect::<std::collections::HashSet<_>>(),
            );
            reload.snapshot_refreshed = true;
        }

        if let Some(handle) = self.monitors.get(id) {
            handle.reload_config(effective.clone(), snapshot_paths);
            reload.runtime_reloaded = true;
        }

        Ok(reload)
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
}
