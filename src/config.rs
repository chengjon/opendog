mod ignore;
mod io;
mod patching;
mod paths;
mod validation;

pub use self::ignore::{matches_ignore_pattern, should_ignore_path};
pub use self::io::{
    load_global_config, load_global_config_from_path, save_global_config,
    save_global_config_to_path,
};
pub use self::patching::{
    apply_global_config_patch, apply_project_config_patch, changed_config_fields,
    normalize_project_config, normalize_project_overrides, normalize_string_list,
    resolve_project_config,
};
pub use self::paths::{
    daemon_pid_is_live, daemon_pid_path, daemon_socket_path, data_dir, global_config_path,
    project_db_path, registry_path,
};
pub use self::validation::{is_windows_mount_path, validate_project_id, validate_root_path};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetentionPolicy {
    #[serde(default = "default_cleanup_review_db_bytes_threshold")]
    pub cleanup_review_db_bytes_threshold: i64,
    #[serde(default = "default_vacuum_reclaimable_bytes_threshold")]
    pub vacuum_reclaimable_bytes_threshold: i64,
    #[serde(default = "default_vacuum_reclaim_ratio_threshold_percent")]
    pub vacuum_reclaim_ratio_threshold_percent: i64,
    #[serde(default = "default_activity_rows_threshold")]
    pub activity_rows_threshold: i64,
    #[serde(default = "default_verification_runs_threshold")]
    pub verification_runs_threshold: i64,
    #[serde(default = "default_snapshot_runs_threshold")]
    pub snapshot_runs_threshold: i64,
    #[serde(default = "default_activity_retention_days")]
    pub activity_retention_days: i64,
    #[serde(default = "default_verification_retention_days")]
    pub verification_retention_days: i64,
    #[serde(default = "default_keep_snapshot_runs")]
    pub keep_snapshot_runs: i64,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            cleanup_review_db_bytes_threshold: default_cleanup_review_db_bytes_threshold(),
            vacuum_reclaimable_bytes_threshold: default_vacuum_reclaimable_bytes_threshold(),
            vacuum_reclaim_ratio_threshold_percent: default_vacuum_reclaim_ratio_threshold_percent(
            ),
            activity_rows_threshold: default_activity_rows_threshold(),
            verification_runs_threshold: default_verification_runs_threshold(),
            snapshot_runs_threshold: default_snapshot_runs_threshold(),
            activity_retention_days: default_activity_retention_days(),
            verification_retention_days: default_verification_retention_days(),
            keep_snapshot_runs: default_keep_snapshot_runs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectConfig {
    pub ignore_patterns: Vec<String>,
    #[serde(default = "default_process_whitelist")]
    pub process_whitelist: Vec<String>,
    #[serde(default)]
    pub retention: RetentionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProjectConfigOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_patterns: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_whitelist: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention: Option<RetentionPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ConfigPatch {
    #[serde(default)]
    pub ignore_patterns: Option<Vec<String>>,
    #[serde(default)]
    pub process_whitelist: Option<Vec<String>>,
    #[serde(default)]
    pub retention: Option<RetentionPolicy>,
    #[serde(default)]
    pub add_ignore_patterns: Vec<String>,
    #[serde(default)]
    pub remove_ignore_patterns: Vec<String>,
    #[serde(default)]
    pub add_process_whitelist: Vec<String>,
    #[serde(default)]
    pub remove_process_whitelist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProjectConfigPatch {
    #[serde(default)]
    pub ignore_patterns: Option<Vec<String>>,
    #[serde(default)]
    pub process_whitelist: Option<Vec<String>>,
    #[serde(default)]
    pub retention: Option<RetentionPolicy>,
    #[serde(default)]
    pub add_ignore_patterns: Vec<String>,
    #[serde(default)]
    pub remove_ignore_patterns: Vec<String>,
    #[serde(default)]
    pub add_process_whitelist: Vec<String>,
    #[serde(default)]
    pub remove_process_whitelist: Vec<String>,
    #[serde(default)]
    pub inherit_ignore_patterns: bool,
    #[serde(default)]
    pub inherit_process_whitelist: bool,
    #[serde(default)]
    pub inherit_retention: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectConfigView {
    pub project_id: String,
    pub global_defaults: ProjectConfig,
    pub project_overrides: ProjectConfigOverrides,
    pub effective: ProjectConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProjectConfigReload {
    pub monitor_running: bool,
    pub runtime_reloaded: bool,
    pub snapshot_refreshed: bool,
    #[serde(default)]
    pub changed_fields: Vec<String>,
    #[serde(default)]
    pub skipped_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectConfigUpdateResult {
    pub project_id: String,
    pub global_defaults: ProjectConfig,
    pub project_overrides: ProjectConfigOverrides,
    pub effective: ProjectConfig,
    pub reload: ProjectConfigReload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectReloadStatus {
    pub project_id: String,
    pub monitor_running: bool,
    pub runtime_reloaded: bool,
    pub snapshot_refreshed: bool,
    #[serde(default)]
    pub changed_fields: Vec<String>,
    #[serde(default)]
    pub skipped_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalConfigUpdateResult {
    pub global_defaults: ProjectConfig,
    pub reloaded_projects: Vec<ProjectReloadStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub root_path: PathBuf,
    pub db_path: PathBuf,
    pub config: ProjectConfigOverrides,
    pub created_at: String,
    pub status: String,
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

fn default_cleanup_review_db_bytes_threshold() -> i64 {
    16 * 1024 * 1024
}

fn default_vacuum_reclaimable_bytes_threshold() -> i64 {
    8 * 1024 * 1024
}

fn default_vacuum_reclaim_ratio_threshold_percent() -> i64 {
    20
}

fn default_activity_rows_threshold() -> i64 {
    1_000_000
}

fn default_verification_runs_threshold() -> i64 {
    10_000
}

fn default_snapshot_runs_threshold() -> i64 {
    100
}

fn default_activity_retention_days() -> i64 {
    30
}

fn default_verification_retention_days() -> i64 {
    60
}

fn default_keep_snapshot_runs() -> i64 {
    20
}

#[cfg(test)]
mod tests;
