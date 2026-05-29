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
    let result =
        super::apply_project_list_patch(&current, &["b".to_string()], None, vec![], vec![], false);
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
