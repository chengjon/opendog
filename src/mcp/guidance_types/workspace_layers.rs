use serde::Serialize;
use serde_json::Value;

use super::{
    DataRiskFocusDistribution, ExecutionEvidencePriority, ExternalTruthBoundary, RepoRiskCoupling,
    RepoRiskPreferredTool, RepoRiskStrategyMode, RepoTruthGapDistribution, ReviewFocusProjection,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkspaceObservationLayerStatus {
    Available,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkspaceObservationAnalysisState {
    Empty,
    InsufficientActivity,
    Ready,
    Stale,
}

#[derive(Serialize)]
pub(crate) struct WorkspaceObservationLayer {
    pub(crate) status: WorkspaceObservationLayerStatus,
    pub(crate) project_count: usize,
    pub(crate) monitoring_count: usize,
    pub(crate) analysis_state: WorkspaceObservationAnalysisState,
    pub(crate) projects_missing_snapshot: usize,
    pub(crate) projects_with_stale_snapshot: usize,
    pub(crate) projects_missing_activity: usize,
    pub(crate) projects_with_stale_activity: usize,
    pub(crate) projects_missing_verification: usize,
    pub(crate) projects_with_stale_verification: usize,
    pub(crate) projects_with_storage_maintenance_candidates: u64,
    pub(crate) projects_with_vacuum_candidates: u64,
    pub(crate) total_storage_reclaimable_bytes: Value,
    pub(crate) data_risk_focus_distribution: Value,
    pub(crate) projects_requiring_hardcoded_review: Value,
    pub(crate) projects_requiring_mock_review: Value,
    pub(crate) projects_requiring_mixed_file_review: Value,
    pub(crate) notes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExecutionStrategyLayerStatus {
    Available,
}

#[derive(Serialize)]
pub(crate) struct ExecutionStrategyLayer {
    pub(crate) status: ExecutionStrategyLayerStatus,
    pub(crate) recommended_flow: Vec<String>,
    pub(crate) project_recommendations: Vec<Value>,
    pub(crate) global_strategy_mode: RepoRiskStrategyMode,
    pub(crate) preferred_primary_tool: RepoRiskPreferredTool,
    pub(crate) preferred_secondary_tool: RepoRiskPreferredTool,
    pub(crate) evidence_priority: Vec<ExecutionEvidencePriority>,
    pub(crate) risk_strategy_coupling: RepoRiskCoupling,
    pub(crate) external_truth_boundary: ExternalTruthBoundary,
    pub(crate) review_focus_projection: ReviewFocusProjection,
    pub(crate) when_to_use_opendog: Vec<&'static str>,
    pub(crate) when_to_use_shell: Vec<&'static str>,
    pub(crate) guardrails: Vec<&'static str>,
    pub(crate) projects_not_ready_for_cleanup: usize,
    pub(crate) projects_not_ready_for_refactor: usize,
    pub(crate) projects_with_hardcoded_data_candidates: usize,
    pub(crate) projects_missing_snapshot: usize,
    pub(crate) projects_with_stale_snapshot: usize,
    pub(crate) projects_missing_activity: usize,
    pub(crate) projects_with_stale_activity: usize,
    pub(crate) projects_missing_verification: usize,
    pub(crate) projects_with_stale_verification: usize,
    pub(crate) projects_with_storage_maintenance_candidates: u64,
    pub(crate) projects_with_vacuum_candidates: u64,
    pub(crate) review_opendog_retention_before_large_cleanup: bool,
    pub(crate) recommend_manual_review_for_hardcoded_data: bool,
    pub(crate) data_risk_focus_distribution: DataRiskFocusDistribution,
    pub(crate) projects_requiring_hardcoded_review: u64,
    pub(crate) projects_requiring_mock_review: u64,
    pub(crate) projects_requiring_mixed_file_review: u64,
    pub(crate) projects_requiring_monitor_start: u64,
    pub(crate) projects_requiring_snapshot_refresh: u64,
    pub(crate) projects_requiring_activity_generation: u64,
    pub(crate) projects_with_repo_truth_gaps: u64,
    pub(crate) repo_truth_gap_distribution: RepoTruthGapDistribution,
    pub(crate) mandatory_shell_check_examples: Vec<String>,
    pub(crate) projects_requiring_verification_run: u64,
    pub(crate) projects_requiring_failing_verification_repair: u64,
    pub(crate) projects_requiring_repo_stabilization: u64,
    pub(crate) repo_stabilization_priority_projects: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConstraintsBoundariesLayerStatus {
    Available,
}

#[derive(Serialize)]
pub(crate) struct ConstraintsBoundariesLayer {
    pub(crate) status: ConstraintsBoundariesLayerStatus,
    pub(crate) direct_observations: Vec<String>,
    pub(crate) inferences: Vec<String>,
    pub(crate) blind_spots: Vec<String>,
    pub(crate) guardrails: Vec<String>,
    pub(crate) destructive_operations_requiring_confirmation: Vec<String>,
    pub(crate) human_review_required_for: Vec<String>,
    pub(crate) cleanup_blockers: Vec<String>,
    pub(crate) refactor_blockers: Vec<String>,
    pub(crate) requires_shell_verification: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_not_ready_for_cleanup: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_not_ready_for_refactor: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_hardcoded_data_candidates: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_missing_snapshot: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_stale_snapshot: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_missing_activity: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_stale_activity: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_missing_verification: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_stale_verification: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_storage_maintenance_candidates: Option<u64>,
}
