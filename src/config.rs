use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "target",
    "__pycache__",
    ".cache",
    "build",
    ".next",
    ".nuxt",
    "vendor",
    ".venv",
    "venv",
    ".tox",
    ".mypy_cache",
    ".pytest_cache",
    ".gradle",
    ".idea",
    ".vscode",
    "*.pyc",
    ".DS_Store",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub ignore_patterns: Vec<String>,
    #[serde(default = "default_process_whitelist")]
    pub process_whitelist: Vec<String>,
}

fn default_process_whitelist() -> Vec<String> {
    vec![
        "claude".to_string(),
        "codex".to_string(),
        "node".to_string(),
        "python".to_string(),
        "python3".to_string(),
        "gpt".to_string(),
        "glm".to_string(),
    ]
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            ignore_patterns: DEFAULT_IGNORE_PATTERNS.iter().map(|s| s.to_string()).collect(),
            process_whitelist: default_process_whitelist(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub root_path: PathBuf,
    pub db_path: PathBuf,
    pub config: ProjectConfig,
    pub created_at: String,
    pub status: String,
}

pub fn data_dir() -> PathBuf {
    let base = dirs().join("data");
    base
}

pub fn registry_path() -> PathBuf {
    dirs().join("registry.db")
}

pub fn project_db_path(project_id: &str) -> PathBuf {
    dirs().join("projects").join(format!("{}.db", project_id))
}

fn dirs() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".opendog")
}

pub fn validate_project_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

pub fn validate_root_path(path: &Path) -> bool {
    path.is_absolute() && path.is_dir()
}
