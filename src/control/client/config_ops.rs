use crate::config::{
    ConfigPatch, GlobalConfigUpdateResult, ProjectConfig, ProjectConfigPatch, ProjectConfigReload,
    ProjectConfigUpdateResult, ProjectConfigView, ProjectInfo,
};
use crate::error::{OpenDogError, Result};

use super::{ControlRequest, ControlResponse, DaemonClient};

impl DaemonClient {
    pub fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        match self.send(ControlRequest::ListProjects)? {
            ControlResponse::Projects { projects } => Ok(projects),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon projects response: {:?}",
                response
            ))),
        }
    }

    pub fn global_config(&self) -> Result<ProjectConfig> {
        match self.send(ControlRequest::GetGlobalConfig)? {
            ControlResponse::GlobalConfig { config } => Ok(config),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon global config response: {:?}",
                response
            ))),
        }
    }

    pub fn get_project_config(&self, id: &str) -> Result<ProjectConfigView> {
        match self.send(ControlRequest::GetProjectConfig { id: id.to_string() })? {
            ControlResponse::ProjectConfig { view } => Ok(view),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon project config response: {:?}",
                response
            ))),
        }
    }

    pub fn update_global_config(&self, patch: ConfigPatch) -> Result<GlobalConfigUpdateResult> {
        match self.send(ControlRequest::UpdateGlobalConfig {
            ignore_patterns: patch.ignore_patterns,
            process_whitelist: patch.process_whitelist,
            add_ignore_patterns: patch.add_ignore_patterns,
            remove_ignore_patterns: patch.remove_ignore_patterns,
            add_process_whitelist: patch.add_process_whitelist,
            remove_process_whitelist: patch.remove_process_whitelist,
        })? {
            ControlResponse::GlobalConfigUpdated { result } => Ok(result),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon global config update response: {:?}",
                response
            ))),
        }
    }

    pub fn update_project_config(
        &self,
        id: &str,
        patch: ProjectConfigPatch,
    ) -> Result<ProjectConfigUpdateResult> {
        match self.send(ControlRequest::UpdateProjectConfig {
            id: id.to_string(),
            ignore_patterns: patch.ignore_patterns,
            process_whitelist: patch.process_whitelist,
            add_ignore_patterns: patch.add_ignore_patterns,
            remove_ignore_patterns: patch.remove_ignore_patterns,
            add_process_whitelist: patch.add_process_whitelist,
            remove_process_whitelist: patch.remove_process_whitelist,
            inherit_ignore_patterns: patch.inherit_ignore_patterns,
            inherit_process_whitelist: patch.inherit_process_whitelist,
        })? {
            ControlResponse::ProjectConfigUpdated { result } => Ok(result),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon project config update response: {:?}",
                response
            ))),
        }
    }

    pub fn reload_project_config(&self, id: &str) -> Result<(ProjectConfigReload, ProjectConfig)> {
        match self.send(ControlRequest::ReloadProjectConfig { id: id.to_string() })? {
            ControlResponse::ProjectConfigReloaded {
                reload, effective, ..
            } => Ok((reload, effective)),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon project config reload response: {:?}",
                response
            ))),
        }
    }
}
