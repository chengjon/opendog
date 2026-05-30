use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Json;
use rmcp::model::CallToolResult;
use serde_json::Value;

use super::*;

mod analysis;
mod config;
mod governance;
mod lifecycle;
mod risk;
mod verification;

type ToolResult = Result<CallToolResult, rmcp::ErrorData>;

fn structured_tool_output(payload: Json<Value>) -> ToolResult {
    Ok(CallToolResult::structured(payload.0))
}

impl OpenDogServer {
    pub(super) fn tool_router() -> ToolRouter<Self> {
        let mut router = ToolRouter::new();
        router.merge(Self::config_tool_router());
        router.merge(Self::lifecycle_tool_router());
        router.merge(Self::analysis_tool_router());
        router.merge(Self::verification_tool_router());
        router.merge(Self::risk_tool_router());
        router.merge(Self::governance_tool_router());
        router
    }
}
