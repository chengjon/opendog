use rmcp::handler::server::wrapper::Json;
use serde_json::{json, Value};

use crate::control::{
    DaemonClient, DaemonProjectLifecycle, DirectProjectLifecycle, FallbackLifecycle,
    ProjectLifecycle,
};
use crate::core::snapshot;
use crate::error::OpenDogError;

use super::{
    delete_project_payload, error_json_for, list_projects_payload, open_dog_error_code,
    register_project_payload, snapshot_payload, start_monitor_payload, stop_monitor_payload,
    versioned_error_payload, OpenDogServer, MCP_DELETE_PROJECT_V1, MCP_LIST_PROJECTS_V1,
    MCP_REGISTER_PROJECT_V1, MCP_SNAPSHOT_V1, MCP_START_MONITOR_V1, MCP_STOP_MONITOR_V1,
};

fn project_lifecycle(server: &OpenDogServer) -> FallbackLifecycle<DaemonProjectLifecycle<'static>, DirectProjectLifecycle<'_>> {
    static DAEMON: std::sync::OnceLock<DaemonClient> = std::sync::OnceLock::new();
    let client = DAEMON.get_or_init(DaemonClient::new);
    FallbackLifecycle::new(
        DaemonProjectLifecycle::new(client),
        DirectProjectLifecycle::new(&server.inner),
    )
}

// --- Project lifecycle handlers (migrated to trait) ---

pub(super) fn handle_register_project(server: &OpenDogServer, id: &str, path: &str) -> Json<Value> {
    let svc = project_lifecycle(server);
    match svc.create_project(id, path) {
        Ok(info) => Json(register_project_payload(&info.id, &info.root_path)),
        Err(e) => Json(versioned_error_payload(
            MCP_REGISTER_PROJECT_V1,
            open_dog_error_code(&e),
            &e.to_string(),
            [("id", json!(id)), ("requested_path", json!(path))],
        )),
    }
}

pub(super) fn handle_list_projects(server: &OpenDogServer) -> Json<Value> {
    let svc = project_lifecycle(server);
    match svc.list_projects() {
        Ok(projects) => Json(list_projects_payload(&projects)),
        Err(e) => error_json_for(MCP_LIST_PROJECTS_V1, None, &e),
    }
}

pub(super) fn handle_delete_project(server: &OpenDogServer, id: &str) -> Json<Value> {
    let svc = project_lifecycle(server);
    match svc.delete_project(id) {
        Ok(true) => Json(delete_project_payload(id, true)),
        Ok(false) => Json(delete_project_payload(id, false)),
        Err(e) => error_json_for(MCP_DELETE_PROJECT_V1, Some(id), &e),
    }
}

// --- Snapshot & monitor handlers (still using inline fallback, to migrate in Step 3) ---

pub(super) fn handle_take_snapshot(server: &OpenDogServer, id: &str) -> Json<Value> {
    match DaemonClient::new().take_snapshot(id) {
        Ok(r) => {
            return Json(snapshot_payload(id, r.total_files, r.new_files, r.removed_files));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_SNAPSHOT_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, info) = server.get_project(id)?;
        let effective_config = server
            .inner
            .lock()
            .unwrap()
            .project_manager()
            .resolve_project_config(&info)?;
        snapshot::take_snapshot(&db, &info.root_path, &effective_config)
    })();
    match result {
        Ok(r) => Json(snapshot_payload(id, r.total_files, r.new_files, r.removed_files)),
        Err(e) => error_json_for(MCP_SNAPSHOT_V1, Some(id), &e),
    }
}

pub(super) fn handle_start_monitor(server: &OpenDogServer, id: &str) -> Json<Value> {
    match DaemonClient::new().start_monitor(id) {
        Ok(outcome) => {
            return Json(start_monitor_payload(id, outcome.already_running, outcome.snapshot_taken));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_START_MONITOR_V1, Some(id), &e),
    }

    let result = {
        let mut inner = server.inner.lock().unwrap();
        inner.start_monitor(id)
    };
    match result {
        Ok(outcome) => Json(start_monitor_payload(id, outcome.already_running, outcome.snapshot_taken)),
        Err(e) => error_json_for(MCP_START_MONITOR_V1, Some(id), &e),
    }
}

pub(super) fn handle_stop_monitor(server: &OpenDogServer, id: &str) -> Json<Value> {
    match DaemonClient::new().stop_monitor(id) {
        Ok(true) => {
            return Json(stop_monitor_payload(id, true));
        }
        Ok(false) => {
            return Json(stop_monitor_payload(id, false));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_STOP_MONITOR_V1, Some(id), &e),
    }

    let mut inner = server.inner.lock().unwrap();
    if inner.stop_monitor(id) {
        Json(stop_monitor_payload(id, true))
    } else {
        Json(stop_monitor_payload(id, false))
    }
}
