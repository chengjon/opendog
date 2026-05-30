use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

use super::*;

#[tool_router(router = risk_tool_router, vis = "pub(super)")]
impl OpenDogServer {
    #[tool(
        name = "get_data_risk_candidates",
        description = "Detect mock, fixture, demo, sample, and suspicious hardcoded business-like data candidates for one project. Required param: id. Optional params: candidate_type, min_review_priority, limit."
    )]
    fn get_data_risk_candidates(
        &self,
        Parameters(params): Parameters<DataRiskParams>,
    ) -> ToolResult {
        structured_tool_output(handle_get_data_risk_candidates(self, params))
    }

    #[tool(
        name = "get_workspace_data_risk_overview",
        description = "Aggregate mock and hardcoded-data risk signals across all registered projects so AI can prioritize which project to inspect first. Optional params: candidate_type, min_review_priority, project_limit."
    )]
    fn get_workspace_data_risk_overview(
        &self,
        Parameters(params): Parameters<WorkspaceDataRiskParams>,
    ) -> ToolResult {
        structured_tool_output(handle_get_workspace_data_risk_overview(self, params))
    }
}
