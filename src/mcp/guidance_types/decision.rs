use serde::Serialize;
use serde_json::Value;

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
