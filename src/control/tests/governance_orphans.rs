use super::*;

#[test]
fn handle_request_returns_governance_payloads() {
    let (_dir, mut controller) = test_controller();

    // Create lane
    let created = controller.handle_request(ControlRequest::CreateGovernanceLane {
        id: "demo".to_string(),
        input: CreateLaneInput {
            lane_id: "test-lane".to_string(),
            title: "Test lane".to_string(),
            description: Some("governance roundtrip test".to_string()),
        },
    });
    match created {
        ControlResponse::GovernanceLaneCreated { id, lane } => {
            assert_eq!(id, "demo");
            assert_eq!(lane.lane_id, "test-lane");
            assert_eq!(lane.title, "Test lane");
            assert_eq!(lane.status, "active");
        }
        other => panic!("unexpected create response: {:?}", other),
    }

    // Upsert node
    let upserted = controller.handle_request(ControlRequest::UpsertGovernanceNode {
        id: "demo".to_string(),
        input: UpsertNodeInput {
            node_id: "node-1".to_string(),
            lane_id: "test-lane".to_string(),
            state: Some("open".to_string()),
            summary: Some("test node".to_string()),
            evidence_refs: None,
            artifact_refs: None,
            reported_git_head: None,
            suggested_next: None,
            forbidden_scope: None,
            external_anchors: None,
        },
    });
    match upserted {
        ControlResponse::GovernanceNodeUpserted { id, result } => {
            assert_eq!(id, "demo");
            assert_eq!(result.node_id, "node-1");
            assert_eq!(result.state, "open");
            assert!(result.created);
        }
        other => panic!("unexpected upsert response: {:?}", other),
    }

    // Get state
    let state = controller.handle_request(ControlRequest::GetGovernanceState {
        id: "demo".to_string(),
        input: GetGovernanceStateInput {
            lane_id: None,
            node_id: None,
            active_only: None,
        },
    });
    match state {
        ControlResponse::GovernanceState { id, state } => {
            assert_eq!(id, "demo");
            assert_eq!(state.lanes.len(), 1);
            assert_eq!(state.lanes[0].lane_id, "test-lane");
            assert_eq!(state.nodes.len(), 1);
        }
        other => panic!("unexpected state response: {:?}", other),
    }

    // Close lane
    let closed = controller.handle_request(ControlRequest::CloseGovernanceLane {
        id: "demo".to_string(),
        input: CloseLaneInput {
            lane_id: "test-lane".to_string(),
            action: "complete".to_string(),
        },
    });
    match closed {
        ControlResponse::GovernanceLaneClosed {
            id,
            lane_id,
            status,
            nodes_affected,
            ..
        } => {
            assert_eq!(id, "demo");
            assert_eq!(lane_id, "test-lane");
            assert_eq!(status, "completed");
            assert_eq!(nodes_affected, 1);
        }
        other => panic!("unexpected close response: {:?}", other),
    }
}

#[test]
fn handle_request_returns_orphan_scan_and_deletion_plan_payloads() {
    let (_dir, mut controller) = test_controller();

    // Scan orphans (default input, no external reports)
    let scan = controller.handle_request(ControlRequest::ScanOrphans {
        id: "demo".to_string(),
        input: ScanOrphansInput {
            subjects: None,
            external_reports: Vec::new(),
            include_internal_scanners: true,
            required_scanners: None,
            max_age_secs: None,
            limit: Some(10),
            include_evidence: false,
        },
    });
    match scan {
        ControlResponse::OrphansScanned { id, result } => {
            assert_eq!(id, "demo");
            assert_eq!(result.status, "ok");
        }
        other => panic!("unexpected scan response: {:?}", other),
    }

    // Verify deletion plan (empty targets should return safe=false)
    let plan = controller.handle_request(ControlRequest::VerifyDeletionPlan {
        id: "demo".to_string(),
        input: DeletionPlanInput {
            targets: vec![OrphanSubject {
                subject_kind: OrphanSubjectKind::File,
                subject: "nonexistent.rs".to_string(),
                path: Some("nonexistent.rs".to_string()),
                display_name: None,
            }],
            external_reports: Vec::new(),
            required_project_verification_commands: Vec::new(),
            max_age_secs: None,
        },
    });
    match plan {
        ControlResponse::DeletionPlanVerified { id, result } => {
            assert_eq!(id, "demo");
            assert!(!result.safe_to_plan_deletion);
        }
        other => panic!("unexpected plan response: {:?}", other),
    }
}
