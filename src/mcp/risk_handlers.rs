use rmcp::handler::server::wrapper::Json;
use serde_json::Value;

use crate::control::DaemonClient;
use crate::core::stats;
use crate::error::OpenDogError;

use super::{
    error_json_for, normalize_candidate_type, normalize_min_review_priority,
    project_data_risk_payload, validation_error_json, workspace_data_risk_payload, DataRiskParams,
    OpenDogServer, WorkspaceDataRiskParams, MCP_DATA_RISK_V1, MCP_WORKSPACE_DATA_RISK_V1,
};

pub(super) fn handle_get_data_risk_candidates(
    server: &OpenDogServer,
    DataRiskParams {
        id,
        candidate_type,
        min_review_priority,
        limit,
    }: DataRiskParams,
) -> Json<Value> {
    let candidate_type = match normalize_candidate_type(candidate_type) {
        Ok(value) => value,
        Err(error) => {
            return validation_error_json(
                MCP_DATA_RISK_V1,
                Some(&id),
                "invalid_candidate_type",
                error["error"].as_str().unwrap_or("Invalid candidate_type"),
            );
        }
    };
    let min_review_priority = match normalize_min_review_priority(min_review_priority) {
        Ok(value) => value,
        Err(error) => {
            return validation_error_json(
                MCP_DATA_RISK_V1,
                Some(&id),
                "invalid_min_review_priority",
                error["error"]
                    .as_str()
                    .unwrap_or("Invalid min_review_priority"),
            );
        }
    };
    let limit = limit.unwrap_or(20).max(1);

    match DaemonClient::new().get_data_risk_candidates(
        &id,
        &candidate_type,
        &min_review_priority,
        limit,
        MCP_DATA_RISK_V1,
    ) {
        Ok(payload) => return Json(payload),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_DATA_RISK_V1, Some(&id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, info) = server.get_project(&id)?;
        let entries = stats::get_stats(&db)?;
        Ok((info.root_path, entries))
    })();

    match result {
        Ok((root_path, entries)) => Json(project_data_risk_payload(
            MCP_DATA_RISK_V1,
            &id,
            &candidate_type,
            &min_review_priority,
            limit,
            &root_path,
            &entries,
        )),
        Err(e) => error_json_for(MCP_DATA_RISK_V1, Some(&id), &e),
    }
}

pub(super) fn handle_get_workspace_data_risk_overview(
    server: &OpenDogServer,
    WorkspaceDataRiskParams {
        candidate_type,
        min_review_priority,
        project_limit,
    }: WorkspaceDataRiskParams,
) -> Json<Value> {
    let candidate_type = match normalize_candidate_type(candidate_type) {
        Ok(value) => value,
        Err(error) => {
            return validation_error_json(
                MCP_WORKSPACE_DATA_RISK_V1,
                None,
                "invalid_candidate_type",
                error["error"].as_str().unwrap_or("Invalid candidate_type"),
            );
        }
    };
    let min_review_priority = match normalize_min_review_priority(min_review_priority) {
        Ok(value) => value,
        Err(error) => {
            return validation_error_json(
                MCP_WORKSPACE_DATA_RISK_V1,
                None,
                "invalid_min_review_priority",
                error["error"]
                    .as_str()
                    .unwrap_or("Invalid min_review_priority"),
            );
        }
    };
    let project_limit = project_limit.unwrap_or(20).max(1);

    match DaemonClient::new().get_workspace_data_risk_overview(
        &candidate_type,
        &min_review_priority,
        project_limit,
        MCP_WORKSPACE_DATA_RISK_V1,
    ) {
        Ok(payload) => return Json(payload),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_WORKSPACE_DATA_RISK_V1, None, &e),
    }

    let inner = server.inner.lock().unwrap();
    match inner.list_projects() {
        Ok(projects) => Json(workspace_data_risk_payload(
            MCP_WORKSPACE_DATA_RISK_V1,
            &projects,
            &candidate_type,
            &min_review_priority,
            project_limit,
            |project| {
                inner
                    .project_manager()
                    .open_project_db(&project.id)
                    .ok()
                    .and_then(|db| stats::get_stats(&db).ok())
                    .unwrap_or_default()
            },
        )),
        Err(e) => error_json_for(MCP_WORKSPACE_DATA_RISK_V1, None, &e),
    }
}
