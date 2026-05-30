use super::*;

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
