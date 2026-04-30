use rmcp::handler::server::wrapper::Json;
use serde_json::{json, Value};

use crate::config::ProjectInfo;
use crate::error::OpenDogError;

use super::{versioned_error_payload, versioned_project_error_payload};

pub(super) fn open_dog_error_code(error: &OpenDogError) -> &'static str {
    match error {
        OpenDogError::ProjectExists(_) => "project_exists",
        OpenDogError::ProjectNotFound(_) => "project_not_found",
        OpenDogError::InvalidProjectId(_) => "invalid_project_id",
        OpenDogError::InvalidPath(_) => "invalid_path",
        OpenDogError::InvalidVerification(_) => "invalid_verification",
        OpenDogError::InvalidInput(_) => "invalid_input",
        OpenDogError::DaemonAlreadyRunning(_) => "daemon_already_running",
        OpenDogError::MonitorAlreadyRunning(_) => "monitor_already_running",
        OpenDogError::DaemonUnavailable => "daemon_unavailable",
        OpenDogError::DaemonControlUnavailable => "daemon_control_unavailable",
        OpenDogError::RemoteControl(_) => "remote_control_error",
        OpenDogError::Database(_) => "database_error",
        OpenDogError::Io(_) => "io_error",
        OpenDogError::Serialization(_) => "serialization_error",
        OpenDogError::Walk(_) => "walk_error",
    }
}

pub(super) fn error_json_for(
    schema_version: &str,
    project_id: Option<&str>,
    error: &OpenDogError,
) -> Json<Value> {
    let remediation = match error {
        OpenDogError::DaemonControlUnavailable => Some(json!({
            "socket_path": crate::config::daemon_socket_path().display().to_string(),
            "pid_path": crate::config::daemon_pid_path().display().to_string(),
            "suggested_actions": [
                "Check whether the daemon control socket exists and is reachable.",
                "Remove a stale daemon socket if the daemon is no longer serving requests.",
                "Restart `opendog daemon` cleanly after checking the daemon pid file."
            ]
        })),
        _ => None,
    };

    let extra_fields: Vec<(&'static str, Value)> = remediation
        .into_iter()
        .map(|value| ("remediation", value))
        .collect();

    let value = if let Some(project_id) = project_id {
        versioned_project_error_payload(
            schema_version,
            project_id,
            open_dog_error_code(error),
            &error.to_string(),
            extra_fields,
        )
    } else {
        versioned_error_payload(
            schema_version,
            open_dog_error_code(error),
            &error.to_string(),
            extra_fields,
        )
    };

    Json(value)
}

pub(super) fn validation_error_json(
    schema_version: &str,
    project_id: Option<&str>,
    error_code: &'static str,
    error_message: &str,
) -> Json<Value> {
    let value = if let Some(project_id) = project_id {
        versioned_project_error_payload(schema_version, project_id, error_code, error_message, [])
    } else {
        versioned_error_payload(schema_version, error_code, error_message, [])
    };
    Json(value)
}

pub(super) fn scoped_projects_or_error(
    mut projects: Vec<ProjectInfo>,
    project_id: Option<&str>,
) -> Result<Vec<ProjectInfo>, OpenDogError> {
    if let Some(project_id) = project_id {
        projects.retain(|project| project.id == project_id);
        if projects.is_empty() {
            return Err(OpenDogError::ProjectNotFound(project_id.to_string()));
        }
    }
    Ok(projects)
}
