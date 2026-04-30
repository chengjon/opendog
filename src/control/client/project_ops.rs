use crate::config::ProjectInfo;
use crate::control::StartMonitorOutcome;
use crate::core::retention::{ProjectDataCleanupRequest, ProjectDataCleanupResult};
use crate::core::snapshot::SnapshotResult;
use crate::error::{OpenDogError, Result};

use super::{ControlRequest, ControlResponse, DaemonClient};

impl DaemonClient {
    pub fn cleanup_project_data(
        &self,
        id: &str,
        request: ProjectDataCleanupRequest,
    ) -> Result<ProjectDataCleanupResult> {
        match self.send(ControlRequest::CleanupProjectData {
            id: id.to_string(),
            scope: request.scope.as_str().to_string(),
            older_than_days: request.older_than_days,
            keep_snapshot_runs: request.keep_snapshot_runs,
            vacuum: request.vacuum,
            dry_run: request.dry_run,
        })? {
            ControlResponse::CleanupProjectData { result, .. } => Ok(result),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon cleanup-project-data response: {:?}",
                response
            ))),
        }
    }

    pub fn create_project(&self, id: &str, path: &str) -> Result<ProjectInfo> {
        match self.send(ControlRequest::CreateProject {
            id: id.to_string(),
            path: path.to_string(),
        })? {
            ControlResponse::ProjectCreated { info } => Ok(info),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon create response: {:?}",
                response
            ))),
        }
    }

    pub fn delete_project(&self, id: &str) -> Result<bool> {
        match self.send(ControlRequest::DeleteProject { id: id.to_string() })? {
            ControlResponse::ProjectDeleted { deleted, .. } => Ok(deleted),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon delete response: {:?}",
                response
            ))),
        }
    }

    pub fn start_monitor(&self, id: &str) -> Result<StartMonitorOutcome> {
        match self.send(ControlRequest::StartMonitor { id: id.to_string() })? {
            ControlResponse::Started {
                already_running,
                snapshot_taken,
                ..
            } => Ok(StartMonitorOutcome {
                already_running,
                snapshot_taken,
            }),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon start response: {:?}",
                response
            ))),
        }
    }

    pub fn stop_monitor(&self, id: &str) -> Result<bool> {
        match self.send(ControlRequest::StopMonitor { id: id.to_string() })? {
            ControlResponse::Stopped { was_running, .. } => Ok(was_running),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon stop response: {:?}",
                response
            ))),
        }
    }

    pub fn take_snapshot(&self, id: &str) -> Result<SnapshotResult> {
        match self.send(ControlRequest::TakeSnapshot { id: id.to_string() })? {
            ControlResponse::Snapshot { result, .. } => Ok(result),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon snapshot response: {:?}",
                response
            ))),
        }
    }
}
