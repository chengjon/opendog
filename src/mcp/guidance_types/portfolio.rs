use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkspacePortfolioLayerStatus {
    Available,
}

impl WorkspacePortfolioLayerStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
        }
    }
}

#[derive(Serialize)]
pub(crate) struct WorkspacePortfolioLayer {
    pub(crate) status: WorkspacePortfolioLayerStatus,
    pub(crate) project_count: usize,
    pub(crate) monitoring_count: usize,
    pub(crate) monitored_projects: Vec<Value>,
    pub(crate) priority_candidates: Vec<Value>,
    pub(crate) project_overviews: Vec<Value>,
    pub(crate) priority_model: String,
    pub(crate) dirty_projects: usize,
    pub(crate) high_risk_projects: usize,
    pub(crate) projects_with_failing_verification: usize,
    pub(crate) projects_safe_for_cleanup: usize,
    pub(crate) projects_safe_for_refactor: usize,
    pub(crate) projects_with_hardcoded_candidates: usize,
    pub(crate) projects_with_hardcoded_data_candidates: usize,
    pub(crate) total_mock_candidates: u64,
    pub(crate) total_hardcoded_candidates: u64,
    pub(crate) projects_in_operation: Vec<Value>,
    pub(crate) attention_queue: Vec<Value>,
    pub(crate) attention_batches: Value,
}
