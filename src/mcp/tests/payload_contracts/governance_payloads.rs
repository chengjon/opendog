use super::*;
use crate::core::governance::{
    GovernanceLaneSummary, GovernanceState, ObservationHints, UpsertNodeResult,
};
use crate::storage::queries::{GovernanceLane, GovernanceNode};

fn test_lane() -> GovernanceLane {
    GovernanceLane {
        lane_id: "test-lane".to_string(),
        title: "Test Lane".to_string(),
        description: Some("governance payload test".to_string()),
        status: "active".to_string(),
        created_at: "2026-05-25T00:00:00Z".to_string(),
        updated_at: "2026-05-25T00:00:00Z".to_string(),
    }
}

fn test_node() -> GovernanceNode {
    GovernanceNode {
        node_id: "node-1".to_string(),
        lane_id: "test-lane".to_string(),
        state: "open".to_string(),
        summary: Some("test node".to_string()),
        evidence_refs: None,
        artifact_refs: None,
        reported_git_head: Some("abc123".to_string()),
        suggested_next: None,
        forbidden_scope: None,
        external_anchors: None,
        created_at: "2026-05-25T00:00:00Z".to_string(),
        updated_at: "2026-05-25T00:00:00Z".to_string(),
    }
}

#[test]
fn create_governance_lane_payload_has_versioned_contract() {
    let value = create_governance_lane_payload("demo", &test_lane());
    assert_eq!(value["schema_version"], MCP_CREATE_GOVERNANCE_LANE_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["lane_id"], "test-lane");
    assert_eq!(value["title"], "Test Lane");
    assert_eq!(value["status"], "active");
    assert_eq!(value["created_at"], "2026-05-25T00:00:00Z");
}

#[test]
fn upsert_governance_node_payload_has_versioned_contract() {
    let result = UpsertNodeResult {
        node_id: "node-1".to_string(),
        lane_id: "test-lane".to_string(),
        state: "open".to_string(),
        created: true,
    };
    let value = upsert_governance_node_payload("demo", &result);
    assert_eq!(value["schema_version"], MCP_UPSERT_GOVERNANCE_NODE_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["node_id"], "node-1");
    assert_eq!(value["lane_id"], "test-lane");
    assert_eq!(value["state"], "open");
    assert_eq!(value["created"], true);
}

#[test]
fn get_governance_state_payload_has_versioned_contract() {
    let state = GovernanceState {
        lanes: vec![GovernanceLaneSummary {
            lane_id: "test-lane".to_string(),
            title: "Test Lane".to_string(),
            status: "active".to_string(),
            node_count: 1,
            active_nodes: 1,
        }],
        nodes: vec![test_node()],
        observation_hints: ObservationHints {
            snapshot_freshness: "fresh".to_string(),
            verification_status: "passed".to_string(),
            data_risk_candidates: 0,
            unused_files: 0,
        },
    };
    let value = get_governance_state_payload("demo", &state);
    assert_eq!(value["schema_version"], MCP_GET_GOVERNANCE_STATE_V1);
    assert_eq!(value["project_id"], "demo");
    assert!(value["lanes"].is_array());
    assert_eq!(value["lanes"][0]["lane_id"], "test-lane");
    assert_eq!(value["lanes"][0]["node_count"], 1);
    assert!(value["nodes"].is_array());
    assert_eq!(value["nodes"][0]["node_id"], "node-1");
    assert_eq!(value["nodes"][0]["reported_git_head"], "abc123");
    assert!(value["observation_hints"].is_object());
    assert_eq!(value["observation_hints"]["snapshot_freshness"], "fresh");
}

#[test]
fn close_governance_lane_payload_has_versioned_contract() {
    let value = close_governance_lane_payload("demo", "test-lane", "complete", "completed", 2);
    assert_eq!(value["schema_version"], MCP_CLOSE_GOVERNANCE_LANE_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["lane_id"], "test-lane");
    assert_eq!(value["action_taken"], "complete");
    assert_eq!(value["status"], "completed");
    assert_eq!(value["nodes_affected"], 2);
}
