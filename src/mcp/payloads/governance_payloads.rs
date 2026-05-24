use serde_json::{json, Value};

use crate::contracts::{
    versioned_project_payload, MCP_CLOSE_GOVERNANCE_LANE_V1, MCP_CREATE_GOVERNANCE_LANE_V1,
    MCP_GET_GOVERNANCE_STATE_V1, MCP_UPSERT_GOVERNANCE_NODE_V1,
};
use crate::core::governance::{GovernanceState, UpsertNodeResult};
use crate::storage::queries::GovernanceLane;

pub(crate) fn create_governance_lane_payload(id: &str, lane: &GovernanceLane) -> Value {
    versioned_project_payload(
        MCP_CREATE_GOVERNANCE_LANE_V1,
        id,
        [
            ("lane_id", json!(lane.lane_id)),
            ("title", json!(lane.title)),
            ("description", json!(lane.description)),
            ("status", json!(lane.status)),
            ("created_at", json!(lane.created_at)),
        ],
    )
}

pub(crate) fn upsert_governance_node_payload(id: &str, result: &UpsertNodeResult) -> Value {
    versioned_project_payload(
        MCP_UPSERT_GOVERNANCE_NODE_V1,
        id,
        [
            ("node_id", json!(result.node_id)),
            ("lane_id", json!(result.lane_id)),
            ("state", json!(result.state)),
            ("created", json!(result.created)),
        ],
    )
}

pub(crate) fn get_governance_state_payload(id: &str, state: &GovernanceState) -> Value {
    versioned_project_payload(
        MCP_GET_GOVERNANCE_STATE_V1,
        id,
        [
            (
                "lanes",
                json!(state.lanes.iter().map(|l| json!({
                    "lane_id": l.lane_id, "title": l.title, "status": l.status,
                    "node_count": l.node_count, "active_nodes": l.active_nodes,
                })).collect::<Vec<_>>()),
            ),
            (
                "nodes",
                json!(state.nodes.iter().map(|n| {
                    let mut obj = json!({
                        "node_id": n.node_id, "lane_id": n.lane_id, "state": n.state,
                        "updated_at": n.updated_at,
                    });
                    if n.summary.is_some() { obj["summary"] = json!(n.summary); }
                    if n.evidence_refs.is_some() { obj["evidence_refs"] = json!(n.evidence_refs); }
                    if n.artifact_refs.is_some() { obj["artifact_refs"] = json!(n.artifact_refs); }
                    if n.reported_git_head.is_some() { obj["reported_git_head"] = json!(n.reported_git_head); }
                    if n.suggested_next.is_some() { obj["suggested_next"] = json!(n.suggested_next); }
                    if n.forbidden_scope.is_some() { obj["forbidden_scope"] = json!(n.forbidden_scope); }
                    if n.external_anchors.is_some() { obj["external_anchors"] = json!(n.external_anchors); }
                    obj
                }).collect::<Vec<_>>()),
            ),
            ("observation_hints", json!({
                "snapshot_freshness": state.observation_hints.snapshot_freshness,
                "verification_status": state.observation_hints.verification_status,
                "data_risk_candidates": state.observation_hints.data_risk_candidates,
                "unused_files": state.observation_hints.unused_files,
            })),
        ],
    )
}

pub(crate) fn close_governance_lane_payload(
    id: &str,
    lane_id: &str,
    action_taken: &str,
    status: &str,
    nodes_affected: usize,
) -> Value {
    versioned_project_payload(
        MCP_CLOSE_GOVERNANCE_LANE_V1,
        id,
        [
            ("lane_id", json!(lane_id)),
            ("action_taken", json!(action_taken)),
            ("status", json!(status)),
            ("nodes_affected", json!(nodes_affected)),
        ],
    )
}
