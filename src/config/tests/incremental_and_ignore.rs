use super::*;

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
fn ignore_pattern_matching_supports_segments_and_wildcards() {
    assert!(matches_ignore_pattern("src/cache/app.rs", "cache"));
    assert!(matches_ignore_pattern("build/main.pyc", "*.pyc"));
    assert!(!matches_ignore_pattern("src/main.rs", "*.pyc"));
}
