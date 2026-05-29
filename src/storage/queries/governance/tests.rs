use super::*;
use crate::storage::database::Database;

fn test_db() -> Database {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("governance_test.db");
    let db = Database::open_project(&db_path).unwrap();
    Box::leak(Box::new(dir));
    db
}

#[test]
fn insert_and_read_lane() {
    let db = test_db();
    let now = "2026-05-24T12:00:00Z";

    let lane = NewGovernanceLane {
        lane_id: "lane-1".to_string(),
        title: "Test Lane".to_string(),
        description: Some("A test lane".to_string()),
    };
    insert_governance_lane(&db, &lane, now).unwrap();

    // Read back by id
    let found = get_governance_lane_by_id(&db, "lane-1").unwrap();
    assert!(found.is_some());
    let l = found.unwrap();
    assert_eq!(l.lane_id, "lane-1");
    assert_eq!(l.title, "Test Lane");
    assert_eq!(l.description, Some("A test lane".to_string()));
    assert_eq!(l.status, "active");
    assert_eq!(l.created_at, now);

    // List all lanes
    let lanes = get_governance_lanes(&db).unwrap();
    assert_eq!(lanes.len(), 1);

    // Not found returns None
    let missing = get_governance_lane_by_id(&db, "nope").unwrap();
    assert!(missing.is_none());

    // has_governance_data
    assert!(has_governance_data(&db).unwrap());
}

#[test]
fn upsert_creates_and_updates_node() {
    let db = test_db();
    let now = "2026-05-24T12:00:00Z";

    // Create lane first
    let lane = NewGovernanceLane {
        lane_id: "lane-2".to_string(),
        title: "Node Test".to_string(),
        description: None,
    };
    insert_governance_lane(&db, &lane, now).unwrap();

    // Insert node
    let created = upsert_governance_node(
        &db,
        &UpsertGovernanceNode {
            node_id: "node-1".to_string(),
            lane_id: "lane-2".to_string(),
            state: Some("open".to_string()),
            summary: Some("initial summary".to_string()),
            evidence_refs: None,
            artifact_refs: None,
            reported_git_head: None,
            suggested_next: None,
            forbidden_scope: None,
            external_anchors: None,
        },
        now,
    )
    .unwrap();
    assert!(created, "first upsert should create");

    // Update node — change state and summary, leave other fields as None (no overwrite)
    let updated = upsert_governance_node(
        &db,
        &UpsertGovernanceNode {
            node_id: "node-1".to_string(),
            lane_id: "lane-2".to_string(),
            state: Some("in_progress".to_string()),
            summary: None, // should retain old value
            evidence_refs: Some("ref-1".to_string()),
            artifact_refs: None,
            reported_git_head: None,
            suggested_next: Some("do-next".to_string()),
            forbidden_scope: None,
            external_anchors: None,
        },
        "2026-05-24T13:00:00Z",
    )
    .unwrap();
    assert!(!updated, "second upsert should update");

    // Verify fields
    let nodes = get_governance_nodes(&db, Some("lane-2"), None).unwrap();
    assert_eq!(nodes.len(), 1);
    let n = &nodes[0];
    assert_eq!(n.node_id, "node-1");
    assert_eq!(n.state, "in_progress");
    assert_eq!(
        n.summary,
        Some("initial summary".to_string()),
        "summary should be retained from first insert"
    );
    assert_eq!(n.evidence_refs, Some("ref-1".to_string()));
    assert_eq!(n.suggested_next, Some("do-next".to_string()));

    // Count active nodes
    assert_eq!(count_active_nodes_for_lane(&db, "lane-2").unwrap(), 1);
    assert_eq!(count_nodes_for_lane(&db, "lane-2").unwrap(), 1);
}

#[test]
fn close_lane_deletes_nodes_on_delete() {
    let db = test_db();
    let now = "2026-05-24T12:00:00Z";

    // Create lane
    let lane = NewGovernanceLane {
        lane_id: "lane-3".to_string(),
        title: "Delete Test".to_string(),
        description: None,
    };
    insert_governance_lane(&db, &lane, now).unwrap();

    // Create two nodes
    for i in 0..2 {
        upsert_governance_node(
            &db,
            &UpsertGovernanceNode {
                node_id: format!("node-del-{}", i),
                lane_id: "lane-3".to_string(),
                state: Some("open".to_string()),
                summary: None,
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
            },
            now,
        )
        .unwrap();
    }
    assert_eq!(count_nodes_for_lane(&db, "lane-3").unwrap(), 2);

    // Delete nodes then lane
    let deleted_nodes = delete_governance_nodes_by_lane(&db, "lane-3").unwrap();
    assert_eq!(deleted_nodes, 2);

    let deleted_lane = delete_governance_lane(&db, "lane-3").unwrap();
    assert_eq!(deleted_lane, 1);

    // Verify empty
    assert!(get_governance_lane_by_id(&db, "lane-3").unwrap().is_none());
    assert_eq!(count_nodes_for_lane(&db, "lane-3").unwrap(), 0);

    // Global counters
    assert_eq!(count_all_active_lanes(&db).unwrap(), 0);
    assert_eq!(count_all_active_nodes(&db).unwrap(), 0);
    assert!(!has_governance_data(&db).unwrap());
}

