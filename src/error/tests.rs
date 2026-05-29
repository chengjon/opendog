use super::*;

#[test]
fn project_exists_display() {
    let err = OpenDogError::ProjectExists("myproj".to_string());
    assert!(err.to_string().contains("myproj"));
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn project_not_found_display() {
    let err = OpenDogError::ProjectNotFound("test".to_string());
    assert!(err.to_string().contains("test"));
    assert!(err.to_string().contains("not found"));
}

#[test]
fn invalid_project_id_display() {
    let err = OpenDogError::InvalidProjectId("bad id!".to_string());
    assert!(err.to_string().contains("bad id!"));
}

#[test]
fn invalid_input_display() {
    let err = OpenDogError::InvalidInput("some reason".to_string());
    assert!(err.to_string().contains("some reason"));
}

#[test]
fn daemon_unavailable_display() {
    let err = OpenDogError::DaemonUnavailable;
    assert!(err.to_string().contains("unavailable"));
}

#[test]
fn governance_lane_not_found_display() {
    let err = OpenDogError::GovernanceLaneNotFound("lane-1".to_string());
    assert!(err.to_string().contains("lane-1"));
}

#[test]
fn governance_node_state_required_display() {
    let err = OpenDogError::GovernanceNodeStateRequired("node-x".to_string());
    assert!(err.to_string().contains("node-x"));
    assert!(err.to_string().contains("required"));
}

#[test]
fn daemon_already_running_display() {
    let err = OpenDogError::DaemonAlreadyRunning("/tmp/opendog.pid".to_string());
    assert!(err.to_string().contains("/tmp/opendog.pid"));
}

#[test]
fn monitor_already_running_display() {
    let err = OpenDogError::MonitorAlreadyRunning("proj".to_string());
    assert!(err.to_string().contains("proj"));
}

#[test]
fn daemon_control_unavailable_display() {
    let err = OpenDogError::DaemonControlUnavailable;
    assert!(err.to_string().contains("control socket"));
}

#[test]
fn daemon_response_integrity_display() {
    let err = OpenDogError::DaemonResponseIntegrity("bad checksum".to_string());
    assert!(err.to_string().contains("bad checksum"));
}

#[test]
fn remote_control_display() {
    let err = OpenDogError::RemoteControl("timeout".to_string());
    assert!(err.to_string().contains("timeout"));
}

#[test]
fn mcp_error_display() {
    let err = OpenDogError::Mcp("tool not found".to_string());
    assert!(err.to_string().contains("tool not found"));
}

#[test]
fn lock_poisoned_display() {
    let err = OpenDogError::LockPoisoned("project_map".to_string());
    assert!(err.to_string().contains("project_map"));
}

#[test]
fn schema_migration_display() {
    let err = OpenDogError::SchemaMigration("version mismatch".to_string());
    assert!(err.to_string().contains("version mismatch"));
}

#[test]
fn from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let err: OpenDogError = io_err.into();
    assert!(err.to_string().contains("file missing"));
}

#[test]
fn from_database_error() {
    let db_err = rusqlite::Error::InvalidColumnIndex(99);
    let err: OpenDogError = db_err.into();
    assert!(err.to_string().contains("Invalid"));
}

#[test]
fn from_serialization_error() {
    let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
    let err: OpenDogError = json_err.into();
    assert!(err.to_string().contains("expected"));
}

#[test]
fn invalid_verification_display() {
    let err = OpenDogError::InvalidVerification("missing kind".to_string());
    assert!(err.to_string().contains("missing kind"));
}

#[test]
fn verification_record_missing_display() {
    let err = OpenDogError::VerificationRecordMissing("run-42".to_string());
    assert!(err.to_string().contains("run-42"));
}

#[test]
fn invalid_path_display() {
    let err = OpenDogError::InvalidPath("/mnt/c/forbidden".to_string());
    assert!(err.to_string().contains("/mnt/c/forbidden"));
}
