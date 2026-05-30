use super::*;

#[test]
fn update_project_config_reloads_running_monitor_state() {
    let (_dir, mut controller) = test_controller();
    controller.start_monitor("demo").unwrap();

    let update = controller
        .update_project_config(
            "demo",
            ProjectConfigPatch {
                ignore_patterns: Some(vec!["logs".to_string()]),
                process_whitelist: Some(vec!["codex".to_string()]),
                inherit_ignore_patterns: false,
                inherit_process_whitelist: false,
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(update.effective.ignore_patterns, vec!["logs".to_string()]);
    assert_eq!(
        update.reload.changed_fields,
        vec![
            "ignore_patterns".to_string(),
            "process_whitelist".to_string()
        ]
    );
    assert!(update.reload.monitor_running);
    assert!(update.reload.runtime_reloaded);
    assert!(update.reload.snapshot_refreshed);

    let handle = controller.monitors.get("demo").unwrap();
    let live = handle.current_config();
    assert_eq!(live.ignore_patterns, vec!["logs".to_string()]);
    assert_eq!(live.process_whitelist, vec!["codex".to_string()]);
    controller.stop_all();
}

#[test]
fn reload_project_config_picks_up_latest_global_defaults() {
    let (_dir, mut controller) = test_controller();
    controller.start_monitor("demo").unwrap();

    controller
        .project_manager()
        .update_global_config(crate::config::ConfigPatch {
            ignore_patterns: Some(vec!["generated".to_string()]),
            process_whitelist: Some(vec!["claude".to_string()]),
            ..Default::default()
        })
        .unwrap();

    let outcome = controller.reload_project_config("demo").unwrap();
    assert!(outcome.monitor_running);
    assert!(outcome.runtime_reloaded);
    assert_eq!(
        outcome.changed_fields,
        vec![
            "ignore_patterns".to_string(),
            "process_whitelist".to_string()
        ]
    );

    let handle = controller.monitors.get("demo").unwrap();
    let live = handle.current_config();
    assert_eq!(live.ignore_patterns, vec!["generated".to_string()]);
    assert_eq!(live.process_whitelist, vec!["claude".to_string()]);
    controller.stop_all();
}

#[test]
fn handle_request_applies_incremental_global_config_updates() {
    let (_dir, mut controller) = test_controller();

    controller
        .project_manager()
        .update_global_config(ConfigPatch {
            ignore_patterns: Some(vec!["node_modules".to_string()]),
            ..Default::default()
        })
        .unwrap();

    let response = controller.handle_request(ControlRequest::UpdateGlobalConfig(ConfigPatch {
        ignore_patterns: None,
        process_whitelist: Some(vec!["claude".to_string(), "codex".to_string()]),
        retention: None,
        add_ignore_patterns: vec!["logs".to_string()],
        remove_ignore_patterns: vec!["node_modules".to_string()],
        add_process_whitelist: vec!["roo".to_string()],
        remove_process_whitelist: vec!["claude".to_string()],
    }));

    match response {
        ControlResponse::GlobalConfigUpdated { result } => {
            assert_eq!(
                result.global_defaults.ignore_patterns,
                vec!["logs".to_string()]
            );
            assert_eq!(
                result.global_defaults.process_whitelist,
                vec!["codex".to_string(), "roo".to_string()]
            );
        }
        other => panic!("unexpected response: {:?}", other),
    }
}

#[test]
fn control_request_deserialization_defaults_omitted_incremental_patch_vectors() {
    let global_request = serde_json::from_value::<ControlRequest>(json!({
        "UpdateGlobalConfig": {
            "ignore_patterns": null,
            "process_whitelist": ["claude"]
        }
    }))
    .unwrap();

    match global_request {
        ControlRequest::UpdateGlobalConfig(patch) => {
            assert_eq!(patch.ignore_patterns, None);
            assert_eq!(patch.process_whitelist, Some(vec!["claude".to_string()]));
            assert!(patch.add_ignore_patterns.is_empty());
            assert!(patch.remove_ignore_patterns.is_empty());
            assert!(patch.add_process_whitelist.is_empty());
            assert!(patch.remove_process_whitelist.is_empty());
        }
        other => panic!("unexpected response: {:?}", other),
    }

    let project_request = serde_json::from_value::<ControlRequest>(json!({
        "UpdateProjectConfig": {
            "id": "demo",
            "ignore_patterns": null,
            "process_whitelist": ["codex"],
            "inherit_ignore_patterns": false,
            "inherit_process_whitelist": true
        }
    }))
    .unwrap();

    match project_request {
        ControlRequest::UpdateProjectConfig(fields) => {
            assert_eq!(fields.id, "demo");
            assert_eq!(fields.patch.ignore_patterns, None);
            assert_eq!(
                fields.patch.process_whitelist,
                Some(vec!["codex".to_string()])
            );
            assert!(fields.patch.add_ignore_patterns.is_empty());
            assert!(fields.patch.remove_ignore_patterns.is_empty());
            assert!(fields.patch.add_process_whitelist.is_empty());
            assert!(fields.patch.remove_process_whitelist.is_empty());
            assert!(!fields.patch.inherit_ignore_patterns);
            assert!(fields.patch.inherit_process_whitelist);
        }
        other => panic!("unexpected response: {:?}", other),
    }
}
