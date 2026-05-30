use rmcp::schemars;
use serde::Deserialize;

#[derive(Deserialize, schemars::JsonSchema)]
pub struct DataRiskParams {
    pub id: String,
    /// Optional filter: "all" (default), "mock", or "hardcoded"
    pub candidate_type: Option<String>,
    /// Optional minimum priority: "low", "medium", or "high"
    pub min_review_priority: Option<String>,
    /// Optional per-list result limit, defaults to 20
    pub limit: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct WorkspaceDataRiskParams {
    /// Optional filter: "all" (default), "mock", or "hardcoded"
    pub candidate_type: Option<String>,
    /// Optional minimum priority: "low", "medium", or "high"
    pub min_review_priority: Option<String>,
    /// Optional maximum number of matching projects to return, defaults to 20
    pub project_limit: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct AgentGuidanceParams {
    /// Optional single-project scope
    pub project_id: Option<String>,
    /// Optional priority list limit, defaults to 5
    pub top: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct DecisionBriefParams {
    /// Optional single-project scope
    pub project_id: Option<String>,
    /// Optional priority list limit, defaults to 5
    pub top: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct GuidanceParams {
    /// Optional single-project scope
    pub project_id: Option<String>,
    /// Optional priority list limit, defaults to 5
    pub top: Option<usize>,
    /// Optional merged response mode: "summary" (default) or "decision"
    pub detail: Option<String>,
}
