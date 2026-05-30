use rmcp::schemars;
use serde::Deserialize;

use crate::core::governance::{
    CloseLaneInput, CreateLaneInput, GetGovernanceStateInput, UpsertNodeInput,
};

#[derive(Deserialize, schemars::JsonSchema)]
pub struct CreateGovernanceLaneParams {
    /// Project identifier
    pub id: String,
    /// Unique lane identifier
    pub lane_id: String,
    /// Lane title
    pub title: String,
    /// Optional lane description
    pub description: Option<String>,
}

impl CreateGovernanceLaneParams {
    pub(crate) fn into_parts(self) -> (String, CreateLaneInput) {
        (
            self.id,
            CreateLaneInput {
                lane_id: self.lane_id,
                title: self.title,
                description: self.description,
            },
        )
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct UpsertGovernanceNodeParams {
    /// Project identifier
    pub id: String,
    /// Lane identifier this node belongs to
    pub lane_id: String,
    /// Unique node identifier (e.g. "G2.46")
    pub node_id: String,
    /// Node state — required on create, optional on update
    pub state: Option<String>,
    /// One-line factual summary
    pub summary: Option<String>,
    /// JSON array of report/document paths
    pub evidence_refs: Option<Vec<String>>,
    /// JSON array of generated artifact paths
    pub artifact_refs: Option<Vec<String>>,
    /// Caller-reported HEAD anchor
    pub reported_git_head: Option<String>,
    /// Recommended next step
    pub suggested_next: Option<String>,
    /// JSON array of semantic scope descriptions
    pub forbidden_scope: Option<Vec<String>>,
    /// JSON object with external references
    pub external_anchors: Option<serde_json::Value>,
}

impl UpsertGovernanceNodeParams {
    pub(crate) fn into_parts(self) -> (String, UpsertNodeInput) {
        (
            self.id,
            UpsertNodeInput {
                node_id: self.node_id,
                lane_id: self.lane_id,
                state: self.state,
                summary: self.summary,
                evidence_refs: self.evidence_refs,
                artifact_refs: self.artifact_refs,
                reported_git_head: self.reported_git_head,
                suggested_next: self.suggested_next,
                forbidden_scope: self.forbidden_scope,
                external_anchors: self.external_anchors,
            },
        )
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct GetGovernanceStateParams {
    /// Project identifier
    pub id: String,
    /// Optional lane filter
    pub lane_id: Option<String>,
    /// Optional specific node filter
    pub node_id: Option<String>,
    /// When true, filter out closed/completed lanes and closed nodes
    pub active_only: Option<bool>,
}

impl GetGovernanceStateParams {
    pub(crate) fn into_parts(self) -> (String, GetGovernanceStateInput) {
        (
            self.id,
            GetGovernanceStateInput {
                lane_id: self.lane_id,
                node_id: self.node_id,
                active_only: self.active_only,
            },
        )
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct CloseGovernanceLaneParams {
    /// Project identifier
    pub id: String,
    /// Lane identifier
    pub lane_id: String,
    /// Action: "complete", "defer", or "delete"
    pub action: String,
}

impl CloseGovernanceLaneParams {
    pub(crate) fn into_parts(self) -> (String, CloseLaneInput) {
        (
            self.id,
            CloseLaneInput {
                lane_id: self.lane_id,
                action: self.action,
            },
        )
    }
}
