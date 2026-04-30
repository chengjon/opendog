use super::*;

#[test]
fn create_project_payload_has_versioned_contract() {
    let value = create_project_payload("demo", std::path::Path::new("/tmp/demo"));
    assert_eq!(value["schema_version"], MCP_CREATE_PROJECT_V1);
    assert_eq!(value["id"], "demo");
    assert_eq!(value["status"], "created");
    assert_eq!(value["guidance"]["schema_version"], MCP_GUIDANCE_V1);
}

#[test]
fn start_monitor_payload_has_versioned_contract() {
    let value = start_monitor_payload("demo", true, false);
    assert_eq!(value["schema_version"], MCP_START_MONITOR_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["status"], "monitoring");
    assert_eq!(value["already_running"], true);
}

#[test]
fn stop_monitor_payload_reports_missing_monitor_consistently() {
    let value = stop_monitor_payload("demo", false);
    assert_eq!(value["schema_version"], MCP_STOP_MONITOR_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["status"], "not_running");
    assert!(value["error"]
        .as_str()
        .unwrap()
        .contains("No monitor running"));
}

#[test]
fn list_projects_payload_has_versioned_contract() {
    let projects = vec![ProjectInfo {
        id: "demo".to_string(),
        root_path: std::path::PathBuf::from("/tmp/demo"),
        db_path: std::path::PathBuf::from("/tmp/demo.db"),
        config: ProjectConfigOverrides::default(),
        created_at: "2026-04-26T00:00:00Z".to_string(),
        status: "active".to_string(),
    }];
    let value = list_projects_payload(&projects);
    assert_eq!(value["schema_version"], MCP_LIST_PROJECTS_V1);
    assert_eq!(value["count"], 1);
    assert_eq!(value["projects"][0]["id"], "demo");
    assert_eq!(value["guidance"]["schema_version"], MCP_GUIDANCE_V1);
}

#[test]
fn delete_project_payload_reports_not_found_consistently() {
    let value = delete_project_payload("demo", false);
    assert_eq!(value["schema_version"], MCP_DELETE_PROJECT_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["status"], "not_found");
    assert!(value["error"].as_str().unwrap().contains("not found"));
}

#[test]
fn snapshot_payload_has_versioned_contract() {
    let value = snapshot_payload("demo", 10, 3, 1);
    assert_eq!(value["schema_version"], MCP_SNAPSHOT_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["total_files"], 10);
    assert!(value["guidance"]["recommended_flow"].is_array());
}
