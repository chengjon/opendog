use std::collections::HashSet;

use super::{
    default_process_whitelist, ConfigPatch, ProjectConfig, ProjectConfigOverrides,
    ProjectConfigPatch, DEFAULT_IGNORE_PATTERNS,
};

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            ignore_patterns: DEFAULT_IGNORE_PATTERNS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            process_whitelist: default_process_whitelist(),
        }
    }
}

impl ConfigPatch {
    pub fn is_empty(&self) -> bool {
        self.ignore_patterns.is_none() && self.process_whitelist.is_none()
    }

    pub fn normalized(mut self) -> Self {
        if let Some(ignore_patterns) = self.ignore_patterns.take() {
            self.ignore_patterns = Some(normalize_string_list(ignore_patterns));
        }
        if let Some(process_whitelist) = self.process_whitelist.take() {
            self.process_whitelist = Some(normalize_string_list(process_whitelist));
        }
        self
    }
}

impl ProjectConfigPatch {
    pub fn is_empty(&self) -> bool {
        self.ignore_patterns.is_none()
            && self.process_whitelist.is_none()
            && !self.inherit_ignore_patterns
            && !self.inherit_process_whitelist
    }

    pub fn normalized(mut self) -> Self {
        if let Some(ignore_patterns) = self.ignore_patterns.take() {
            self.ignore_patterns = Some(normalize_string_list(ignore_patterns));
        }
        if let Some(process_whitelist) = self.process_whitelist.take() {
            self.process_whitelist = Some(normalize_string_list(process_whitelist));
        }
        self
    }
}

pub fn resolve_project_config(
    global_defaults: &ProjectConfig,
    overrides: &ProjectConfigOverrides,
) -> ProjectConfig {
    ProjectConfig {
        ignore_patterns: overrides
            .ignore_patterns
            .clone()
            .unwrap_or_else(|| global_defaults.ignore_patterns.clone()),
        process_whitelist: overrides
            .process_whitelist
            .clone()
            .unwrap_or_else(|| global_defaults.process_whitelist.clone()),
    }
}

pub fn apply_global_config_patch(current: &ProjectConfig, patch: ConfigPatch) -> ProjectConfig {
    let patch = patch.normalized();
    ProjectConfig {
        ignore_patterns: patch
            .ignore_patterns
            .unwrap_or_else(|| current.ignore_patterns.clone()),
        process_whitelist: patch
            .process_whitelist
            .unwrap_or_else(|| current.process_whitelist.clone()),
    }
}

pub fn apply_project_config_patch(
    current: &ProjectConfigOverrides,
    patch: ProjectConfigPatch,
) -> ProjectConfigOverrides {
    let patch = patch.normalized();
    let ignore_patterns = if patch.inherit_ignore_patterns {
        None
    } else {
        patch
            .ignore_patterns
            .or_else(|| current.ignore_patterns.clone())
    };
    let process_whitelist = if patch.inherit_process_whitelist {
        None
    } else {
        patch
            .process_whitelist
            .or_else(|| current.process_whitelist.clone())
    };

    ProjectConfigOverrides {
        ignore_patterns,
        process_whitelist,
    }
}

pub fn changed_config_fields(before: &ProjectConfig, after: &ProjectConfig) -> Vec<String> {
    let mut changed = Vec::new();
    if before.ignore_patterns != after.ignore_patterns {
        changed.push("ignore_patterns".to_string());
    }
    if before.process_whitelist != after.process_whitelist {
        changed.push("process_whitelist".to_string());
    }
    changed
}

pub fn normalize_project_config(config: ProjectConfig) -> ProjectConfig {
    ProjectConfig {
        ignore_patterns: normalize_string_list(config.ignore_patterns),
        process_whitelist: normalize_string_list(config.process_whitelist),
    }
}

pub fn normalize_project_overrides(overrides: ProjectConfigOverrides) -> ProjectConfigOverrides {
    ProjectConfigOverrides {
        ignore_patterns: overrides.ignore_patterns.map(normalize_string_list),
        process_whitelist: overrides.process_whitelist.map(normalize_string_list),
    }
}

pub fn normalize_string_list(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            normalized.push(trimmed.to_string());
        }
    }

    normalized
}
