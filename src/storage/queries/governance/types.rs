use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceLane {
    pub lane_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceNode {
    pub node_id: String,
    pub lane_id: String,
    pub state: String,
    pub summary: Option<String>,
    pub evidence_refs: Option<String>,
    pub artifact_refs: Option<String>,
    pub reported_git_head: Option<String>,
    pub suggested_next: Option<String>,
    pub forbidden_scope: Option<String>,
    pub external_anchors: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGovernanceLane {
    pub lane_id: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertGovernanceNode {
    pub node_id: String,
    pub lane_id: String,
    pub state: Option<String>,
    pub summary: Option<String>,
    pub evidence_refs: Option<String>,
    pub artifact_refs: Option<String>,
    pub reported_git_head: Option<String>,
    pub suggested_next: Option<String>,
    pub forbidden_scope: Option<String>,
    pub external_anchors: Option<String>,
}
