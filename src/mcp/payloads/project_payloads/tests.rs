use super::*;
use std::path::PathBuf;

fn make_project_info(id: &str) -> ProjectInfo {
    ProjectInfo {
        id: id.to_string(),
        root_path: PathBuf::from("/tmp/test-project"),
        db_path: PathBuf::from("/tmp/test-project.db"),
        config: Default::default(),
        created_at: "2025-01-01T00:00:00Z".to_string(),
        status: "idle".to_string(),
    }
}

// --- snapshot_payload ---

#[test]
fn snapshot_payload_contains_totals() {
    let result = snapshot_payload("myproj", 100, 10, 5);
    assert_eq!(result["project_id"], "myproj");
    assert_eq!(result["total_files"], 100);
    assert_eq!(result["new_files"], 10);
    assert_eq!(result["removed_files"], 5);
    assert!(result["guidance"].is_object());
}

#[test]
fn snapshot_payload_zero_files() {
    let result = snapshot_payload("empty", 0, 0, 0);
    assert_eq!(result["total_files"], 0);
}

// --- register_project_payload ---

#[test]
fn register_project_payload_fields() {
    let path = Path::new("/opt/test");
    let result = register_project_payload("reg1", path);
    assert_eq!(result["id"], "reg1");
    assert_eq!(result["root_path"], "/opt/test");
    assert_eq!(result["status"], "registered");
    assert!(result["guidance"].is_object());
}

// --- start_monitor_payload ---

#[test]
fn start_monitor_payload_fresh_start() {
    let result = start_monitor_payload("mon1", false, true);
    assert_eq!(result["project_id"], "mon1");
    assert_eq!(result["status"], "monitoring");
    assert_eq!(result["already_running"], false);
    assert_eq!(result["snapshot_taken"], true);
}

#[test]
fn start_monitor_payload_already_running() {
    let result = start_monitor_payload("mon2", true, false);
    assert_eq!(result["already_running"], true);
    assert_eq!(result["snapshot_taken"], false);
}

// --- stop_monitor_payload ---

#[test]
fn stop_monitor_payload_stopped() {
    let result = stop_monitor_payload("s1", true);
    assert_eq!(result["project_id"], "s1");
    assert_eq!(result["status"], "stopped");
    assert!(result.get("error").is_none() || result["error"].is_null());
}

#[test]
fn stop_monitor_payload_not_running() {
    let result = stop_monitor_payload("s2", false);
    assert_eq!(result["status"], "not_running");
    assert!(result["error"].as_str().unwrap().contains("s2"));
}

// --- list_projects_payload ---

#[test]
fn list_projects_empty() {
    let result = list_projects_payload(&[]);
    assert_eq!(result["count"], 0);
    assert!(result["projects"].as_array().unwrap().is_empty());
}

#[test]
fn list_projects_multiple() {
    let projects = vec![make_project_info("p1"), make_project_info("p2")];
    let result = list_projects_payload(&projects);
    assert_eq!(result["count"], 2);
    let list = result["projects"].as_array().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0]["id"], "p1");
    assert_eq!(list[1]["id"], "p2");
}

#[test]
fn list_projects_payload_has_project_fields() {
    let projects = vec![make_project_info("single")];
    let result = list_projects_payload(&projects);
    let entry = &result["projects"].as_array().unwrap()[0];
    assert_eq!(entry["id"], "single");
    assert_eq!(entry["root_path"], "/tmp/test-project");
    assert_eq!(entry["status"], "idle");
    assert!(entry["created_at"].is_string());
}

// --- delete_project_payload ---

#[test]
fn delete_project_payload_deleted() {
    let result = delete_project_payload("d1", true);
    assert_eq!(result["project_id"], "d1");
    assert_eq!(result["status"], "deleted");
    assert!(result.get("error").is_none() || result["error"].is_null());
}

#[test]
fn delete_project_payload_not_found() {
    let result = delete_project_payload("d2", false);
    assert_eq!(result["status"], "not_found");
    assert!(result["error"].as_str().unwrap().contains("d2"));
}
