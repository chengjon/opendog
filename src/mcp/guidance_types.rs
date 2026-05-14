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
