use rmcp::handler::server::wrapper::Json;
use serde_json::{json, Value};

use crate::error::OpenDogError;

use super::{versioned_error_payload, versioned_project_error_payload};

pub(super) fn open_dog_error_code(error: &OpenDogError) -> &'static str {
    match error {
        OpenDogError::ProjectExists(_) => "project_exists",
        OpenDogError::ProjectNotFound(_) => "project_not_found",
        OpenDogError::InvalidProjectId(_) => "invalid_project_id",
        OpenDogError::InvalidPath(_) => "invalid_path",
        OpenDogError::InvalidVerification(_) => "invalid_verification",
        OpenDogError::VerificationRecordMissing(_) => "verification_record_missing",
        OpenDogError::GovernanceLaneNotFound(_) => "governance_lane_not_found",
        OpenDogError::GovernanceNodeStateRequired(_) => "governance_node_state_required",
        OpenDogError::InvalidInput(_) => "invalid_input",
        OpenDogError::DaemonAlreadyRunning(_) => "daemon_already_running",
        OpenDogError::MonitorAlreadyRunning(_) => "monitor_already_running",
        OpenDogError::DaemonUnavailable => "daemon_unavailable",
        OpenDogError::DaemonControlUnavailable => "daemon_control_unavailable",
        OpenDogError::DaemonResponseIntegrity(_) => "daemon_response_integrity_error",
        OpenDogError::RemoteControl(_) => "remote_control_error",
        OpenDogError::Mcp(_) => "mcp_error",
        OpenDogError::LockPoisoned(_) => "lock_poisoned",
        OpenDogError::Database(_) => "database_error",
        OpenDogError::Io(_) => "io_error",
        OpenDogError::Serialization(_) => "serialization_error",
        OpenDogError::SchemaMigration(_) => "schema_migration_error",
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
        OpenDogError::DaemonResponseIntegrity(_) => Some(json!({
            "socket_path": crate::config::daemon_socket_path().display().to_string(),
            "suggested_actions": [
                "Retry the request once to rule out a transient socket interruption.",
                "Use the equivalent CLI command to confirm core business logic still succeeds.",
                "Restart `opendog daemon` if repeated daemon-backed MCP calls return incomplete responses."
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn error_code_project_exists() {
        let err = OpenDogError::ProjectExists("demo".into());
        assert_eq!(open_dog_error_code(&err), "project_exists");
    }

    #[test]
    fn error_code_project_not_found() {
        let err = OpenDogError::ProjectNotFound("demo".into());
        assert_eq!(open_dog_error_code(&err), "project_not_found");
    }

    #[test]
    fn error_code_invalid_project_id() {
        let err = OpenDogError::InvalidProjectId("bad!".into());
        assert_eq!(open_dog_error_code(&err), "invalid_project_id");
    }

    #[test]
    fn error_code_invalid_path() {
        let err = OpenDogError::InvalidPath("/nope".into());
        assert_eq!(open_dog_error_code(&err), "invalid_path");
    }

    #[test]
    fn error_code_invalid_verification() {
        let err = OpenDogError::InvalidVerification("bad".into());
        assert_eq!(open_dog_error_code(&err), "invalid_verification");
    }

    #[test]
    fn error_code_verification_record_missing() {
        let err = OpenDogError::VerificationRecordMissing("x".into());
        assert_eq!(open_dog_error_code(&err), "verification_record_missing");
    }

    #[test]
    fn error_code_governance_lane_not_found() {
        let err = OpenDogError::GovernanceLaneNotFound("lane-1".into());
        assert_eq!(open_dog_error_code(&err), "governance_lane_not_found");
    }

    #[test]
    fn error_code_governance_node_state_required() {
        let err = OpenDogError::GovernanceNodeStateRequired("node-1".into());
        assert_eq!(open_dog_error_code(&err), "governance_node_state_required");
    }

    #[test]
    fn error_code_invalid_input() {
        let err = OpenDogError::InvalidInput("wat".into());
        assert_eq!(open_dog_error_code(&err), "invalid_input");
    }

    #[test]
    fn error_code_daemon_already_running() {
        let err = OpenDogError::DaemonAlreadyRunning("123".into());
        assert_eq!(open_dog_error_code(&err), "daemon_already_running");
    }

    #[test]
    fn error_code_monitor_already_running() {
        let err = OpenDogError::MonitorAlreadyRunning("demo".into());
        assert_eq!(open_dog_error_code(&err), "monitor_already_running");
    }

    #[test]
    fn error_code_daemon_unavailable() {
        assert_eq!(open_dog_error_code(&OpenDogError::DaemonUnavailable), "daemon_unavailable");
    }

    #[test]
    fn error_code_daemon_control_unavailable() {
        assert_eq!(open_dog_error_code(&OpenDogError::DaemonControlUnavailable), "daemon_control_unavailable");
    }

    #[test]
    fn error_code_daemon_response_integrity() {
        let err = OpenDogError::DaemonResponseIntegrity("truncated".into());
        assert_eq!(open_dog_error_code(&err), "daemon_response_integrity_error");
    }

    #[test]
    fn error_code_remote_control() {
        let err = OpenDogError::RemoteControl("fail".into());
        assert_eq!(open_dog_error_code(&err), "remote_control_error");
    }

    #[test]
    fn error_code_mcp() {
        let err = OpenDogError::Mcp("oops".into());
        assert_eq!(open_dog_error_code(&err), "mcp_error");
    }

    #[test]
    fn error_code_lock_poisoned() {
        let err = OpenDogError::LockPoisoned("state".into());
        assert_eq!(open_dog_error_code(&err), "lock_poisoned");
    }

    #[test]
    fn error_code_database() {
        let err = OpenDogError::Database(rusqlite::Error::InvalidParameterName("x".into()));
        assert_eq!(open_dog_error_code(&err), "database_error");
    }

    #[test]
    fn error_code_io() {
        let err = OpenDogError::Io(io::Error::new(io::ErrorKind::NotFound, "gone"));
        assert_eq!(open_dog_error_code(&err), "io_error");
    }

    #[test]
    fn error_code_serialization() {
        let err = OpenDogError::Serialization(serde_json::from_str::<()>("bad").unwrap_err());
        assert_eq!(open_dog_error_code(&err), "serialization_error");
    }

    #[test]
    fn error_code_schema_migration() {
        let err = OpenDogError::SchemaMigration("v2".into());
        assert_eq!(open_dog_error_code(&err), "schema_migration_error");
    }

    #[test]
    fn error_code_walk() {
        let walk_err = walkdir::WalkDir::new("/nonexistent_opendog_test_path_xyz")
            .into_iter()
            .find_map(|r| r.err())
            .expect("walking a nonexistent dir should produce an error");
        let err = OpenDogError::Walk(walk_err);
        assert_eq!(open_dog_error_code(&err), "walk_error");
    }

    // validation_error_json tests

    #[test]
    fn validation_error_json_without_project() {
        let Json(value) = validation_error_json("v1", None, "bad_request", "missing field");
        assert_eq!(value["schema_version"], "v1");
        assert_eq!(value["status"], "error");
        assert_eq!(value["error_code"], "bad_request");
        assert_eq!(value["error"], "missing field");
        assert!(value.get("project_id").is_none());
    }

    #[test]
    fn validation_error_json_with_project() {
        let Json(value) = validation_error_json("v1", Some("demo"), "bad_request", "missing field");
        assert_eq!(value["schema_version"], "v1");
        assert_eq!(value["project_id"], "demo");
        assert_eq!(value["status"], "error");
        assert_eq!(value["error_code"], "bad_request");
        assert_eq!(value["error"], "missing field");
    }

    #[test]
    fn validation_error_json_carries_no_extra_fields() {
        let Json(value) = validation_error_json("v1", None, "err", "msg");
        assert!(value.get("remediation").is_none());
    }
}
