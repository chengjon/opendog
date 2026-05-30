use super::*;

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
