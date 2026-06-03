use super::*;

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
    const LANE_ID: &str = "lane-delete";

    let db = test_db();

    create_lane(
        &db,
        CreateLaneInput {
            lane_id: LANE_ID.to_string(),
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
                lane_id: LANE_ID.to_string(),
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
            lane_id: LANE_ID.to_string(),
            action: "delete".to_string(),
        },
    )
    .unwrap();

    assert_eq!(status, "deleted");
    assert_eq!(count, 2);

    // Lane is gone
    assert!(queries::get_governance_lane_by_id(&db, LANE_ID)
        .unwrap()
        .is_none());

    // Nodes are gone
    assert_eq!(
        queries::get_governance_nodes(&db, Some(LANE_ID), None)
            .unwrap()
            .len(),
        0
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
