use rmcp::handler::server::wrapper::Json;
use serde_json::Value;

use crate::config::{ConfigPatch, ProjectConfigPatch};
use crate::control::DaemonClient;
use crate::error::OpenDogError;

use super::{
    error_json_for, global_config_payload, project_config_payload, project_config_reload_payload,
    project_config_update_payload, update_global_config_payload, OpenDogServer,
    MCP_GLOBAL_CONFIG_V1, MCP_PROJECT_CONFIG_V1, MCP_RELOAD_PROJECT_CONFIG_V1,
    MCP_UPDATE_GLOBAL_CONFIG_V1, MCP_UPDATE_PROJECT_CONFIG_V1,
};

pub(super) fn handle_get_global_config(server: &OpenDogServer) -> Json<Value> {
    match DaemonClient::new().global_config() {
        Ok(config) => return Json(global_config_payload(MCP_GLOBAL_CONFIG_V1, &config)),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_GLOBAL_CONFIG_V1, None, &e),
    }

    let inner = server.inner.lock().unwrap();
    match inner.project_manager().global_config() {
        Ok(config) => Json(global_config_payload(MCP_GLOBAL_CONFIG_V1, &config)),
        Err(e) => error_json_for(MCP_GLOBAL_CONFIG_V1, None, &e),
    }
}

pub(super) fn handle_get_project_config(server: &OpenDogServer, id: &str) -> Json<Value> {
    match DaemonClient::new().get_project_config(id) {
        Ok(view) => return Json(project_config_payload(MCP_PROJECT_CONFIG_V1, &view)),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_PROJECT_CONFIG_V1, Some(id), &e),
    }

    let inner = server.inner.lock().unwrap();
    match inner.project_manager().project_config_view(id) {
        Ok(view) => Json(project_config_payload(MCP_PROJECT_CONFIG_V1, &view)),
        Err(e) => error_json_for(MCP_PROJECT_CONFIG_V1, Some(id), &e),
    }
}

pub(super) fn handle_update_global_config(
    server: &OpenDogServer,
    patch: ConfigPatch,
) -> Json<Value> {
    match DaemonClient::new().update_global_config(patch.clone()) {
        Ok(result) => {
            return Json(update_global_config_payload(
                MCP_UPDATE_GLOBAL_CONFIG_V1,
                &result,
            ));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_UPDATE_GLOBAL_CONFIG_V1, None, &e),
    }

    let mut inner = server.inner.lock().unwrap();
    match inner.update_global_config(patch) {
        Ok(result) => Json(update_global_config_payload(
            MCP_UPDATE_GLOBAL_CONFIG_V1,
            &result,
        )),
        Err(e) => error_json_for(MCP_UPDATE_GLOBAL_CONFIG_V1, None, &e),
    }
}

pub(super) fn handle_update_project_config(
    server: &OpenDogServer,
    id: &str,
    patch: ProjectConfigPatch,
) -> Json<Value> {
    match DaemonClient::new().update_project_config(id, patch.clone()) {
        Ok(result) => {
            return Json(project_config_update_payload(
                MCP_UPDATE_PROJECT_CONFIG_V1,
                &result,
            ));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_UPDATE_PROJECT_CONFIG_V1, Some(id), &e),
    }

    let mut inner = server.inner.lock().unwrap();
    match inner.update_project_config(id, patch) {
        Ok(result) => Json(project_config_update_payload(
            MCP_UPDATE_PROJECT_CONFIG_V1,
            &result,
        )),
        Err(e) => error_json_for(MCP_UPDATE_PROJECT_CONFIG_V1, Some(id), &e),
    }
}

pub(super) fn handle_reload_project_config(server: &OpenDogServer, id: &str) -> Json<Value> {
    match DaemonClient::new().reload_project_config(id) {
        Ok((reload, effective)) => {
            return Json(project_config_reload_payload(
                MCP_RELOAD_PROJECT_CONFIG_V1,
                id,
                &reload,
                &effective,
            ));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_RELOAD_PROJECT_CONFIG_V1, Some(id), &e),
    }

    let mut inner = server.inner.lock().unwrap();
    match inner.reload_project_config(id) {
        Ok(reload) => match inner.project_manager().effective_project_config(id) {
            Ok(effective) => Json(project_config_reload_payload(
                MCP_RELOAD_PROJECT_CONFIG_V1,
                id,
                &reload,
                &effective,
            )),
            Err(e) => error_json_for(MCP_RELOAD_PROJECT_CONFIG_V1, Some(id), &e),
        },
        Err(e) => error_json_for(MCP_RELOAD_PROJECT_CONFIG_V1, Some(id), &e),
    }
}
