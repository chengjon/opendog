use serde_json::{json, Value};
use std::path::Path;

use crate::config::ProjectInfo;
use crate::contracts::{
    versioned_payload, versioned_project_payload, MCP_CREATE_PROJECT_V1, MCP_DELETE_PROJECT_V1,
    MCP_LIST_PROJECTS_V1, MCP_SNAPSHOT_V1, MCP_START_MONITOR_V1, MCP_STOP_MONITOR_V1,
};

use super::super::{
    create_project_guidance, snapshot_guidance, start_monitor_guidance, tool_guidance,
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

pub(crate) fn create_project_payload(id: &str, root_path: &Path) -> Value {
    versioned_payload(
        MCP_CREATE_PROJECT_V1,
        [
            ("id", json!(id)),
            ("root_path", json!(root_path.display().to_string())),
            ("status", json!("created")),
            ("guidance", create_project_guidance()),
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
