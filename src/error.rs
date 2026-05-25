use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenDogError {
    #[error("Project '{0}' already exists")]
    ProjectExists(String),

    #[error("Project '{0}' not found")]
    ProjectNotFound(String),

    #[error("Invalid project ID: {0}")]
    InvalidProjectId(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Invalid verification input: {0}")]
    InvalidVerification(String),

    #[error("Verification record missing after insert: {0}")]
    VerificationRecordMissing(String),

    #[error("Governance lane '{0}' not found")]
    GovernanceLaneNotFound(String),

    #[error("Governance node state is required on create: {0}")]
    GovernanceNodeStateRequired(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Daemon already running: {0}")]
    DaemonAlreadyRunning(String),

    #[error("Monitor already running: {0}")]
    MonitorAlreadyRunning(String),

    #[error("Daemon control socket unavailable")]
    DaemonUnavailable,

    #[error("Daemon appears to be running but the control socket is unavailable")]
    DaemonControlUnavailable,

    #[error("Daemon IPC response integrity error: {0}")]
    DaemonResponseIntegrity(String),

    #[error("Remote control error: {0}")]
    RemoteControl(String),

    #[error("MCP error: {0}")]
    Mcp(String),

    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Schema migration error: {0}")]
    SchemaMigration(String),

    #[error("Walk error: {0}")]
    Walk(#[from] walkdir::Error),
}

pub type Result<T> = std::result::Result<T, OpenDogError>;

#[cfg(test)]
mod tests {
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
}
