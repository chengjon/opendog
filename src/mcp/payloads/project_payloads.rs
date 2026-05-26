use serde_json::{json, Value};
use std::path::Path;

use crate::config::ProjectInfo;
use crate::contracts::{
    versioned_payload, versioned_project_payload, MCP_DELETE_PROJECT_V1, MCP_LIST_PROJECTS_V1,
    MCP_REGISTER_PROJECT_V1, MCP_SNAPSHOT_V1, MCP_START_MONITOR_V1, MCP_STOP_MONITOR_V1,
};

use super::super::{
    register_project_guidance, snapshot_guidance, start_monitor_guidance, tool_guidance,
};

pub(crate) fn snapshot_payload(
    id: &str,
    total_files: usize,
    new_files: usize,
    removed_files: usize,
) -> Value {
    versioned_project_payload(
        MCP_SNAPSHOT_V1,
        id,
        [
            ("total_files", json!(total_files)),
            ("new_files", json!(new_files)),
            ("removed_files", json!(removed_files)),
            ("guidance", snapshot_guidance(total_files)),
        ],
    )
}

pub(crate) fn register_project_payload(id: &str, root_path: &Path) -> Value {
    versioned_payload(
        MCP_REGISTER_PROJECT_V1,
        [
            ("id", json!(id)),
            ("root_path", json!(root_path.display().to_string())),
            ("status", json!("registered")),
            ("guidance", register_project_guidance()),
        ],
    )
}

pub(crate) fn start_monitor_payload(
    id: &str,
    already_running: bool,
    snapshot_taken: bool,
) -> Value {
    versioned_project_payload(
        MCP_START_MONITOR_V1,
        id,
        [
            ("status", json!("monitoring")),
            ("already_running", json!(already_running)),
            ("snapshot_taken", json!(snapshot_taken)),
            (
                "guidance",
                start_monitor_guidance(already_running, snapshot_taken),
            ),
        ],
    )
}

pub(crate) fn stop_monitor_payload(id: &str, stopped: bool) -> Value {
    let mut fields = vec![(
        "status",
        json!(if stopped { "stopped" } else { "not_running" }),
    )];
    if !stopped {
        fields.push((
            "error",
            json!(format!("No monitor running for project '{}'", id)),
        ));
    }
    versioned_project_payload(MCP_STOP_MONITOR_V1, id, fields)
}

pub(crate) fn list_projects_payload(projects: &[ProjectInfo]) -> Value {
    let list: Vec<Value> = projects
        .iter()
        .map(|p| {
            json!({
                "id": p.id,
                "root_path": p.root_path.display().to_string(),
                "status": p.status,
                "created_at": p.created_at,
            })
        })
        .collect();

    versioned_payload(
        MCP_LIST_PROJECTS_V1,
        [
            ("projects", json!(list)),
            ("count", json!(projects.len())),
            (
                "guidance",
                tool_guidance(
                    "Project list loaded. Pick a project, then start monitoring or inspect stats.",
                    &[
                        "opendog start --id <project>",
                        "opendog stats --id <project>",
                        "opendog snapshot --id <project>",
                    ],
                    &["start_monitor", "get_stats", "take_snapshot"],
                    Some(
                        "Use shell commands after you know which project you want to inspect in detail.",
                    ),
                ),
            ),
        ],
    )
}

pub(crate) fn delete_project_payload(id: &str, deleted: bool) -> Value {
    let mut fields = vec![(
        "status",
        json!(if deleted { "deleted" } else { "not_found" }),
    )];
    if !deleted {
        fields.push(("error", json!(format!("Project '{}' not found", id))));
    }
    versioned_project_payload(MCP_DELETE_PROJECT_V1, id, fields)
}

#[cfg(test)]
mod tests {
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
}
