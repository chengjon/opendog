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
            retention: Default::default(),
        }
    }
}

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
        retention: overrides
            .retention
            .clone()
            .unwrap_or_else(|| global_defaults.retention.clone()),
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
        retention: patch.retention.unwrap_or_else(|| current.retention.clone()),
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
        retention: apply_project_retention_patch(
            &current.retention,
            patch.retention,
            patch.inherit_retention,
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
    if before.retention != after.retention {
        changed.push("retention".to_string());
    }
    changed
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

fn apply_project_retention_patch(
    current: &Option<super::RetentionPolicy>,
    replacement: Option<super::RetentionPolicy>,
    inherit: bool,
) -> Option<super::RetentionPolicy> {
    if inherit {
        None
    } else if let Some(replacement) = replacement {
        Some(replacement)
    } else {
        current.clone()
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- normalize_string_list ---

    #[test]
    fn normalize_string_list_removes_whitespace_entries() {
        let result = normalize_string_list(vec![
            "hello".to_string(),
            "   ".to_string(),
            "world".to_string(),
        ]);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn normalize_string_list_trims_values() {
        let result = normalize_string_list(vec!["  hello  ".to_string(), " world ".to_string()]);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn normalize_string_list_deduplicates() {
        let result = normalize_string_list(vec![
            "alpha".to_string(),
            "alpha".to_string(),
            "beta".to_string(),
        ]);
        assert_eq!(result, vec!["alpha", "beta"]);
    }

    #[test]
    fn normalize_string_list_empty_input() {
        let result = normalize_string_list(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn normalize_string_list_all_whitespace() {
        let result = normalize_string_list(vec!["  ".to_string(), "\t".to_string()]);
        assert!(result.is_empty());
    }

    // --- changed_config_fields ---

    #[test]
    fn changed_config_fields_no_changes() {
        let before = ProjectConfig::default();
        let after = before.clone();
        assert!(changed_config_fields(&before, &after).is_empty());
    }

    #[test]
    fn changed_config_fields_ignore_patterns_changed() {
        let before = ProjectConfig::default();
        let after = ProjectConfig {
            ignore_patterns: vec!["logs".to_string()],
            process_whitelist: before.process_whitelist.clone(),
            ..before.clone()
        };
        let changed = changed_config_fields(&before, &after);
        assert_eq!(changed, vec!["ignore_patterns".to_string()]);
    }

    #[test]
    fn changed_config_fields_process_whitelist_changed() {
        let before = ProjectConfig::default();
        let after = ProjectConfig {
            ignore_patterns: before.ignore_patterns.clone(),
            process_whitelist: vec!["gpt".to_string()],
            ..before.clone()
        };
        let changed = changed_config_fields(&before, &after);
        assert_eq!(changed, vec!["process_whitelist".to_string()]);
    }

    #[test]
    fn changed_config_fields_both_changed() {
        let before = ProjectConfig::default();
        let after = ProjectConfig {
            ignore_patterns: vec!["logs".to_string()],
            process_whitelist: vec!["gpt".to_string()],
            ..before.clone()
        };
        let changed = changed_config_fields(&before, &after);
        assert_eq!(changed.len(), 2);
        assert!(changed.contains(&"ignore_patterns".to_string()));
        assert!(changed.contains(&"process_whitelist".to_string()));
    }

    // --- apply_list_patch (private) ---

    #[test]
    fn apply_list_patch_no_changes() {
        let current = vec!["a".to_string(), "b".to_string()];
        let result = super::apply_list_patch(&current, None, vec![], vec![]);
        assert_eq!(result, vec!["a", "b"]);
    }

    #[test]
    fn apply_list_patch_with_replacement() {
        let current = vec!["a".to_string()];
        let result = super::apply_list_patch(
            &current,
            Some(vec!["x".to_string(), "y".to_string()]),
            vec![],
            vec![],
        );
        assert_eq!(result, vec!["x", "y"]);
    }

    #[test]
    fn apply_list_patch_add_items() {
        let current = vec!["a".to_string()];
        let result = super::apply_list_patch(&current, None, vec!["b".to_string()], vec![]);
        assert_eq!(result, vec!["a", "b"]);
    }

    #[test]
    fn apply_list_patch_add_does_not_duplicate() {
        let current = vec!["a".to_string()];
        let result = super::apply_list_patch(&current, None, vec!["a".to_string()], vec![]);
        assert_eq!(result, vec!["a"]);
    }

    #[test]
    fn apply_list_patch_remove_items() {
        let current = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = super::apply_list_patch(&current, None, vec![], vec!["b".to_string()]);
        assert_eq!(result, vec!["a", "c"]);
    }

    #[test]
    fn apply_list_patch_add_and_remove() {
        let current = vec!["a".to_string(), "b".to_string()];
        let result =
            super::apply_list_patch(&current, None, vec!["c".to_string()], vec!["a".to_string()]);
        assert_eq!(result, vec!["b", "c"]);
    }

    // --- apply_project_list_patch (private) ---

    #[test]
    fn apply_project_list_patch_inherit_returns_none() {
        let result = super::apply_project_list_patch(
            &Some(vec!["a".to_string()]),
            &["b".to_string()],
            None,
            vec![],
            vec![],
            true, // inherit
        );
        assert!(result.is_none());
    }

    #[test]
    fn apply_project_list_patch_replacement_overrides() {
        let result = super::apply_project_list_patch(
            &Some(vec!["a".to_string()]),
            &["b".to_string()],
            Some(vec!["x".to_string()]),
            vec![],
            vec![],
            false,
        );
        assert_eq!(result, Some(vec!["x".to_string()]));
    }

    #[test]
    fn apply_project_list_patch_no_op_returns_current() {
        let current = Some(vec!["a".to_string()]);
        let result = super::apply_project_list_patch(
            &current,
            &["b".to_string()],
            None,
            vec![],
            vec![],
            false,
        );
        assert_eq!(result, current);
    }

    #[test]
    fn apply_project_list_patch_incremental_edit_falls_back_to_effective() {
        let result = super::apply_project_list_patch(
            &None,
            &["a".to_string(), "b".to_string()],
            None,
            vec!["c".to_string()],
            vec!["a".to_string()],
            false,
        );
        // base = effective = ["a", "b"], remove "a", add "c" => ["b", "c"]
        assert_eq!(result, Some(vec!["b".to_string(), "c".to_string()]));
    }

    #[test]
    fn apply_project_list_patch_incremental_noop_on_inherited_returns_none() {
        let result = super::apply_project_list_patch(
            &None,
            &["a".to_string()],
            None,
            vec!["a".to_string()], // adding "a" which already exists
            vec![],
            false,
        );
        // base = effective = ["a"], add "a" (duplicate) => ["a"], same as effective => None
        assert!(result.is_none());
    }

    // --- resolve_project_config ---

    #[test]
    fn resolve_project_config_uses_overrides_when_set() {
        let global = ProjectConfig {
            ignore_patterns: vec!["dist".to_string()],
            process_whitelist: vec!["claude".to_string()],
            ..Default::default()
        };
        let overrides = ProjectConfigOverrides {
            ignore_patterns: Some(vec!["logs".to_string()]),
            process_whitelist: Some(vec!["gpt".to_string()]),
            ..Default::default()
        };
        let resolved = resolve_project_config(&global, &overrides);
        assert_eq!(resolved.ignore_patterns, vec!["logs"]);
        assert_eq!(resolved.process_whitelist, vec!["gpt"]);
    }

    #[test]
    fn resolve_project_config_falls_back_to_global() {
        let global = ProjectConfig {
            ignore_patterns: vec!["dist".to_string()],
            process_whitelist: vec!["claude".to_string()],
            ..Default::default()
        };
        let overrides = ProjectConfigOverrides {
            ignore_patterns: None,
            process_whitelist: None,
            ..Default::default()
        };
        let resolved = resolve_project_config(&global, &overrides);
        assert_eq!(resolved.ignore_patterns, global.ignore_patterns);
        assert_eq!(resolved.process_whitelist, global.process_whitelist);
    }

    #[test]
    fn resolve_project_config_mixed_overrides() {
        let global = ProjectConfig {
            ignore_patterns: vec!["dist".to_string()],
            process_whitelist: vec!["claude".to_string()],
            ..Default::default()
        };
        let overrides = ProjectConfigOverrides {
            ignore_patterns: Some(vec!["logs".to_string()]),
            process_whitelist: None,
            ..Default::default()
        };
        let resolved = resolve_project_config(&global, &overrides);
        assert_eq!(resolved.ignore_patterns, vec!["logs"]);
        assert_eq!(resolved.process_whitelist, global.process_whitelist);
    }

    // --- apply_global_config_patch ---

    #[test]
    fn apply_global_config_patch_empty_patch_is_noop() {
        let current = ProjectConfig::default();
        let result = apply_global_config_patch(&current, ConfigPatch::default());
        assert_eq!(result, current);
    }

    #[test]
    fn apply_global_config_patch_full_replacement() {
        let current = ProjectConfig::default();
        let result = apply_global_config_patch(
            &current,
            ConfigPatch {
                ignore_patterns: Some(vec!["custom".to_string()]),
                process_whitelist: Some(vec!["myproc".to_string()]),
                ..Default::default()
            },
        );
        assert_eq!(result.ignore_patterns, vec!["custom"]);
        assert_eq!(result.process_whitelist, vec!["myproc"]);
    }

    // --- apply_project_config_patch ---

    #[test]
    fn apply_project_config_patch_inherit_clears_override() {
        let current = ProjectConfigOverrides {
            ignore_patterns: Some(vec!["logs".to_string()]),
            process_whitelist: None,
            ..Default::default()
        };
        let effective = ProjectConfig::default();
        let result = apply_project_config_patch(
            &current,
            &effective,
            ProjectConfigPatch {
                inherit_ignore_patterns: true,
                ..Default::default()
            },
        );
        assert!(result.ignore_patterns.is_none());
    }

    #[test]
    fn apply_project_config_patch_replacement_sets_override() {
        let current = ProjectConfigOverrides::default();
        let effective = ProjectConfig::default();
        let result = apply_project_config_patch(
            &current,
            &effective,
            ProjectConfigPatch {
                ignore_patterns: Some(vec!["new_pattern".to_string()]),
                ..Default::default()
            },
        );
        assert_eq!(
            result.ignore_patterns,
            Some(vec!["new_pattern".to_string()])
        );
    }

    #[test]
    fn apply_project_config_patch_incremental_edit() {
        let current = ProjectConfigOverrides {
            ignore_patterns: Some(vec!["dist".to_string()]),
            process_whitelist: None,
            ..Default::default()
        };
        let effective = ProjectConfig {
            ignore_patterns: vec!["dist".to_string()],
            process_whitelist: vec!["claude".to_string()],
            ..Default::default()
        };
        let result = apply_project_config_patch(
            &current,
            &effective,
            ProjectConfigPatch {
                add_ignore_patterns: vec!["logs".to_string()],
                ..Default::default()
            },
        );
        assert_eq!(
            result.ignore_patterns,
            Some(vec!["dist".to_string(), "logs".to_string()])
        );
    }
}
