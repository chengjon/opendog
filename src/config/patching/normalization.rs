use std::collections::HashSet;

use super::*;

impl ConfigPatch {
    pub fn is_empty(&self) -> bool {
        let normalized = self.clone().normalized();
        normalized.ignore_patterns.is_none()
            && normalized.process_whitelist.is_none()
            && normalized.retention.is_none()
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
            && normalized.retention.is_none()
            && normalized.add_ignore_patterns.is_empty()
            && normalized.remove_ignore_patterns.is_empty()
            && normalized.add_process_whitelist.is_empty()
            && normalized.remove_process_whitelist.is_empty()
            && !normalized.inherit_ignore_patterns
            && !normalized.inherit_process_whitelist
            && !normalized.inherit_retention
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

pub fn normalize_project_config(config: ProjectConfig) -> ProjectConfig {
    ProjectConfig {
        ignore_patterns: normalize_string_list(config.ignore_patterns),
        process_whitelist: normalize_string_list(config.process_whitelist),
        retention: config.retention,
    }
}

pub fn normalize_project_overrides(overrides: ProjectConfigOverrides) -> ProjectConfigOverrides {
    ProjectConfigOverrides {
        ignore_patterns: overrides.ignore_patterns.map(normalize_string_list),
        process_whitelist: overrides.process_whitelist.map(normalize_string_list),
        retention: overrides.retention,
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
