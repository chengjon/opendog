use super::*;

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
fn apply_project_config_patch_replaces_retention_override() {
    let global = ProjectConfig::default();
    let retention = RetentionPolicy {
        snapshot_runs_threshold: 12,
        ..Default::default()
    };

    let updated = apply_project_config_patch(
        &ProjectConfigOverrides::default(),
        &global,
        ProjectConfigPatch {
            retention: Some(retention.clone()),
            ..Default::default()
        },
    );

    assert_eq!(updated.retention, Some(retention));
}

#[test]
fn project_config_patch_supports_incremental_override_edits() {
    let current = ProjectConfigOverrides {
        ignore_patterns: Some(vec!["dist".to_string(), "target".to_string()]),
        process_whitelist: Some(vec!["claude".to_string(), "codex".to_string()]),
        ..Default::default()
    };
    let effective = ProjectConfig {
        ignore_patterns: vec!["dist".to_string(), "target".to_string()],
        process_whitelist: vec!["claude".to_string(), "codex".to_string()],
        ..Default::default()
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
        ..Default::default()
    };
    let effective = ProjectConfig {
        ignore_patterns: vec!["dist".to_string(), "target".to_string()],
        process_whitelist: vec!["claude".to_string(), "codex".to_string()],
        ..Default::default()
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
