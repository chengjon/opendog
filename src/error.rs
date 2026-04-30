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

    #[error("Remote control error: {0}")]
    RemoteControl(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Walk error: {0}")]
    Walk(#[from] walkdir::Error),
}

pub type Result<T> = std::result::Result<T, OpenDogError>;