#[test]
fn get_governance_nodes_filters_by_node_id() {
    let db = test_db();
    let now = "2026-05-24T12:00:00Z";
    let lane = NewGovernanceLane {
        lane_id: "lane-filter".to_string(),
        title: "Filter Test".to_string(),
        description: None,
    };
    insert_governance_lane(&db, &lane, now).unwrap();

    for i in 0..3 {
        upsert_governance_node(
            &db,
            &UpsertGovernanceNode {
                node_id: format!("node-f-{}", i),
                lane_id: "lane-filter".to_string(),
                state: Some("open".to_string()),
                summary: None,
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
            },
            now,
        )
        .unwrap();
    }

    // Filter by node_id
    let nodes = get_governance_nodes(&db, None, Some("node-f-1")).unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].node_id, "node-f-1");

    // Filter by both lane_id and node_id
    let nodes = get_governance_nodes(&db, Some("lane-filter"), Some("node-f-2")).unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].node_id, "node-f-2");
}

#[test]
fn get_governance_nodes_returns_empty_for_no_match() {
    let db = test_db();
    let nodes = get_governance_nodes(&db, Some("nonexistent-lane"), None).unwrap();
    assert!(nodes.is_empty());

    let nodes = get_governance_nodes(&db, None, Some("nonexistent-node")).unwrap();
    assert!(nodes.is_empty());
}

#[test]
fn get_governance_nodes_no_filter_returns_all() {
    let db = test_db();
    let now = "2026-05-24T12:00:00Z";
    let lane = NewGovernanceLane {
        lane_id: "lane-all".to_string(),
        title: "All Test".to_string(),
        description: None,
    };
    insert_governance_lane(&db, &lane, now).unwrap();

    for i in 0..2 {
        upsert_governance_node(
            &db,
            &UpsertGovernanceNode {
                node_id: format!("node-all-{}", i),
                lane_id: "lane-all".to_string(),
                state: Some("open".to_string()),
                summary: None,
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
            },
            now,
        )
        .unwrap();
    }

    let nodes = get_governance_nodes(&db, None, None).unwrap();
    assert_eq!(nodes.len(), 2);
}

#[test]
fn update_lane_status_changes_status() {
    let db = test_db();
    let now = "2026-05-24T12:00:00Z";
    let lane = NewGovernanceLane {
        lane_id: "lane-status".to_string(),
        title: "Status Test".to_string(),
        description: None,
    };
    insert_governance_lane(&db, &lane, now).unwrap();

    let affected =
        update_lane_status(&db, "lane-status", "complete", "2026-05-24T14:00:00Z").unwrap();
    assert_eq!(affected, 1);

    let found = get_governance_lane_by_id(&db, "lane-status")
        .unwrap()
        .unwrap();
    assert_eq!(found.status, "complete");
    assert_eq!(found.updated_at, "2026-05-24T14:00:00Z");
}

#[test]
fn update_lane_status_nonexistent_affects_zero() {
    let db = test_db();
    let affected = update_lane_status(&db, "no-lane", "complete", "now").unwrap();
    assert_eq!(affected, 0);
}

