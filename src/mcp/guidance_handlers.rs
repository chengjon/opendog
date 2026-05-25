use rmcp::handler::server::wrapper::Json;
use serde_json::Value;

use crate::control::Guidance;
use crate::error::OpenDogError;

use super::project_handlers::project_lifecycle;
use super::{
    error_json_for, AgentGuidanceParams, DecisionBriefParams, GuidanceParams, OpenDogServer,
    MCP_DECISION_BRIEF_V1, MCP_GUIDANCE_V1,
};

#[derive(Debug)]
pub(super) enum GuidanceDetail {
    Summary,
    Decision,
}

fn parse_guidance_detail(detail: Option<&str>) -> crate::error::Result<GuidanceDetail> {
    match detail.unwrap_or("summary") {
        "summary" => Ok(GuidanceDetail::Summary),
        "decision" => Ok(GuidanceDetail::Decision),
        value => Err(OpenDogError::InvalidInput(format!(
            "detail must be one of: summary, decision; got '{}'",
            value
        ))),
    }
}

pub(super) fn handle_get_guidance(
    server: &OpenDogServer,
    GuidanceParams {
        project_id,
        top,
        detail,
    }: GuidanceParams,
) -> Json<Value> {
    let detail = match parse_guidance_detail(detail.as_deref()) {
        Ok(detail) => detail,
        Err(error) => return error_json_for(MCP_GUIDANCE_V1, project_id.as_deref(), &error),
    };
    match detail {
        GuidanceDetail::Summary => {
            handle_get_agent_guidance(server, AgentGuidanceParams { project_id, top })
        }
        GuidanceDetail::Decision => {
            handle_get_decision_brief(server, DecisionBriefParams { project_id, top })
        }
    }
}

pub(super) fn handle_get_agent_guidance(
    server: &OpenDogServer,
    AgentGuidanceParams { project_id, top }: AgentGuidanceParams,
) -> Json<Value> {
    let svc = project_lifecycle(server);
    let top = top.unwrap_or(5).max(1);
    match svc.get_agent_guidance(project_id.as_deref(), top) {
        Ok(payload) => Json(payload),
        Err(e) => error_json_for(MCP_GUIDANCE_V1, project_id.as_deref(), &e),
    }
}

pub(super) fn handle_get_decision_brief(
    server: &OpenDogServer,
    DecisionBriefParams { project_id, top }: DecisionBriefParams,
) -> Json<Value> {
    let svc = project_lifecycle(server);
    let top = top.unwrap_or(5).max(1);
    match svc.get_decision_brief(MCP_DECISION_BRIEF_V1, project_id.as_deref(), top) {
        Ok(payload) => Json(payload),
        Err(e) => error_json_for(MCP_DECISION_BRIEF_V1, project_id.as_deref(), &e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_guidance_detail_accepts_valid_values() {
        assert!(parse_guidance_detail(None).is_ok());
        assert!(parse_guidance_detail(Some("summary")).is_ok());
        assert!(parse_guidance_detail(Some("decision")).is_ok());
    }

    #[test]
    fn parse_guidance_detail_rejects_invalid_value() {
        let err = parse_guidance_detail(Some("detailed")).unwrap_err();
        assert!(err.to_string().contains("detail must be one of: summary, decision"));
        assert!(err.to_string().contains("detailed"));
    }
}
