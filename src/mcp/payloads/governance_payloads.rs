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
                json!(state
                    .lanes
                    .iter()
                    .map(|l| json!({
                        "lane_id": l.lane_id, "title": l.title, "status": l.status,
                        "node_count": l.node_count, "active_nodes": l.active_nodes,
                    }))
                    .collect::<Vec<_>>()),
            ),
            (
                "nodes",
                json!(state
                    .nodes
                    .iter()
                    .map(|n| {
                        let mut obj = json!({
                            "node_id": n.node_id, "lane_id": n.lane_id, "state": n.state,
                            "updated_at": n.updated_at,
                        });
                        if n.summary.is_some() {
                            obj["summary"] = json!(n.summary);
                        }
                        if let Some(ref s) = n.evidence_refs {
                            obj["evidence_refs"] = serde_json::from_str::<'_, serde_json::Value>(s)
                                .unwrap_or_else(|_| json!(s));
                        }
                        if let Some(ref s) = n.artifact_refs {
                            obj["artifact_refs"] = serde_json::from_str::<'_, serde_json::Value>(s)
                                .unwrap_or_else(|_| json!(s));
                        }
                        if n.reported_git_head.is_some() {
                            obj["reported_git_head"] = json!(n.reported_git_head);
                        }
                        if n.suggested_next.is_some() {
                            obj["suggested_next"] = json!(n.suggested_next);
                        }
                        if let Some(ref s) = n.forbidden_scope {
                            obj["forbidden_scope"] =
                                serde_json::from_str::<'_, serde_json::Value>(s)
                                    .unwrap_or_else(|_| json!(s));
                        }
                        if let Some(ref s) = n.external_anchors {
                            obj["external_anchors"] =
                                serde_json::from_str::<'_, serde_json::Value>(s)
                                    .unwrap_or_else(|_| json!(s));
                        }
                        obj
                    })
                    .collect::<Vec<_>>()),
            ),
            (
                "observation_hints",
                json!({
                    "snapshot_freshness": state.observation_hints.snapshot_freshness,
                    "verification_status": state.observation_hints.verification_status,
                    "data_risk_candidates": state.observation_hints.data_risk_candidates,
                    "unused_files": state.observation_hints.unused_files,
                }),
            ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::governance::{
        GovernanceLaneSummary, GovernanceState, ObservationHints, UpsertNodeResult,
    };
    use crate::storage::queries::{GovernanceLane, GovernanceNode};

    // --- create_governance_lane_payload ---

    #[test]
    fn create_lane_payload_fields() {
        let lane = GovernanceLane {
            lane_id: "lane1".to_string(),
            title: "Test Lane".to_string(),
            description: Some("A test".to_string()),
            status: "active".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };
        let payload = create_governance_lane_payload("proj1", &lane);
        assert_eq!(payload["project_id"], "proj1");
        assert_eq!(payload["lane_id"], "lane1");
        assert_eq!(payload["title"], "Test Lane");
        assert_eq!(payload["description"], "A test");
        assert_eq!(payload["status"], "active");
        assert_eq!(payload["created_at"], "2025-01-01T00:00:00Z");
    }

    #[test]
    fn create_lane_payload_null_description() {
        let lane = GovernanceLane {
            lane_id: "lane2".to_string(),
            title: "No Desc".to_string(),
            description: None,
            status: "active".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };
        let payload = create_governance_lane_payload("proj2", &lane);
        assert!(payload["description"].is_null());
    }

    // --- upsert_governance_node_payload ---

    #[test]
    fn upsert_node_payload_created() {
        let result = UpsertNodeResult {
            node_id: "n1".to_string(),
            lane_id: "lane1".to_string(),
            state: "in_progress".to_string(),
            created: true,
        };
        let payload = upsert_governance_node_payload("proj1", &result);
        assert_eq!(payload["project_id"], "proj1");
        assert_eq!(payload["node_id"], "n1");
        assert_eq!(payload["lane_id"], "lane1");
        assert_eq!(payload["state"], "in_progress");
        assert_eq!(payload["created"], true);
    }

    #[test]
    fn upsert_node_payload_updated() {
        let result = UpsertNodeResult {
            node_id: "n2".to_string(),
            lane_id: "lane2".to_string(),
            state: "done".to_string(),
            created: false,
        };
        let payload = upsert_governance_node_payload("proj2", &result);
        assert_eq!(payload["created"], false);
    }

    // --- close_governance_lane_payload ---

    #[test]
    fn close_lane_payload_fields() {
        let payload = close_governance_lane_payload("proj1", "lane1", "complete", "closed", 3);
        assert_eq!(payload["project_id"], "proj1");
        assert_eq!(payload["lane_id"], "lane1");
        assert_eq!(payload["action_taken"], "complete");
        assert_eq!(payload["status"], "closed");
        assert_eq!(payload["nodes_affected"], 3);
    }

    #[test]
    fn close_lane_payload_zero_nodes() {
        let payload = close_governance_lane_payload("p", "l", "delete", "deleted", 0);
        assert_eq!(payload["nodes_affected"], 0);
    }

    // --- get_governance_state_payload ---

    #[test]
    fn get_state_payload_empty() {
        let state = GovernanceState {
            lanes: vec![],
            nodes: vec![],
            observation_hints: ObservationHints {
                snapshot_freshness: "fresh".to_string(),
                verification_status: "unknown".to_string(),
                data_risk_candidates: 0,
                unused_files: 0,
            },
        };
        let payload = get_governance_state_payload("proj1", &state);
        assert_eq!(payload["project_id"], "proj1");
        assert!(payload["lanes"].as_array().unwrap().is_empty());
        assert!(payload["nodes"].as_array().unwrap().is_empty());
        assert_eq!(payload["observation_hints"]["snapshot_freshness"], "fresh");
        assert_eq!(
            payload["observation_hints"]["verification_status"],
            "unknown"
        );
        assert_eq!(payload["observation_hints"]["data_risk_candidates"], 0);
        assert_eq!(payload["observation_hints"]["unused_files"], 0);
    }

    #[test]
    fn get_state_payload_with_lanes_and_nodes() {
        let state = GovernanceState {
            lanes: vec![GovernanceLaneSummary {
                lane_id: "l1".to_string(),
                title: "Lane One".to_string(),
                status: "active".to_string(),
                node_count: 2,
                active_nodes: 1,
            }],
            nodes: vec![GovernanceNode {
                node_id: "nd1".to_string(),
                lane_id: "l1".to_string(),
                state: "in_progress".to_string(),
                summary: Some("doing work".to_string()),
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
                created_at: "2025-01-01".to_string(),
                updated_at: "2025-01-02".to_string(),
            }],
            observation_hints: ObservationHints {
                snapshot_freshness: "stale".to_string(),
                verification_status: "passed".to_string(),
                data_risk_candidates: 5,
                unused_files: 10,
            },
        };
        let payload = get_governance_state_payload("proj2", &state);
        let lanes = payload["lanes"].as_array().unwrap();
        assert_eq!(lanes.len(), 1);
        assert_eq!(lanes[0]["lane_id"], "l1");
        assert_eq!(lanes[0]["title"], "Lane One");
        assert_eq!(lanes[0]["node_count"], 2);
        assert_eq!(lanes[0]["active_nodes"], 1);

        let nodes = payload["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["node_id"], "nd1");
        assert_eq!(nodes[0]["summary"], "doing work");

        assert_eq!(payload["observation_hints"]["snapshot_freshness"], "stale");
        assert_eq!(payload["observation_hints"]["data_risk_candidates"], 5);
        assert_eq!(payload["observation_hints"]["unused_files"], 10);
    }

    #[test]
    fn get_state_payload_node_optional_fields_absent() {
        let state = GovernanceState {
            lanes: vec![],
            nodes: vec![GovernanceNode {
                node_id: "nd2".to_string(),
                lane_id: "l2".to_string(),
                state: "pending".to_string(),
                summary: None,
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
                created_at: "2025-01-01".to_string(),
                updated_at: "2025-01-01".to_string(),
            }],
            observation_hints: ObservationHints {
                snapshot_freshness: "fresh".to_string(),
                verification_status: "unknown".to_string(),
                data_risk_candidates: 0,
                unused_files: 0,
            },
        };
        let payload = get_governance_state_payload("p3", &state);
        let node = &payload["nodes"].as_array().unwrap()[0];
        // Optional fields should not appear when None
        assert!(node.get("summary").is_none());
        assert!(node.get("evidence_refs").is_none());
        assert!(node.get("artifact_refs").is_none());
        assert!(node.get("reported_git_head").is_none());
        assert!(node.get("suggested_next").is_none());
        assert!(node.get("forbidden_scope").is_none());
        assert!(node.get("external_anchors").is_none());
    }
}
