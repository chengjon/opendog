use super::*;
use crate::storage::database::Database;

fn test_db() -> Database {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("governance_core_test.db");
    let db = Database::open_project(&db_path).unwrap();
    Box::leak(Box::new(dir));
    db
}

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

#[test]
fn close_lane_complete_marks_completed() {
    let db = test_db();

    create_lane(
        &db,
        CreateLaneInput {
            lane_id: "lane-complete".to_string(),
            title: "To complete".to_string(),
            description: None,
        },
    )
    .unwrap();

    upsert_node(
        &db,
        UpsertNodeInput {
            node_id: "node-c1".to_string(),
            lane_id: "lane-complete".to_string(),
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
    .unwrap();

    let (status, count) = close_lane(
        &db,
        CloseLaneInput {
            lane_id: "lane-complete".to_string(),
            action: "complete".to_string(),
        },
    )
    .unwrap();

    assert_eq!(status, "completed");
    assert_eq!(count, 1);

    // Verify lane status changed
    let lane = queries::get_governance_lane_by_id(&db, "lane-complete")
        .unwrap()
        .unwrap();
    assert_eq!(lane.status, "completed");
}

#[test]
fn close_lane_delete_removes_everything() {
    let db = test_db();

    create_lane(
        &db,
        CreateLaneInput {
            lane_id: "lane-delete".to_string(),
            title: "To delete".to_string(),
            description: None,
        },
    )
    .unwrap();

    // Create two nodes
    for i in 0..2 {
        upsert_node(
            &db,
            UpsertNodeInput {
                node_id: format!("node-d{}", i),
                lane_id: "lane-delete".to_string(),
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
        .unwrap();
    }

    let (status, count) = close_lane(
        &db,
        CloseLaneInput {
            lane_id: "lane-delete".to_string(),
            action: "delete".to_string(),
        },
    )
    .unwrap();

    assert_eq!(status, "deleted");
    assert_eq!(count, 2);

    // Lane is gone
    assert!(queries::get_governance_lane_by_id(&db, "lane-delete")
        .unwrap()
        .is_none());

    // Nodes are gone
    assert_eq!(
        queries::get_governance_nodes(&db, Some("lane-delete"), None)
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn get_governance_nodes_returns_latest_updated_first() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open_project(&db_path).unwrap();

    let now = "2026-01-01T00:00:00Z";
    let lane = NewGovernanceLane {
        lane_id: "lane-order".to_string(),
        title: "Order test".to_string(),
        description: None,
    };
    queries::insert_governance_lane(&db, &lane, now).unwrap();

    let node_a = UpsertNodeInput {
        lane_id: "lane-order".to_string(),
        node_id: "node-a".to_string(),
        state: Some("planning".to_string()),
        summary: Some("First node".to_string()),
        evidence_refs: None,
        artifact_refs: None,
        reported_git_head: None,
        suggested_next: None,
        forbidden_scope: None,
        external_anchors: None,
    };
    upsert_node(&db, node_a).unwrap();

    let node_b = UpsertNodeInput {
        lane_id: "lane-order".to_string(),
        node_id: "node-b".to_string(),
        state: Some("executing".to_string()),
        summary: Some("Second node".to_string()),
        evidence_refs: None,
        artifact_refs: None,
        reported_git_head: None,
        suggested_next: None,
        forbidden_scope: None,
        external_anchors: None,
    };
    upsert_node(&db, node_b).unwrap();

    // Update node-a so it has a later updated_at than node-b
    let node_a_update = UpsertNodeInput {
        lane_id: "lane-order".to_string(),
        node_id: "node-a".to_string(),
        state: Some("reviewing".to_string()),
        summary: Some("Updated first node".to_string()),
        evidence_refs: None,
        artifact_refs: None,
        reported_git_head: None,
        suggested_next: None,
        forbidden_scope: None,
        external_anchors: None,
    };
    upsert_node(&db, node_a_update).unwrap();

    let nodes = queries::get_governance_nodes(&db, Some("lane-order"), None).unwrap();
    assert_eq!(nodes.len(), 2);
    assert_eq!(
        nodes[0].node_id, "node-a",
        "most recently updated node should be first"
    );
}

#[test]
fn close_lane_rejects_invalid_action() {
    let db = test_db();
    create_lane(
        &db,
        CreateLaneInput {
            lane_id: "lane-bad-action".to_string(),
            title: "Test".to_string(),
            description: None,
        },
    )
    .unwrap();

    let err = close_lane(
        &db,
        CloseLaneInput {
            lane_id: "lane-bad-action".to_string(),
            action: "cancel".to_string(),
        },
    )
    .unwrap_err();

    match err {
        OpenDogError::InvalidInput(msg) => {
            assert!(msg.contains("invalid close action 'cancel'"));
        }
        other => panic!("expected InvalidInput, got {:?}", other),
    }
}

#[test]
fn close_lane_defer_marks_deferred() {
    let db = test_db();
    create_lane(
        &db,
        CreateLaneInput {
            lane_id: "lane-defer".to_string(),
            title: "To defer".to_string(),
            description: None,
        },
    )
    .unwrap();

    let (status, _count) = close_lane(
        &db,
        CloseLaneInput {
            lane_id: "lane-defer".to_string(),
            action: "defer".to_string(),
        },
    )
    .unwrap();

    assert_eq!(status, "deferred");
    let lane = queries::get_governance_lane_by_id(&db, "lane-defer")
        .unwrap()
        .unwrap();
    assert_eq!(lane.status, "deferred");
}

// -----------------------------------------------------------------------
// New integration tests
// -----------------------------------------------------------------------

/// A: get_governance_state with active_only=true filters closed nodes and lanes.
#[test]
fn get_state_active_only_filters_closed_nodes_and_lanes() {
    let db = test_db();

    // Create lane with two nodes: one open, one to be closed.
    create_lane(
        &db,
        CreateLaneInput {
            lane_id: "lane-ao".to_string(),
            title: "Active-only test".to_string(),
            description: None,
        },
    )
    .unwrap();

    upsert_node(
        &db,
        UpsertNodeInput {
            node_id: "node-open".to_string(),
            lane_id: "lane-ao".to_string(),
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
    .unwrap();

    upsert_node(
        &db,
        UpsertNodeInput {
            node_id: "node-closed".to_string(),
            lane_id: "lane-ao".to_string(),
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
    .unwrap();

    // Close one node.
    upsert_node(
        &db,
        UpsertNodeInput {
            node_id: "node-closed".to_string(),
            lane_id: "lane-ao".to_string(),
            state: Some("closed".to_string()),
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

    // active_only should exclude the closed node.
    let state = get_governance_state(
        &db,
        GetGovernanceStateInput {
            lane_id: None,
            node_id: None,
            active_only: Some(true),
        },
    )
    .unwrap();
    assert_eq!(
        state.nodes.len(),
        1,
        "active_only should filter out closed node"
    );
    assert_eq!(state.nodes[0].node_id, "node-open");
    assert_eq!(state.lanes.len(), 1, "lane is still active");

    // Now close the lane.
    close_lane(
        &db,
        CloseLaneInput {
            lane_id: "lane-ao".to_string(),
            action: "complete".to_string(),
        },
    )
    .unwrap();

    // active_only should now also filter out the completed lane.
    let state = get_governance_state(
        &db,
        GetGovernanceStateInput {
            lane_id: None,
            node_id: None,
            active_only: Some(true),
        },
    )
    .unwrap();
    assert!(
        state.lanes.is_empty(),
        "completed lane should be filtered out when active_only=true"
    );
}

/// B: get_governance_state with no filters returns all lanes and nodes.
#[test]
fn get_state_no_filters_returns_all() {
    let db = test_db();

    // Create two lanes with multiple nodes each.
    for lane_idx in 1..=2 {
        create_lane(
            &db,
            CreateLaneInput {
                lane_id: format!("lane-multi-{}", lane_idx),
                title: format!("Multi lane {}", lane_idx),
                description: None,
            },
        )
        .unwrap();

        for node_idx in 1..=3 {
            upsert_node(
                &db,
                UpsertNodeInput {
                    node_id: format!("node-l{}-n{}", lane_idx, node_idx),
                    lane_id: format!("lane-multi-{}", lane_idx),
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
            .unwrap();
        }
    }

    let state = get_governance_state(
        &db,
        GetGovernanceStateInput {
            lane_id: None,
            node_id: None,
            active_only: None,
        },
    )
    .unwrap();

    assert_eq!(state.lanes.len(), 2, "should return all lanes");
    assert_eq!(
        state.nodes.len(),
        6,
        "should return all nodes across both lanes"
    );
}

/// C: get_governance_state with node_id filter returns only that node.
#[test]
fn get_state_node_id_filter_returns_single_node() {
    let db = test_db();

    create_lane(
        &db,
        CreateLaneInput {
            lane_id: "lane-nf".to_string(),
            title: "Node filter test".to_string(),
            description: None,
        },
    )
    .unwrap();

    for name in &["alpha", "beta", "gamma"] {
        upsert_node(
            &db,
            UpsertNodeInput {
                node_id: format!("node-nf-{}", name),
                lane_id: "lane-nf".to_string(),
                state: Some("open".to_string()),
                summary: Some(format!("Node {}", name)),
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
            },
        )
        .unwrap();
    }

    let state = get_governance_state(
        &db,
        GetGovernanceStateInput {
            lane_id: None,
            node_id: Some("node-nf-beta".to_string()),
            active_only: None,
        },
    )
    .unwrap();

    assert_eq!(state.nodes.len(), 1, "should return only the filtered node");
    assert_eq!(state.nodes[0].node_id, "node-nf-beta");
    assert_eq!(state.nodes[0].summary, Some("Node beta".to_string()));
}

/// D: close_lane on non-existent lane returns GovernanceLaneNotFound.
#[test]
fn close_lane_nonexistent_returns_not_found() {
    let db = test_db();

    let err = close_lane(
        &db,
        CloseLaneInput {
            lane_id: "ghost-lane".to_string(),
            action: "complete".to_string(),
        },
    )
    .unwrap_err();

    match err {
        OpenDogError::GovernanceLaneNotFound(id) => {
            assert_eq!(id, "ghost-lane");
        }
        other => panic!("expected GovernanceLaneNotFound, got {:?}", other),
    }
}

/// E: compute_observation_hints coverage — seeds data and verifies hints.
#[test]
fn observation_hints_reflect_seeded_data() {
    use crate::storage::queries::{insert_snapshot_batch, insert_verification_run, SnapshotEntry};

    let db = test_db();

    // Seed snapshot data so snapshot_freshness is "fresh".
    insert_snapshot_batch(
        &db,
        &[SnapshotEntry {
            path: "src/main.rs".to_string(),
            size: 100,
            mtime: 1,
            file_type: "rs".to_string(),
            scan_timestamp: "1000".to_string(),
        }],
    )
    .unwrap();

    // Seed a verification run so verification_status is "passed".
    insert_verification_run(
        &db,
        &crate::storage::queries::NewVerificationRun {
            kind: "test".to_string(),
            status: "passed".to_string(),
            command: "cargo test".to_string(),
            exit_code: Some(0),
            summary: None,
            source: "test".to_string(),
            started_at: None,
            finished_at: "1000".to_string(),
        },
    )
    .unwrap();

    // Create a governance lane so the overall state has structure.
    create_lane(
        &db,
        CreateLaneInput {
            lane_id: "lane-hints".to_string(),
            title: "Hints test".to_string(),
            description: None,
        },
    )
    .unwrap();

    upsert_node(
        &db,
        UpsertNodeInput {
            node_id: "node-hints".to_string(),
            lane_id: "lane-hints".to_string(),
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
    .unwrap();

    let state = get_governance_state(
        &db,
        GetGovernanceStateInput {
            lane_id: None,
            node_id: None,
            active_only: None,
        },
    )
    .unwrap();

    let hints = &state.observation_hints;
    assert_eq!(hints.snapshot_freshness, "fresh");
    assert_eq!(hints.verification_status, "passed");
    // The snapshot entry has no matching file_stats row, so it counts as unused.
    assert_eq!(
        hints.unused_files, 1,
        "one snapshot file with no file_stats"
    );
    assert_eq!(hints.data_risk_candidates, 0, "no data-risk cache seeded");

    // Structural sanity: the lane and node are returned.
    assert_eq!(state.lanes.len(), 1);
    assert_eq!(state.nodes.len(), 1);
}

// -----------------------------------------------------------------------
// Pure helper function tests
// -----------------------------------------------------------------------

#[test]
fn json_to_string_with_some_object() {
    use serde_json::json;
    let result = json_to_string(&Some(json!({"key": "value"})));
    assert!(result.is_some());
    assert!(
        result.as_ref().unwrap().contains("\"key\""),
        "result should contain key: {:?}",
        result
    );
    assert!(
        result.as_ref().unwrap().contains("\"value\""),
        "result should contain value: {:?}",
        result
    );
}

#[test]
fn json_to_string_with_some_string() {
    use serde_json::json;
    let result = json_to_string(&Some(json!("hello")));
    assert_eq!(result, Some("\"hello\"".to_string()));
}

#[test]
fn json_to_string_with_none() {
    let result: Option<String> = json_to_string(&None);
    assert!(result.is_none());
}

#[test]
fn json_to_string_with_some_number() {
    use serde_json::json;
    let result = json_to_string(&Some(json!(42)));
    assert_eq!(result, Some("42".to_string()));
}

#[test]
fn string_list_to_json_with_some_list() {
    let input = Some(vec!["a".to_string(), "b".to_string()]);
    let result = string_list_to_json(&input);
    assert!(result.is_some());
    let parsed: Vec<String> = serde_json::from_str(result.as_ref().unwrap()).unwrap();
    assert_eq!(parsed, vec!["a", "b"]);
}

#[test]
fn string_list_to_json_with_empty_list() {
    let input: Option<Vec<String>> = Some(vec![]);
    let result = string_list_to_json(&input);
    assert_eq!(result, Some("[]".to_string()));
}

#[test]
fn string_list_to_json_with_none() {
    let input: Option<Vec<String>> = None;
    let result = string_list_to_json(&input);
    assert!(result.is_none());
}

#[test]
fn string_list_to_json_with_strings_containing_special_chars() {
    let input = Some(vec!["hello world".to_string(), "a\"b".to_string()]);
    let result = string_list_to_json(&input);
    assert!(result.is_some());
    let parsed: Vec<String> = serde_json::from_str(result.as_ref().unwrap()).unwrap();
    assert_eq!(parsed, vec!["hello world", "a\"b"]);
}
