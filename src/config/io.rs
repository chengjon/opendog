use std::path::Path;

use crate::error::Result;

use super::{global_config_path, normalize_project_config, ProjectConfig};

pub fn load_global_config() -> Result<ProjectConfig> {
    load_global_config_from_path(&global_config_path())
}

pub fn load_global_config_from_path(path: &Path) -> Result<ProjectConfig> {
    if !path.exists() {
        return Ok(ProjectConfig::default());
    }

    let contents = std::fs::read_to_string(path)?;
    let config = serde_json::from_str::<ProjectConfig>(&contents)?;
    Ok(normalize_project_config(config))
}

pub fn save_global_config(config: &ProjectConfig) -> Result<()> {
    save_global_config_to_path(&global_config_path(), config)
}

pub fn save_global_config_to_path(path: &Path, config: &ProjectConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let normalized = normalize_project_config(config.clone());
    std::fs::write(path, serde_json::to_string_pretty(&normalized)?)?;
    Ok(())
}
