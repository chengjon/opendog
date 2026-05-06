use super::*;

#[test]
fn global_config_payload_has_versioned_contract() {
    let value = global_config_payload(
        MCP_GLOBAL_CONFIG_V1,
        &ProjectConfig {
            ignore_patterns: vec!["dist".to_string()],
            process_whitelist: vec!["codex".to_string()],
        },
    );
    assert_eq!(value["schema_version"], MCP_GLOBAL_CONFIG_V1);
    assert_eq!(value["global_defaults"]["ignore_patterns"][0], "dist");
    assert_eq!(value["guidance"]["schema_version"], MCP_GUIDANCE_V1);
    assert_eq!(
        value["guidance"]["next_tools"],
        json!(["get_project_config"])
    );
}

#[test]
fn project_config_payload_has_versioned_contract() {
    let value = project_config_payload(
        MCP_PROJECT_CONFIG_V1,
        &ProjectConfigView {
            project_id: "demo".to_string(),
            global_defaults: ProjectConfig {
                ignore_patterns: vec!["dist".to_string()],
                process_whitelist: vec!["claude".to_string()],
            },
            project_overrides: ProjectConfigOverrides {
                ignore_patterns: Some(vec!["logs".to_string()]),
                process_whitelist: None,
            },
            effective: ProjectConfig {
                ignore_patterns: vec!["logs".to_string()],
                process_whitelist: vec!["claude".to_string()],
            },
        },
    );
    assert_eq!(value["schema_version"], MCP_PROJECT_CONFIG_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["effective"]["ignore_patterns"][0], "logs");
    assert_eq!(value["inherits"]["process_whitelist"], true);
    assert_eq!(
        value["guidance"]["next_tools"],
        json!(["get_project_config"])
    );
}

#[test]
fn config_update_payloads_include_reload_state() {
    let project_value = project_config_update_payload(
        MCP_UPDATE_PROJECT_CONFIG_V1,
        &ProjectConfigUpdateResult {
            project_id: "demo".to_string(),
            global_defaults: ProjectConfig {
                ignore_patterns: vec!["dist".to_string()],
                process_whitelist: vec!["claude".to_string()],
            },
            project_overrides: ProjectConfigOverrides {
                ignore_patterns: Some(vec!["logs".to_string()]),
                process_whitelist: Some(vec!["codex".to_string()]),
            },
            effective: ProjectConfig {
                ignore_patterns: vec!["logs".to_string()],
                process_whitelist: vec!["codex".to_string()],
            },
            reload: ProjectConfigReload {
                monitor_running: true,
                runtime_reloaded: true,
                snapshot_refreshed: true,
                changed_fields: vec!["ignore_patterns".to_string()],
                skipped_fields: Vec::new(),
            },
        },
    );
    assert_eq!(
        project_value["schema_version"],
        MCP_UPDATE_PROJECT_CONFIG_V1
    );
    assert_eq!(project_value["reload"]["runtime_reloaded"], true);

    let global_value = update_global_config_payload(
        MCP_UPDATE_GLOBAL_CONFIG_V1,
        &GlobalConfigUpdateResult {
            global_defaults: ProjectConfig {
                ignore_patterns: vec!["generated".to_string()],
                process_whitelist: vec!["claude".to_string()],
            },
            reloaded_projects: vec![ProjectReloadStatus {
                project_id: "demo".to_string(),
                monitor_running: true,
                runtime_reloaded: true,
                snapshot_refreshed: false,
                changed_fields: vec!["process_whitelist".to_string()],
                skipped_fields: Vec::new(),
            }],
        },
    );
    assert_eq!(global_value["schema_version"], MCP_UPDATE_GLOBAL_CONFIG_V1);
    assert_eq!(global_value["reloaded_projects"][0]["project_id"], "demo");
    assert_eq!(
        global_value["guidance"]["next_tools"],
        json!(["get_project_config"])
    );
    assert_eq!(
        project_value["guidance"]["next_tools"],
        json!(["get_project_config", "start_monitor"])
    );
}

#[test]
fn project_config_reload_payload_has_versioned_contract() {
    let value = project_config_reload_payload(
        MCP_RELOAD_PROJECT_CONFIG_V1,
        "demo",
        &ProjectConfigReload {
            monitor_running: true,
            runtime_reloaded: true,
            snapshot_refreshed: false,
            changed_fields: vec!["process_whitelist".to_string()],
            skipped_fields: Vec::new(),
        },
        &ProjectConfig {
            ignore_patterns: vec!["logs".to_string()],
            process_whitelist: vec!["codex".to_string()],
        },
    );
    assert_eq!(value["schema_version"], MCP_RELOAD_PROJECT_CONFIG_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["reload"]["changed_fields"][0], "process_whitelist");
}
