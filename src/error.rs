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
