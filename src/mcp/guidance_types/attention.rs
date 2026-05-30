use serde::Serialize;

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
