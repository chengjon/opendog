use crate::config::ProjectInfo;
use crate::control::protocol::StartMonitorOutcome;
use crate::core::project::ProjectManager;
use crate::core::snapshot::{self, SnapshotResult};
use crate::core::stats;
use crate::error::{OpenDogError, Result};
use crate::guidance::{
    build_agent_guidance_for_projects, build_decision_brief_for_projects,
    load_project_guidance_data, DecisionBriefProjectsInput,
};
use serde_json::Value;

use super::{Guidance, ProjectLifecycle, SnapshotMonitor};

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
            DecisionBriefProjectsInput {
                pm: self.pm,
                schema_version,
                scope: if project.is_some() {
                    "project"
                } else {
                    "workspace"
                },
                selected_project_id: project,
                projects: &projects,
                top: top.max(1),
            },
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