#[test]
fn count_active_nodes_excludes_closed_state() {
    let db = test_db();
    let now = "2026-05-24T12:00:00Z";
    let lane = NewGovernanceLane {
        lane_id: "lane-active".to_string(),
        title: "Active Test".to_string(),
        description: None,
    };
    insert_governance_lane(&db, &lane, now).unwrap();

    // Open node
    upsert_governance_node(
        &db,
        &UpsertGovernanceNode {
            node_id: "node-open".to_string(),
            lane_id: "lane-active".to_string(),
            state: Some("open".to_string()),
            summary: None,
            evidence_refs: None,
            artifact_refs: None,
            reported_git_head: None,
            suggested_next: None,
            forbidden_scope: None,
            external_anchors: None,
        },
        now,
    )
    .unwrap();

    // Closed node
    upsert_governance_node(
        &db,
        &UpsertGovernanceNode {
            node_id: "node-closed".to_string(),
            lane_id: "lane-active".to_string(),
            state: Some("closed".to_string()),
            summary: None,
            evidence_refs: None,
            artifact_refs: None,
            reported_git_head: None,
            suggested_next: None,
            forbidden_scope: None,
            external_anchors: None,
        },
        now,
    )
    .unwrap();

    assert_eq!(count_active_nodes_for_lane(&db, "lane-active").unwrap(), 1);
    assert_eq!(count_nodes_for_lane(&db, "lane-active").unwrap(), 2);
    assert_eq!(count_all_active_nodes(&db).unwrap(), 1);
}

#[test]
fn count_all_active_lanes_counts_only_active() {
    let db = test_db();
    let now = "2026-05-24T12:00:00Z";

    for i in 0..3 {
        let lane = NewGovernanceLane {
            lane_id: format!("multi-lane-{}", i),
            title: format!("Lane {}", i),
            description: None,
        };
        insert_governance_lane(&db, &lane, now).unwrap();
    }
    assert_eq!(count_all_active_lanes(&db).unwrap(), 3);

    // Complete one lane
    update_lane_status(&db, "multi-lane-1", "complete", now).unwrap();
    assert_eq!(count_all_active_lanes(&db).unwrap(), 2);
}

#[test]
fn upsert_governance_node_updates_all_optional_fields() {
    let db = test_db();
    let now = "2026-05-24T12:00:00Z";
    let lane = NewGovernanceLane {
        lane_id: "lane-fields".to_string(),
        title: "Fields Test".to_string(),
        description: None,
    };
    insert_governance_lane(&db, &lane, now).unwrap();

    // Insert with minimal fields
    upsert_governance_node(
        &db,
        &UpsertGovernanceNode {
            node_id: "node-fields".to_string(),
            lane_id: "lane-fields".to_string(),
            state: Some("open".to_string()),
            summary: None,
            evidence_refs: None,
            artifact_refs: None,
            reported_git_head: None,
            suggested_next: None,
            forbidden_scope: None,
            external_anchors: None,
        },
        now,
    )
    .unwrap();

    // Update with all optional fields
    upsert_governance_node(
        &db,
        &UpsertGovernanceNode {
            node_id: "node-fields".to_string(),
            lane_id: "lane-fields".to_string(),
            state: Some("done".to_string()),
            summary: Some("updated summary".to_string()),
            evidence_refs: Some("evidence-1".to_string()),
            artifact_refs: Some("artifact-1".to_string()),
            reported_git_head: Some("abc123".to_string()),
            suggested_next: Some("review".to_string()),
            forbidden_scope: Some("production".to_string()),
            external_anchors: Some("anchor-1".to_string()),
        },
        "2026-05-24T14:00:00Z",
    )
    .unwrap();

    let nodes = get_governance_nodes(&db, Some("lane-fields"), None).unwrap();
    assert_eq!(nodes.len(), 1);
    let n = &nodes[0];
    assert_eq!(n.state, "done");
    assert_eq!(n.summary, Some("updated summary".to_string()));
    assert_eq!(n.evidence_refs, Some("evidence-1".to_string()));
    assert_eq!(n.artifact_refs, Some("artifact-1".to_string()));
    assert_eq!(n.reported_git_head, Some("abc123".to_string()));
    assert_eq!(n.suggested_next, Some("review".to_string()));
    assert_eq!(n.forbidden_scope, Some("production".to_string()));
    assert_eq!(n.external_anchors, Some("anchor-1".to_string()));
}

#[test]
fn delete_nonexistent_lane_and_nodes_is_safe() {
    let db = test_db();
    let deleted_nodes = delete_governance_nodes_by_lane(&db, "no-lane").unwrap();
    assert_eq!(deleted_nodes, 0);
    let deleted_lane = delete_governance_lane(&db, "no-lane").unwrap();
    assert_eq!(deleted_lane, 0);
}

#[test]
fn insert_lane_without_description() {
    let db = test_db();
    let now = "2026-05-24T12:00:00Z";
    let lane = NewGovernanceLane {
        lane_id: "lane-nodesc".to_string(),
        title: "No Description".to_string(),
        description: None,
    };
    insert_governance_lane(&db, &lane, now).unwrap();
    let found = get_governance_lane_by_id(&db, "lane-nodesc")
        .unwrap()
        .unwrap();
    assert_eq!(found.description, None);
}
