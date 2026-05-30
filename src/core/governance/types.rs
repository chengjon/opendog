use crate::storage::queries::GovernanceNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLaneInput {
    pub lane_id: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertNodeInput {
    pub node_id: String,
    pub lane_id: String,
    pub state: Option<String>,
    pub summary: Option<String>,
    pub evidence_refs: Option<Vec<String>>,
    pub artifact_refs: Option<Vec<String>>,
    pub reported_git_head: Option<String>,
    pub suggested_next: Option<String>,
    pub forbidden_scope: Option<Vec<String>>,
    pub external_anchors: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGovernanceStateInput {
    pub lane_id: Option<String>,
    pub node_id: Option<String>,
    pub active_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseLaneInput {
    pub lane_id: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationHints {
    pub snapshot_freshness: String,
    pub verification_status: String,
    pub data_risk_candidates: usize,
    pub unused_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceState {
    pub lanes: Vec<GovernanceLaneSummary>,
    pub nodes: Vec<GovernanceNode>,
    pub observation_hints: ObservationHints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceLaneSummary {
    pub lane_id: String,
    pub title: String,
    pub status: String,
    pub node_count: usize,
    pub active_nodes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertNodeResult {
    pub node_id: String,
    pub lane_id: String,
    pub state: String,
    pub created: bool,
}
