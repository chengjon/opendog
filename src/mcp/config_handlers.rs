use rmcp::handler::server::wrapper::Json;
use serde_json::Value;

use crate::control::DaemonClient;
use crate::core::self_update::build_info_needs_rebuild;
use crate::error::OpenDogError;

use super::{
    build_info_payload, error_json_for, global_config_payload, project_config_payload,
    OpenDogServer, MCP_BUILD_INFO_V1, MCP_GLOBAL_CONFIG_V1, MCP_PROJECT_CONFIG_V1,
    OPENDOG_BUILD_TIME, OPENDOG_GIT_HASH, OPENDOG_VERSION,
};

pub(super) fn handle_get_global_config(server: &OpenDogServer) -> Json<Value> {
    match DaemonClient::new().global_config() {
        Ok(config) => return Json(global_config_payload(MCP_GLOBAL_CONFIG_V1, &config)),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_GLOBAL_CONFIG_V1, None, &e),
    }

    let inner = match server.lock_inner() {
        Ok(inner) => inner,
        Err(e) => return error_json_for(MCP_GLOBAL_CONFIG_V1, None, &e),
    };
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

    let inner = match server.lock_inner() {
        Ok(inner) => inner,
        Err(e) => return error_json_for(MCP_PROJECT_CONFIG_V1, Some(id), &e),
    };
    match inner.project_manager().project_config_view(id) {
        Ok(view) => Json(project_config_payload(MCP_PROJECT_CONFIG_V1, &view)),
        Err(e) => error_json_for(MCP_PROJECT_CONFIG_V1, Some(id), &e),
    }
}

pub(super) fn handle_get_build_info(_server: &OpenDogServer) -> Json<Value> {
    let binary_path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let needs_rebuild = build_info_needs_rebuild();
    let daemon_running = DaemonClient::new().ping().is_ok();
    let opendog_home = crate::config::data_dir().display().to_string();
    Json(build_info_payload(
        MCP_BUILD_INFO_V1,
        OPENDOG_VERSION,
        OPENDOG_GIT_HASH,
        OPENDOG_BUILD_TIME,
        &binary_path,
        needs_rebuild,
        daemon_running,
        &opendog_home,
    ))
}
