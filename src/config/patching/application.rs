use std::collections::HashSet;

use super::*;

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

pub(super) fn apply_project_list_patch(
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

pub(super) fn apply_list_patch(
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
