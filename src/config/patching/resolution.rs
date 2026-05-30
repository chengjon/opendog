use super::*;

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
