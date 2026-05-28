use opendog::config::{ConfigPatch, ProjectConfigPatch};

use crate::common::{ensure_dir, setup_manager};

#[test]
fn test_effective_project_config_uses_global_defaults_and_project_overrides() {
    let (dir, mgr) = setup_manager();
    let project_dir = dir.path().join("myproject");
    ensure_dir(&project_dir);

    mgr.update_global_config(ConfigPatch {
        ignore_patterns: Some(vec!["global-cache".to_string(), "dist".to_string()]),
        process_whitelist: Some(vec!["codex".to_string(), "claude".to_string()]),
        ..Default::default()
    })
    .unwrap();

    mgr.create("cfg-test", &project_dir).unwrap();

    let baseline = mgr.effective_project_config("cfg-test").unwrap();
    assert_eq!(
        baseline.ignore_patterns,
        vec!["global-cache".to_string(), "dist".to_string()]
    );
    assert_eq!(
        baseline.process_whitelist,
        vec!["codex".to_string(), "claude".to_string()]
    );

    let updated = mgr
        .update_project_config(
            "cfg-test",
            ProjectConfigPatch {
                ignore_patterns: Some(vec!["logs".to_string(), "tmp".to_string()]),
                process_whitelist: None,
                inherit_ignore_patterns: false,
                inherit_process_whitelist: false,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        updated.project_overrides.ignore_patterns,
        Some(vec!["logs".to_string(), "tmp".to_string()])
    );
    assert_eq!(
        updated.effective.ignore_patterns,
        vec!["logs".to_string(), "tmp".to_string()]
    );
    assert_eq!(
        updated.effective.process_whitelist,
        vec!["codex".to_string(), "claude".to_string()]
    );

    let inherited = mgr
        .update_project_config(
            "cfg-test",
            ProjectConfigPatch {
                ignore_patterns: None,
                process_whitelist: None,
                inherit_ignore_patterns: true,
                inherit_process_whitelist: false,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(inherited.project_overrides.ignore_patterns, None);
    assert_eq!(
        inherited.effective.ignore_patterns,
        vec!["global-cache".to_string(), "dist".to_string()]
    );
}

#[test]
fn test_project_incremental_process_updates_materialize_inherited_defaults() {
    let (dir, mgr) = setup_manager();
    let project_dir = dir.path().join("myproject");
    ensure_dir(&project_dir);

    mgr.update_global_config(ConfigPatch {
        process_whitelist: Some(vec!["claude".to_string(), "codex".to_string()]),
        ..Default::default()
    })
    .unwrap();

    mgr.create("cfg-incremental", &project_dir).unwrap();

    let updated = mgr
        .update_project_config(
            "cfg-incremental",
            ProjectConfigPatch {
                remove_process_whitelist: vec!["claude".to_string()],
                add_process_whitelist: vec!["roo".to_string()],
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(
        updated.project_overrides.process_whitelist,
        Some(vec!["codex".to_string(), "roo".to_string()])
    );
    assert_eq!(
        updated.effective.process_whitelist,
        vec!["codex".to_string(), "roo".to_string()]
    );
}
