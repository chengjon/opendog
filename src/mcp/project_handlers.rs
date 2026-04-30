use rmcp::handler::server::wrapper::Json;
use serde_json::{json, Value};
use std::path::Path;

use crate::control::DaemonClient;
use crate::core::snapshot;
use crate::error::OpenDogError;

use super::{
    create_project_payload, delete_project_payload, error_json_for, list_projects_payload,
    open_dog_error_code, snapshot_payload, start_monitor_payload, stop_monitor_payload,
    versioned_error_payload, OpenDogServer, MCP_CREATE_PROJECT_V1, MCP_DELETE_PROJECT_V1,
    MCP_LIST_PROJECTS_V1, MCP_SNAPSHOT_V1, MCP_START_MONITOR_V1, MCP_STOP_MONITOR_V1,
};

pub(super) fn handle_create_project(server: &OpenDogServer, id: &str, path: &str) -> Json<Value> {
    match DaemonClient::new().create_project(id, path) {
        Ok(info) => {
            return Json(create_project_payload(&info.id, &info.root_path));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => {
            return Json(versioned_error_payload(
                MCP_CREATE_PROJECT_V1,
                open_dog_error_code(&e),
                &e.to_string(),
                [("id", json!(id)), ("requested_path", json!(path))],
            ));
        }
    }

    let inner = server.inner.lock().unwrap();
    match inner.project_manager().create(id, Path::new(path)) {
        Ok(info) => Json(create_project_payload(&info.id, &info.root_path)),
        Err(e) => Json(versioned_error_payload(
            MCP_CREATE_PROJECT_V1,
            open_dog_error_code(&e),
            &e.to_string(),
            [("id", json!(id)), ("requested_path", json!(path))],
        )),
    }
}

pub(super) fn handle_take_snapshot(server: &OpenDogServer, id: &str) -> Json<Value> {
    match DaemonClient::new().take_snapshot(id) {
        Ok(r) => {
            return Json(snapshot_payload(
                id,
                r.total_files,
                r.new_files,
                r.removed_files,
            ));
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
        Ok(r) => Json(snapshot_payload(
            id,
            r.total_files,
            r.new_files,
            r.removed_files,
        )),
        Err(e) => error_json_for(MCP_SNAPSHOT_V1, Some(id), &e),
    }
}

pub(super) fn handle_start_monitor(server: &OpenDogServer, id: &str) -> Json<Value> {
    match DaemonClient::new().start_monitor(id) {
        Ok(outcome) => {
            return Json(start_monitor_payload(
                id,
                outcome.already_running,
                outcome.snapshot_taken,
            ));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_START_MONITOR_V1, Some(id), &e),
    }

    let result = {
        let mut inner = server.inner.lock().unwrap();
        inner.start_monitor(id)
    };
    match result {
        Ok(outcome) => Json(start_monitor_payload(
            id,
            outcome.already_running,
            outcome.snapshot_taken,
        )),
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

pub(super) fn handle_list_projects(server: &OpenDogServer) -> Json<Value> {
    match DaemonClient::new().list_projects() {
        Ok(projects) => {
            return Json(list_projects_payload(&projects));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_LIST_PROJECTS_V1, None, &e),
    }

    let inner = server.inner.lock().unwrap();
    match inner.list_projects() {
        Ok(projects) => Json(list_projects_payload(&projects)),
        Err(e) => error_json_for(MCP_LIST_PROJECTS_V1, None, &e),
    }
}

pub(super) fn handle_delete_project(server: &OpenDogServer, id: &str) -> Json<Value> {
    match DaemonClient::new().delete_project(id) {
        Ok(true) => {
            return Json(delete_project_payload(id, true));
        }
        Ok(false) => {
            return Json(delete_project_payload(id, false));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_DELETE_PROJECT_V1, Some(id), &e),
    }

    let mut inner = server.inner.lock().unwrap();
    match DaemonClient::new().stop_monitor(id) {
        Ok(_) | Err(OpenDogError::DaemonUnavailable) => {
            inner.stop_monitor(id);
        }
        Err(e) => return error_json_for(MCP_DELETE_PROJECT_V1, Some(id), &e),
    }
    match inner.project_manager().delete(id) {
        Ok(true) => Json(delete_project_payload(id, true)),
        Ok(false) => Json(delete_project_payload(id, false)),
        Err(e) => error_json_for(MCP_DELETE_PROJECT_V1, Some(id), &e),
    }
}
