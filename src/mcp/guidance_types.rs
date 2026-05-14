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
