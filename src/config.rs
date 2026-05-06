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
pub struct ProjectConfig {
    pub ignore_patterns: Vec<String>,
    #[serde(default = "default_process_whitelist")]
    pub process_whitelist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProjectConfigOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_patterns: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_whitelist: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ConfigPatch {
    #[serde(default)]
    pub ignore_patterns: Option<Vec<String>>,
    #[serde(default)]
    pub process_whitelist: Option<Vec<String>>,
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

#[cfg(test)]
mod tests {
    use super::{
        apply_global_config_patch, apply_project_config_patch, changed_config_fields,
        matches_ignore_pattern, resolve_project_config, ConfigPatch, ProjectConfig,
        ProjectConfigOverrides, ProjectConfigPatch,
    };

    #[test]
    fn project_config_patch_can_restore_global_inheritance() {
        let current = ProjectConfigOverrides {
            ignore_patterns: Some(vec!["logs".to_string()]),
            process_whitelist: Some(vec!["codex".to_string()]),
        };

        let updated = apply_project_config_patch(
            &current,
            &ProjectConfig::default(),
            ProjectConfigPatch {
                ignore_patterns: None,
                process_whitelist: None,
                inherit_ignore_patterns: true,
                inherit_process_whitelist: false,
                ..Default::default()
            },
        );

        assert_eq!(updated.ignore_patterns, None);
        assert_eq!(updated.process_whitelist, Some(vec!["codex".to_string()]));
    }

    #[test]
    fn resolve_project_config_prefers_project_overrides() {
        let global = ProjectConfig::default();
        let resolved = resolve_project_config(
            &global,
            &ProjectConfigOverrides {
                ignore_patterns: Some(vec!["logs".to_string()]),
                process_whitelist: None,
            },
        );
        assert_eq!(resolved.ignore_patterns, vec!["logs".to_string()]);
        assert_eq!(resolved.process_whitelist, global.process_whitelist);
    }

    #[test]
    fn changed_config_fields_reports_only_real_differences() {
        let before = ProjectConfig::default();
        let after = ProjectConfig {
            ignore_patterns: vec!["logs".to_string()],
            process_whitelist: before.process_whitelist.clone(),
        };
        assert_eq!(
            changed_config_fields(&before, &after),
            vec!["ignore_patterns".to_string()]
        );
    }

    #[test]
    fn config_patch_empty_detection_is_precise() {
        assert!(ConfigPatch::default().is_empty());
        assert!(ProjectConfigPatch::default().is_empty());
    }

    #[test]
    fn config_patch_empty_detection_counts_incremental_fields() {
        assert!(!ConfigPatch {
            add_ignore_patterns: vec!["logs".to_string()],
            ..Default::default()
        }
        .is_empty());
        assert!(!ProjectConfigPatch {
            remove_process_whitelist: vec!["claude".to_string()],
            ..Default::default()
        }
        .is_empty());
    }

    #[test]
    fn config_patch_explicit_empty_replacement_is_not_empty() {
        assert!(!ConfigPatch {
            ignore_patterns: Some(vec![]),
            ..Default::default()
        }
        .is_empty());
    }

    #[test]
    fn apply_global_config_patch_preserves_explicit_empty_replacement() {
        let current = ProjectConfig::default();

        let updated = apply_global_config_patch(
            &current,
            ConfigPatch {
                ignore_patterns: Some(vec![]),
                ..Default::default()
            },
        );

        assert_eq!(updated.ignore_patterns, Vec::<String>::new());
        assert_eq!(updated.process_whitelist, current.process_whitelist);
    }

    #[test]
    fn project_config_patch_explicit_empty_replacement_is_not_empty() {
        assert!(!ProjectConfigPatch {
            process_whitelist: Some(vec![]),
            ..Default::default()
        }
        .is_empty());
    }

    #[test]
    fn apply_project_config_patch_preserves_explicit_empty_override() {
        let global = ProjectConfig::default();
        let current = ProjectConfigOverrides::default();

        let updated = apply_project_config_patch(
            &current,
            &global,
            ProjectConfigPatch {
                process_whitelist: Some(vec![]),
                ..Default::default()
            },
        );

        assert_eq!(updated.process_whitelist, Some(Vec::<String>::new()));
        assert_eq!(updated.ignore_patterns, current.ignore_patterns);
    }

    #[test]
    fn config_patch_whitespace_only_values_are_empty_after_normalization() {
        assert!(ConfigPatch {
            ignore_patterns: Some(vec!["   ".to_string()]),
            add_ignore_patterns: vec!["   ".to_string()],
            ..Default::default()
        }
        .is_empty());
        assert!(ProjectConfigPatch {
            process_whitelist: Some(vec!["   ".to_string()]),
            remove_process_whitelist: vec!["   ".to_string()],
            ..Default::default()
        }
        .is_empty());
    }

    #[test]
    fn config_patch_supports_incremental_add_and_remove() {
        let current = ProjectConfig {
            ignore_patterns: vec!["dist".to_string(), "target".to_string()],
            process_whitelist: vec!["claude".to_string(), "codex".to_string()],
        };

        let updated = apply_global_config_patch(
            &current,
            ConfigPatch {
                add_ignore_patterns: vec!["logs".to_string()],
                remove_ignore_patterns: vec!["dist".to_string()],
                add_process_whitelist: vec!["roo".to_string()],
                remove_process_whitelist: vec!["claude".to_string()],
                ..Default::default()
            },
        );

        assert_eq!(
            updated.ignore_patterns,
            vec!["target".to_string(), "logs".to_string()]
        );
        assert_eq!(
            updated.process_whitelist,
            vec!["codex".to_string(), "roo".to_string()]
        );
    }

    #[test]
    fn project_config_patch_supports_incremental_override_edits() {
        let current = ProjectConfigOverrides {
            ignore_patterns: Some(vec!["dist".to_string(), "target".to_string()]),
            process_whitelist: Some(vec!["claude".to_string(), "codex".to_string()]),
        };
        let effective = ProjectConfig {
            ignore_patterns: vec!["dist".to_string(), "target".to_string()],
            process_whitelist: vec!["claude".to_string(), "codex".to_string()],
        };

        let updated = apply_project_config_patch(
            &current,
            &effective,
            ProjectConfigPatch {
                add_ignore_patterns: vec!["logs".to_string()],
                remove_process_whitelist: vec!["claude".to_string()],
                ..Default::default()
            },
        );

        assert_eq!(
            updated.ignore_patterns,
            Some(vec![
                "dist".to_string(),
                "target".to_string(),
                "logs".to_string()
            ])
        );
        assert_eq!(updated.process_whitelist, Some(vec!["codex".to_string()]));
    }

    #[test]
    fn project_config_patch_keeps_inherited_field_unset_when_incremental_edit_is_noop() {
        let current = ProjectConfigOverrides {
            ignore_patterns: None,
            process_whitelist: None,
        };
        let effective = ProjectConfig {
            ignore_patterns: vec!["dist".to_string(), "target".to_string()],
            process_whitelist: vec!["claude".to_string(), "codex".to_string()],
        };

        let updated = apply_project_config_patch(
            &current,
            &effective,
            ProjectConfigPatch {
                add_process_whitelist: vec!["claude".to_string()],
                remove_ignore_patterns: vec!["missing".to_string()],
                ..Default::default()
            },
        );

        assert_eq!(updated.ignore_patterns, None);
        assert_eq!(updated.process_whitelist, None);
    }

    #[test]
    fn ignore_pattern_matching_supports_segments_and_wildcards() {
        assert!(matches_ignore_pattern("src/cache/app.rs", "cache"));
        assert!(matches_ignore_pattern("build/main.pyc", "*.pyc"));
        assert!(!matches_ignore_pattern("src/main.rs", "*.pyc"));
    }
}
