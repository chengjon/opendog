use super::*;

#[test]
fn create_lane_and_upsert_node_flow() {
    let db = test_db();

    // Create lane
    let lane = create_lane(
        &db,
        CreateLaneInput {
            lane_id: "lane-flow".to_string(),
            title: "Happy path".to_string(),
            description: Some("full flow test".to_string()),
        },
    )
    .unwrap();
    assert_eq!(lane.lane_id, "lane-flow");
    assert_eq!(lane.title, "Happy path");
    assert_eq!(lane.status, "active");

    // Upsert node (create)
    let result = upsert_node(
        &db,
        UpsertNodeInput {
            node_id: "node-flow-1".to_string(),
            lane_id: "lane-flow".to_string(),
            state: Some("open".to_string()),
            summary: Some("first node".to_string()),
            evidence_refs: Some(vec!["e1".to_string(), "e2".to_string()]),
            artifact_refs: None,
            reported_git_head: Some("abc123".to_string()),
            suggested_next: None,
            forbidden_scope: Some(vec!["scope-a".to_string()]),
            external_anchors: Some(serde_json::json!({"link": "https://example.com"})),
        },
    )
    .unwrap();
    assert!(result.created);
    assert_eq!(result.node_id, "node-flow-1");
    assert_eq!(result.state, "open");

    // Upsert same node (update) — change state, leave summary unchanged
    let updated = upsert_node(
        &db,
        UpsertNodeInput {
            node_id: "node-flow-1".to_string(),
            lane_id: "lane-flow".to_string(),
            state: Some("in_progress".to_string()),
            summary: None,
            evidence_refs: None,
            artifact_refs: None,
            reported_git_head: None,
            suggested_next: None,
            forbidden_scope: None,
            external_anchors: None,
        },
    )
    .unwrap();
    assert!(!updated.created);
    assert_eq!(updated.state, "in_progress");

    // Verify via get_governance_state
    let state = get_governance_state(
        &db,
        GetGovernanceStateInput {
            lane_id: Some("lane-flow".to_string()),
            node_id: None,
            active_only: None,
        },
    )
    .unwrap();
    assert_eq!(state.nodes.len(), 1);
    assert_eq!(state.nodes[0].summary, Some("first node".to_string()));
    assert_eq!(state.lanes.len(), 1);
    assert_eq!(state.lanes[0].node_count, 1);
    assert_eq!(state.lanes[0].active_nodes, 1);
}

#[test]
fn upsert_rejects_missing_state_on_create() {
    let db = test_db();

    create_lane(
        &db,
        CreateLaneInput {
            lane_id: "lane-state".to_string(),
            title: "State required".to_string(),
            description: None,
        },
    )
    .unwrap();

    let err = upsert_node(
        &db,
        UpsertNodeInput {
            node_id: "node-no-state".to_string(),
            lane_id: "lane-state".to_string(),
            state: None,
            summary: None,
            evidence_refs: None,
            artifact_refs: None,
            reported_git_head: None,
            suggested_next: None,
            forbidden_scope: None,
            external_anchors: None,
        },
    )
    .unwrap_err();

    match err {
        OpenDogError::GovernanceNodeStateRequired(node_id) => {
            assert_eq!(node_id, "node-no-state");
        }
        other => panic!("expected GovernanceNodeStateRequired, got {:?}", other),
    }
}

#[test]
fn upsert_rejects_unknown_lane() {
    let db = test_db();

    let err = upsert_node(
        &db,
        UpsertNodeInput {
            node_id: "node-ghost".to_string(),
            lane_id: "no-such-lane".to_string(),
            state: Some("open".to_string()),
            summary: None,
            evidence_refs: None,
            artifact_refs: None,
            reported_git_head: None,
            suggested_next: None,
            forbidden_scope: None,
            external_anchors: None,
        },
    )
    .unwrap_err();

    match err {
        OpenDogError::GovernanceLaneNotFound(lane_id) => {
            assert_eq!(lane_id, "no-such-lane");
        }
        other => panic!("expected GovernanceLaneNotFound, got {:?}", other),
    }
}
