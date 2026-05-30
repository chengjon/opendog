use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

use super::*;

#[tool_router(router = governance_tool_router, vis = "pub(super)")]
impl OpenDogServer {
    #[tool(
        name = "create_governance_lane",
        description = "Create a governance work lane for the project. Required params: id, lane_id, title. Optional: description."
    )]
    fn create_governance_lane(
        &self,
        Parameters(params): Parameters<CreateGovernanceLaneParams>,
    ) -> ToolResult {
        let (id, input) = params.into_parts();
        structured_tool_output(handle_create_governance_lane(self, &id, input))
    }

    #[tool(
        name = "upsert_governance_node",
        description = "Create or update a governance node within a lane. Required params: id, lane_id, node_id. State is required on create, optional on update."
    )]
    fn upsert_governance_node(
        &self,
        Parameters(params): Parameters<UpsertGovernanceNodeParams>,
    ) -> ToolResult {
        let (id, input) = params.into_parts();
        structured_tool_output(handle_upsert_governance_node(self, &id, input))
    }

    #[tool(
        name = "get_governance_state",
        description = "Read governance state for a project. Required param: id. Optional params: lane_id, node_id to filter results."
    )]
    fn get_governance_state(
        &self,
        Parameters(params): Parameters<GetGovernanceStateParams>,
    ) -> ToolResult {
        let (id, input) = params.into_parts();
        structured_tool_output(handle_get_governance_state(self, &id, input))
    }

    #[tool(
        name = "close_governance_lane",
        description = "Close, defer, or hard-delete an entire lane and its nodes. Required params: id, lane_id, action (complete|defer|delete)."
    )]
    fn close_governance_lane(
        &self,
        Parameters(params): Parameters<CloseGovernanceLaneParams>,
    ) -> ToolResult {
        let (id, input) = params.into_parts();
        structured_tool_output(handle_close_governance_lane(self, &id, input))
    }
}
