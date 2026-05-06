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
        let normalized = self.clone().normalized();
        normalized.ignore_patterns.is_none()
            && normalized.process_whitelist.is_none()
            && normalized.add_ignore_patterns.is_empty()
            && normalized.remove_ignore_patterns.is_empty()
            && normalized.add_process_whitelist.is_empty()
            && normalized.remove_process_whitelist.is_empty()
    }

    pub fn normalized(mut self) -> Self {
        if let Some(ignore_patterns) = self.ignore_patterns.take() {
            self.ignore_patterns = normalize_optional_replacement_list(ignore_patterns);
        }
        if let Some(process_whitelist) = self.process_whitelist.take() {
            self.process_whitelist = normalize_optional_replacement_list(process_whitelist);
        }
        self.add_ignore_patterns = normalize_string_list(self.add_ignore_patterns);
        self.remove_ignore_patterns = normalize_string_list(self.remove_ignore_patterns);
        self.add_process_whitelist = normalize_string_list(self.add_process_whitelist);
        self.remove_process_whitelist = normalize_string_list(self.remove_process_whitelist);
        self
    }
}

impl ProjectConfigPatch {
    pub fn is_empty(&self) -> bool {
        let normalized = self.clone().normalized();
        normalized.ignore_patterns.is_none()
            && normalized.process_whitelist.is_none()
            && normalized.add_ignore_patterns.is_empty()
            && normalized.remove_ignore_patterns.is_empty()
            && normalized.add_process_whitelist.is_empty()
            && normalized.remove_process_whitelist.is_empty()
            && !normalized.inherit_ignore_patterns
            && !normalized.inherit_process_whitelist
    }

    pub fn normalized(mut self) -> Self {
        if let Some(ignore_patterns) = self.ignore_patterns.take() {
            self.ignore_patterns = normalize_optional_replacement_list(ignore_patterns);
        }
        if let Some(process_whitelist) = self.process_whitelist.take() {
            self.process_whitelist = normalize_optional_replacement_list(process_whitelist);
        }
        self.add_ignore_patterns = normalize_string_list(self.add_ignore_patterns);
        self.remove_ignore_patterns = normalize_string_list(self.remove_ignore_patterns);
        self.add_process_whitelist = normalize_string_list(self.add_process_whitelist);
        self.remove_process_whitelist = normalize_string_list(self.remove_process_whitelist);
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
        ignore_patterns: apply_list_patch(
            &current.ignore_patterns,
            patch.ignore_patterns,
            patch.add_ignore_patterns,
            patch.remove_ignore_patterns,
        ),
        process_whitelist: apply_list_patch(
            &current.process_whitelist,
            patch.process_whitelist,
            patch.add_process_whitelist,
            patch.remove_process_whitelist,
        ),
    }
}

pub fn apply_project_config_patch(
    current: &ProjectConfigOverrides,
    effective: &ProjectConfig,
    patch: ProjectConfigPatch,
) -> ProjectConfigOverrides {
    let patch = patch.normalized();
    ProjectConfigOverrides {
        ignore_patterns: apply_project_list_patch(
            &current.ignore_patterns,
            &effective.ignore_patterns,
            patch.ignore_patterns,
            patch.add_ignore_patterns,
            patch.remove_ignore_patterns,
            patch.inherit_ignore_patterns,
        ),
        process_whitelist: apply_project_list_patch(
            &current.process_whitelist,
            &effective.process_whitelist,
            patch.process_whitelist,
            patch.add_process_whitelist,
            patch.remove_process_whitelist,
            patch.inherit_process_whitelist,
        ),
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

fn normalize_optional_replacement_list(values: Vec<String>) -> Option<Vec<String>> {
    if values.is_empty() {
        return Some(Vec::new());
    }

    let normalized = normalize_string_list(values);
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn apply_project_list_patch(
    current_override: &Option<Vec<String>>,
    effective: &[String],
    replacement: Option<Vec<String>>,
    add: Vec<String>,
    remove: Vec<String>,
    inherit: bool,
) -> Option<Vec<String>> {
    if inherit {
        return None;
    }

    if let Some(replacement) = replacement {
        return Some(replacement);
    }

    if add.is_empty() && remove.is_empty() {
        return current_override.clone();
    }

    let base = current_override.as_deref().unwrap_or(effective);
    let patched = apply_list_patch(base, None, add, remove);
    if current_override.is_none() && patched == effective {
        None
    } else {
        Some(patched)
    }
}

fn apply_list_patch(
    current: &[String],
    replacement: Option<Vec<String>>,
    add: Vec<String>,
    remove: Vec<String>,
) -> Vec<String> {
    let mut values = replacement.unwrap_or_else(|| current.to_vec());
    values = normalize_string_list(values);

    if !remove.is_empty() {
        let remove: HashSet<_> = remove.into_iter().collect();
        values.retain(|value| !remove.contains(value));
    }

    if !add.is_empty() {
        let mut seen: HashSet<_> = values.iter().cloned().collect();
        for value in add {
            if seen.insert(value.clone()) {
                values.push(value);
            }
        }
    }

    values
}
