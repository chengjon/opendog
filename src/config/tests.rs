use super::{
    apply_global_config_patch, apply_project_config_patch, changed_config_fields,
    matches_ignore_pattern, resolve_project_config, ConfigPatch, ProjectConfig,
    ProjectConfigOverrides, ProjectConfigPatch, RetentionPolicy,
};

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
fn changed_config_fields_reports_only_real_differences() {
    let before = ProjectConfig::default();
    let after = ProjectConfig {
        ignore_patterns: vec!["logs".to_string()],
        process_whitelist: before.process_whitelist.clone(),
        ..before.clone()
    };
    assert_eq!(
        changed_config_fields(&before, &after),
        vec!["ignore_patterns".to_string()]
    );
}

#[test]
fn changed_config_fields_reports_retention_policy_changes() {
    let before = ProjectConfig::default();
    let after = ProjectConfig {
        retention: RetentionPolicy {
            snapshot_runs_threshold: before.retention.snapshot_runs_threshold + 1,
            ..before.retention.clone()
        },
        ..before.clone()
    };

    assert_eq!(changed_config_fields(&before, &after), vec!["retention"]);
}

#[test]
fn retention_policy_defaults_are_storage_maintenance_defaults() {
    let policy = RetentionPolicy::default();

    assert_eq!(policy.cleanup_review_db_bytes_threshold, 16 * 1024 * 1024);
    assert_eq!(policy.vacuum_reclaimable_bytes_threshold, 8 * 1024 * 1024);
    assert_eq!(policy.vacuum_reclaim_ratio_threshold_percent, 20);
    assert_eq!(policy.activity_rows_threshold, 1_000_000);
    assert_eq!(policy.verification_runs_threshold, 10_000);
    assert_eq!(policy.snapshot_runs_threshold, 100);
    assert_eq!(policy.activity_retention_days, 30);
    assert_eq!(policy.verification_retention_days, 60);
    assert_eq!(policy.keep_snapshot_runs, 20);
}

#[test]
fn config_patch_empty_detection_is_precise() {
    assert!(ConfigPatch::default().is_empty());
    assert!(ProjectConfigPatch::default().is_empty());
}

#[test]
fn config_patch_empty_detection_counts_incremental_fields() {
    assert!(!ConfigPatch {
        add_ignore_patterns: vec!["logs".to_string()],
        ..Default::default()
    }
    .is_empty());
    assert!(!ConfigPatch {
        retention: Some(RetentionPolicy {
            activity_rows_threshold: 123,
            ..Default::default()
        }),
        ..Default::default()
    }
    .is_empty());
    assert!(!ProjectConfigPatch {
        remove_process_whitelist: vec!["claude".to_string()],
        ..Default::default()
    }
    .is_empty());
    assert!(!ProjectConfigPatch {
        inherit_retention: true,
        ..Default::default()
    }
    .is_empty());
}

#[test]
fn config_patch_explicit_empty_replacement_is_not_empty() {
    assert!(!ConfigPatch {
        ignore_patterns: Some(vec![]),
        ..Default::default()
    }
    .is_empty());
}

#[test]
fn apply_global_config_patch_preserves_explicit_empty_replacement() {
    let current = ProjectConfig::default();

    let updated = apply_global_config_patch(
        &current,
        ConfigPatch {
            ignore_patterns: Some(vec![]),
            ..Default::default()
        },
    );

    assert_eq!(updated.ignore_patterns, Vec::<String>::new());
    assert_eq!(updated.process_whitelist, current.process_whitelist);
}

#[test]
fn apply_global_config_patch_replaces_retention_policy() {
    let current = ProjectConfig::default();
    let retention = RetentionPolicy {
        activity_rows_threshold: 123,
        ..Default::default()
    };

    let updated = apply_global_config_patch(
        &current,
        ConfigPatch {
            retention: Some(retention.clone()),
            ..Default::default()
        },
    );

    assert_eq!(updated.retention, retention);
    assert_eq!(updated.ignore_patterns, current.ignore_patterns);
    assert_eq!(updated.process_whitelist, current.process_whitelist);
}

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

#[test]
fn config_patch_whitespace_only_values_are_empty_after_normalization() {
    assert!(ConfigPatch {
        ignore_patterns: Some(vec!["   ".to_string()]),
        add_ignore_patterns: vec!["   ".to_string()],
        ..Default::default()
    }
    .is_empty());
    assert!(ProjectConfigPatch {
        process_whitelist: Some(vec!["   ".to_string()]),
        remove_process_whitelist: vec!["   ".to_string()],
        ..Default::default()
    }
    .is_empty());
}

#[test]
fn config_patch_supports_incremental_add_and_remove() {
    let current = ProjectConfig {
        ignore_patterns: vec!["dist".to_string(), "target".to_string()],
        process_whitelist: vec!["claude".to_string(), "codex".to_string()],
        ..Default::default()
    };

    let updated = apply_global_config_patch(
        &current,
        ConfigPatch {
            add_ignore_patterns: vec!["logs".to_string()],
            remove_ignore_patterns: vec!["dist".to_string()],
            add_process_whitelist: vec!["roo".to_string()],
            remove_process_whitelist: vec!["claude".to_string()],
            ..Default::default()
        },
    );

    assert_eq!(
        updated.ignore_patterns,
        vec!["target".to_string(), "logs".to_string()]
    );
    assert_eq!(
        updated.process_whitelist,
        vec!["codex".to_string(), "roo".to_string()]
    );
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

#[test]
fn ignore_pattern_matching_supports_segments_and_wildcards() {
    assert!(matches_ignore_pattern("src/cache/app.rs", "cache"));
    assert!(matches_ignore_pattern("build/main.pyc", "*.pyc"));
    assert!(!matches_ignore_pattern("src/main.rs", "*.pyc"));
}
