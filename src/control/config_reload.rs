use crate::config::{
    changed_config_fields, ConfigPatch, GlobalConfigUpdateResult, ProjectConfig,
    ProjectConfigPatch, ProjectConfigReload, ProjectConfigUpdateResult, ProjectReloadStatus,
};
use crate::core::snapshot;
use crate::error::{OpenDogError, Result};
use std::collections::{HashMap, HashSet};

use super::MonitorController;

impl MonitorController {
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
                    .collect::<HashSet<_>>(),
            );
            reload.snapshot_refreshed = true;
        }

        if let Some(handle) = self.monitors.get(id) {
            handle.reload_config(effective.clone(), snapshot_paths);
            reload.runtime_reloaded = true;
        }

        Ok(reload)
    }
}
