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
