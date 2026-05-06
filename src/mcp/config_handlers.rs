use rmcp::handler::server::wrapper::Json;
use serde_json::Value;

use crate::control::DaemonClient;
use crate::error::OpenDogError;

use super::{
    error_json_for, global_config_payload, project_config_payload, OpenDogServer,
    MCP_GLOBAL_CONFIG_V1, MCP_PROJECT_CONFIG_V1,
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
