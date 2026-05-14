use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub(crate) struct Recommendation {
    pub(crate) project_id: String,
    pub(crate) recommended_next_action: String,
    pub(crate) recommended_flow: Vec<String>,
    pub(crate) reason: String,
    pub(crate) confidence: String,
    pub(crate) strategy_mode: String,
    pub(crate) strategy_profile: Value,
    pub(crate) verification_gate_levels: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cleanup_blockers: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) refactor_blockers: Option<Value>,
    pub(crate) repo_truth_gaps: Value,
    pub(crate) mandatory_shell_checks: Value,
    pub(crate) suggested_commands: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct ProjectOverview {
    pub(crate) project_id: String,
    pub(crate) status: String,
    pub(crate) snapshot_available: bool,
    pub(crate) activity_available: bool,
    pub(crate) unused_files: i64,
    pub(crate) observation: Value,
    pub(crate) repo_status_risk: Value,
    pub(crate) verification_evidence: Value,
    pub(crate) mock_data_summary: Value,
    pub(crate) storage_maintenance: Value,
    pub(crate) project_toolchain: Value,
    pub(crate) verification_safe_for_cleanup: Value,
    pub(crate) verification_safe_for_refactor: Value,
    pub(crate) verification_gate_levels: Value,
    pub(crate) safe_for_cleanup: Value,
    pub(crate) safe_for_cleanup_reason: Value,
    pub(crate) cleanup_blockers: Value,
    pub(crate) safe_for_refactor: Value,
    pub(crate) safe_for_refactor_reason: Value,
    pub(crate) refactor_blockers: Value,
    pub(crate) recommended_next_action: Value,
    pub(crate) recommended_flow: Value,
    pub(crate) recommended_reason: Value,
    pub(crate) strategy_confidence: Value,
}

#[derive(Serialize)]
pub(crate) struct AttentionSummary {
    pub(crate) attention_score: i64,
    pub(crate) attention_band: String,
    pub(crate) attention_reasons: Vec<String>,
    pub(crate) evidence_quality: String,
    pub(crate) priority_basis: AttentionPriorityBasis,
}

#[derive(Serialize)]
pub(crate) struct AttentionPriorityBasis {
    pub(crate) recommended_next_action: String,
    pub(crate) recommended_action_base: i64,
    pub(crate) repo_risk_level: String,
    pub(crate) repo_in_operation: bool,
    pub(crate) repo_is_dirty: bool,
    pub(crate) verification_status: String,
    pub(crate) has_failing_verification: bool,
    pub(crate) coverage_state: String,
    pub(crate) snapshot_freshness: String,
    pub(crate) activity_freshness: String,
    pub(crate) verification_freshness: String,
    pub(crate) hardcoded_candidate_count: u64,
    pub(crate) mock_candidate_count: u64,
    pub(crate) safe_for_cleanup: bool,
    pub(crate) safe_for_refactor: bool,
}

#[derive(Serialize)]
pub(crate) struct WorkspacePortfolioLayer {
    pub(crate) status: String,
    pub(crate) project_count: usize,
    pub(crate) priority_model: String,
    pub(crate) dirty_projects: usize,
    pub(crate) high_risk_projects: usize,
    pub(crate) projects_with_failing_verification: usize,
    pub(crate) projects_safe_for_cleanup: usize,
    pub(crate) projects_safe_for_refactor: usize,
    pub(crate) projects_with_hardcoded_candidates: usize,
    pub(crate) total_mock_candidates: u64,
    pub(crate) total_hardcoded_candidates: u64,
    pub(crate) projects_in_operation: Vec<Value>,
    pub(crate) attention_queue: Vec<Value>,
    pub(crate) attention_batches: Value,
}

