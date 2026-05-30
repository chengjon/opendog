use super::{
    apply_global_config_patch, apply_project_config_patch, changed_config_fields,
    matches_ignore_pattern, resolve_project_config, ConfigPatch, ProjectConfig,
    ProjectConfigOverrides, ProjectConfigPatch, RetentionPolicy,
};

#[path = "tests/change_detection.rs"]
mod change_detection;
#[path = "tests/global_patch.rs"]
mod global_patch;
#[path = "tests/incremental_and_ignore.rs"]
mod incremental_and_ignore;
#[path = "tests/inheritance_resolution.rs"]
mod inheritance_resolution;
#[path = "tests/project_patch.rs"]
mod project_patch;
