use super::*;

#[test]
fn get_governance_nodes_returns_latest_updated_first() {
    const LANE_ID: &str = "lane-order";

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open_project(&db_path).unwrap();

    let now = "2026-01-01T00:00:00Z";
    let lane = NewGovernanceLane {
        lane_id: LANE_ID.to_string(),
        title: "Order test".to_string(),
        description: None,
    };
    queries::insert_governance_lane(&db, &lane, now).unwrap();

    let node_a = UpsertNodeInput {
        lane_id: LANE_ID.to_string(),
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
        lane_id: LANE_ID.to_string(),
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
        lane_id: LANE_ID.to_string(),
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

    let nodes = queries::get_governance_nodes(&db, Some(LANE_ID), None).unwrap();
    assert_eq!(nodes.len(), 2);
    assert_eq!(
        nodes[0].node_id, "node-a",
        "most recently updated node should be first"
    );
}

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
