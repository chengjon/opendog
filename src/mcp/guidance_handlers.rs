use rmcp::handler::server::wrapper::Json;
use serde_json::Value;

use crate::control::DaemonClient;
use crate::core::stats;
use crate::error::OpenDogError;
use crate::guidance::{
    build_agent_guidance_for_projects, build_decision_brief_for_projects,
    load_project_guidance_data,
};

use super::{
    error_json_for, scoped_projects_or_error, AgentGuidanceParams, DecisionBriefParams,
    OpenDogServer, MCP_DECISION_BRIEF_V1, MCP_GUIDANCE_V1,
};

pub(super) fn handle_get_agent_guidance(
    server: &OpenDogServer,
    AgentGuidanceParams { project_id, top }: AgentGuidanceParams,
) -> Json<Value> {
    let top = top.unwrap_or(5).max(1);

    match DaemonClient::new().get_agent_guidance(project_id.as_deref(), top) {
        Ok(payload) => return Json(payload),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_GUIDANCE_V1, project_id.as_deref(), &e),
    }

    let inner = server.inner.lock().unwrap();
    match inner.list_projects() {
        Ok(projects) => match scoped_projects_or_error(projects, project_id.as_deref()) {
            Ok(projects) => Json(build_agent_guidance_for_projects(
                &projects,
                top,
                |project| load_project_guidance_data(inner.project_manager(), project),
            )),
            Err(e) => error_json_for(MCP_GUIDANCE_V1, project_id.as_deref(), &e),
        },
        Err(e) => error_json_for(MCP_GUIDANCE_V1, project_id.as_deref(), &e),
    }
}

pub(super) fn handle_get_decision_brief(
    server: &OpenDogServer,
    DecisionBriefParams { project_id, top }: DecisionBriefParams,
) -> Json<Value> {
    let top = top.unwrap_or(5).max(1);

    match DaemonClient::new().get_decision_brief(project_id.as_deref(), top, MCP_DECISION_BRIEF_V1)
    {
        Ok(payload) => return Json(payload),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_DECISION_BRIEF_V1, project_id.as_deref(), &e),
    }

    let inner = server.inner.lock().unwrap();
    let projects = match inner.list_projects() {
        Ok(projects) => projects,
        Err(e) => return error_json_for(MCP_DECISION_BRIEF_V1, project_id.as_deref(), &e),
    };
    let projects = match scoped_projects_or_error(projects, project_id.as_deref()) {
        Ok(projects) => projects,
        Err(e) => return error_json_for(MCP_DECISION_BRIEF_V1, project_id.as_deref(), &e),
    };

    Json(build_decision_brief_for_projects(
        MCP_DECISION_BRIEF_V1,
        if project_id.is_some() {
            "project"
        } else {
            "workspace"
        },
        project_id.as_deref(),
        &projects,
        top,
        |project| load_project_guidance_data(inner.project_manager(), project),
        |project| {
            inner
                .project_manager()
                .open_project_db(&project.id)
                .ok()
                .and_then(|db| stats::get_stats(&db).ok())
                .unwrap_or_default()
        },
    ))
}
