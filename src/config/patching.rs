use super::{
    ConfigPatch, ProjectConfig, ProjectConfigOverrides, ProjectConfigPatch, RetentionPolicy,
};

mod application;
mod defaults;
mod normalization;
mod resolution;

pub use self::application::{apply_global_config_patch, apply_project_config_patch};
#[cfg(test)]
use self::application::{apply_list_patch, apply_project_list_patch};
pub use self::normalization::{
    normalize_project_config, normalize_project_overrides, normalize_string_list,
};
pub use self::resolution::{changed_config_fields, resolve_project_config};

#[cfg(test)]
mod tests;
