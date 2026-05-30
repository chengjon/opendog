use rmcp::schemars;
use serde::Deserialize;

#[derive(Deserialize, schemars::JsonSchema)]
pub struct RegisterProjectParams {
    /// Unique project identifier (alphanumeric, dash, underscore, max 64 chars)
    pub id: String,
    /// Absolute path to the project root directory
    pub path: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct ProjectIdParams {
    /// Project identifier
    pub id: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct ObservationRowsParams {
    /// Project identifier
    pub id: String,
    /// Optional file row limit, defaults to 50 for MCP payload safety.
    pub limit: Option<usize>,
    /// Optional row classification filter: "all" (default), "source", "infrastructure", "backup", or "project".
    pub path_classification: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct TimeWindowReportParams {
    pub id: String,
    /// Optional window: "24h", "7d", or "30d". Defaults to "24h".
    pub window: Option<String>,
    /// Optional row limit, defaults to 10.
    pub limit: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct CompareSnapshotsParams {
    pub id: String,
    /// Optional base snapshot run id. Must be paired with head_run_id when supplied.
    pub base_run_id: Option<i64>,
    /// Optional head snapshot run id. Must be paired with base_run_id when supplied.
    pub head_run_id: Option<i64>,
    /// Optional change row limit, defaults to 20.
    pub limit: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct UsageTrendParams {
    pub id: String,
    /// Optional window: "24h", "7d", or "30d". Defaults to "7d".
    pub window: Option<String>,
    /// Optional file limit, defaults to 10.
    pub limit: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct ActivityRollupParams {
    pub id: String,
    /// Optional window: "24h", "7d", or "30d". Defaults to "30d".
    pub window: Option<String>,
    /// Optional day limit, defaults to 30.
    pub limit: Option<usize>,
}
