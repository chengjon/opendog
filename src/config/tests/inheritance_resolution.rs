use super::*;

#[test]
fn project_config_patch_can_restore_global_inheritance() {
    let current = ProjectConfigOverrides {
        ignore_patterns: Some(vec!["logs".to_string()]),
        process_whitelist: Some(vec!["codex".to_string()]),
        ..Default::default()
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
            ..Default::default()
        },
    );
    assert_eq!(resolved.ignore_patterns, vec!["logs".to_string()]);
    assert_eq!(resolved.process_whitelist, global.process_whitelist);
}

#[test]
fn resolve_project_config_prefers_retention_policy_override() {
    let global = ProjectConfig::default();
    let override_policy = RetentionPolicy {
        activity_rows_threshold: 42,
        activity_retention_days: 7,
        ..Default::default()
    };
    let resolved = resolve_project_config(
        &global,
        &ProjectConfigOverrides {
            retention: Some(override_policy.clone()),
            ..Default::default()
        },
    );

    assert_eq!(resolved.retention, override_policy);
    assert_ne!(
        resolved.retention.activity_rows_threshold,
        global.retention.activity_rows_threshold
    );
}

#[test]
fn project_config_patch_can_restore_retention_inheritance() {
    let global = ProjectConfig::default();
    let current = ProjectConfigOverrides {
        retention: Some(RetentionPolicy {
            snapshot_runs_threshold: 12,
            ..Default::default()
        }),
        ..Default::default()
    };

    let updated = apply_project_config_patch(
        &current,
        &global,
        ProjectConfigPatch {
            inherit_retention: true,
            ..Default::default()
        },
    );

    assert_eq!(updated.retention, None);
}