#[derive(Serialize)]
pub(crate) struct DecisionBrief {
    pub(crate) summary: String,
    pub(crate) recommended_next_action: String,
    pub(crate) reason: Value,
    pub(crate) repo_truth_gaps: Value,
    pub(crate) mandatory_shell_checks: Value,
    pub(crate) external_truth_boundary: Value,
    pub(crate) review_focus: Value,
    pub(crate) execution_sequence: Value,
    pub(crate) data_risk_focus: Value,
    pub(crate) target_project_id: Option<String>,
    pub(crate) strategy_mode: Value,
    pub(crate) preferred_primary_tool: Value,
    pub(crate) preferred_secondary_tool: Value,
    pub(crate) recommended_flow: Value,
    pub(crate) safe_for_cleanup: Option<bool>,
    pub(crate) safe_for_refactor: Option<bool>,
    pub(crate) verification_status: String,
    pub(crate) requires_verification: bool,
    pub(crate) action_profile: Value,
    pub(crate) risk_profile: Value,
    pub(crate) signals: DecisionSignals,
}

#[derive(Serialize)]
pub(crate) struct DecisionSignals {
    pub(crate) repo_risk_level: String,
    pub(crate) repo_is_dirty: bool,
    pub(crate) hardcoded_candidate_count: u64,
    pub(crate) mock_candidate_count: u64,
    pub(crate) mixed_review_file_count: u64,
    pub(crate) storage_maintenance_candidate: bool,
    pub(crate) storage_vacuum_candidate: bool,
    pub(crate) storage_reclaimable_bytes: i64,
    pub(crate) storage_db_size_bytes: i64,
    pub(crate) attention_score: i64,
    pub(crate) attention_band: String,
    pub(crate) attention_reasons: Vec<Value>,
    pub(crate) monitoring_count: u64,
}

#[derive(Serialize)]
pub(crate) struct RepoTruthSummary {
    pub(crate) projects_with_repo_truth_gaps: u64,
    pub(crate) repo_truth_gap_distribution: Value,
    pub(crate) mandatory_shell_check_examples: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct StabilizationSummary {
    pub(crate) projects_requiring_repo_stabilization: u64,
    pub(crate) repo_stabilization_priority_projects: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct VerificationSummary {
    pub(crate) projects_requiring_verification_run: u64,
    pub(crate) projects_requiring_failing_verification_repair: u64,
}

#[derive(Serialize)]
pub(crate) struct ObservationSummary {
    pub(crate) projects_requiring_monitor_start: u64,
    pub(crate) projects_requiring_snapshot_refresh: u64,
    pub(crate) projects_requiring_activity_generation: u64,
}

#[derive(Serialize)]
pub(crate) struct DataRiskFocusSummary {
    pub(crate) data_risk_focus_distribution: Value,
    pub(crate) projects_requiring_hardcoded_review: u64,
    pub(crate) projects_requiring_mock_review: u64,
    pub(crate) projects_requiring_mixed_file_review: u64,
}

#[derive(Serialize)]
pub(crate) struct WorkspaceObservationLayer {
    pub(crate) status: String,
    pub(crate) project_count: usize,
    pub(crate) monitoring_count: usize,
    pub(crate) analysis_state: String,
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

#[derive(Serialize)]
pub(crate) struct ExecutionStrategyLayer {
    pub(crate) status: String,
    pub(crate) recommended_flow: Value,
    pub(crate) project_recommendations: Vec<Value>,
    pub(crate) global_strategy_mode: Value,
    pub(crate) preferred_primary_tool: Value,
    pub(crate) preferred_secondary_tool: Value,
    pub(crate) evidence_priority: Value,
    pub(crate) risk_strategy_coupling: Value,
    pub(crate) external_truth_boundary: Value,
    pub(crate) review_focus_projection: Value,
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
    pub(crate) data_risk_focus_distribution: Value,
    pub(crate) projects_requiring_hardcoded_review: Value,
    pub(crate) projects_requiring_mock_review: Value,
    pub(crate) projects_requiring_mixed_file_review: Value,
    pub(crate) projects_requiring_monitor_start: Value,
    pub(crate) projects_requiring_snapshot_refresh: Value,
    pub(crate) projects_requiring_activity_generation: Value,
    pub(crate) projects_with_repo_truth_gaps: Value,
    pub(crate) repo_truth_gap_distribution: Value,
    pub(crate) mandatory_shell_check_examples: Value,
    pub(crate) projects_requiring_verification_run: Value,
    pub(crate) projects_requiring_failing_verification_repair: Value,
    pub(crate) projects_requiring_repo_stabilization: Value,
    pub(crate) repo_stabilization_priority_projects: Value,
}
