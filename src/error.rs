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
