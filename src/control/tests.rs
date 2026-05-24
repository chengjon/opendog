use super::transport::map_connect_error_with_liveness;
use super::*;
use crate::config::{ConfigPatch, ProjectConfigPatch};
use crate::control::client::decode_control_response;
use crate::core::governance::{
    CloseLaneInput, CreateLaneInput, GetGovernanceStateInput, UpsertNodeInput,
};
use crate::error::OpenDogError;
use serde_json::json;
use tempfile::TempDir;

fn test_controller() -> (TempDir, MonitorController) {
    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path().join("data");
    let project_root = dir.path().join("project");
    std::fs::create_dir_all(&project_root).unwrap();
    let pm = ProjectManager::with_data_dir(&data_dir).unwrap();
    pm.create("demo", &project_root).unwrap();
    (dir, MonitorController::with_project_manager(pm))
}

#[test]
fn handle_request_lists_no_monitors_initially() {
    let (_dir, mut controller) = test_controller();
    let response = controller.handle_request(ControlRequest::ListMonitors);

    match response {
        ControlResponse::Monitors { ids } => assert!(ids.is_empty()),
        other => panic!("unexpected response: {:?}", other),
    }
}

#[test]
fn decode_control_response_reports_empty_daemon_response_as_integrity_error() {
    let error = decode_control_response(&[]).unwrap_err();

    match error {
        OpenDogError::DaemonResponseIntegrity(message) => {
            assert!(message.contains("without returning a response"));
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn decode_control_response_reports_truncated_daemon_response_as_integrity_error() {
    let error = decode_control_response(br#"{"DecisionBrief":{"payload":"#).unwrap_err();

    match error {
        OpenDogError::DaemonResponseIntegrity(message) => {
            assert!(message.contains("incomplete JSON response"));
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn handle_request_returns_shared_decision_payloads() {
    let (_dir, mut controller) = test_controller();

    let guidance = controller.handle_request(ControlRequest::GetAgentGuidance {
        project: Some("demo".to_string()),
        top: 1,
    });
    match guidance {
        ControlResponse::AgentGuidance { payload } => {
            assert_eq!(payload["guidance"]["project_count"], 1);
            assert!(payload["guidance"]["recommended_flow"].is_array());
        }
        other => panic!("unexpected response: {:?}", other),
    }

    let brief = controller.handle_request(ControlRequest::GetDecisionBrief {
        project: Some("demo".to_string()),
        top: 1,
        schema_version: "opendog.test.decision-brief.v1".to_string(),
    });
    match brief {
        ControlResponse::DecisionBrief { payload } => {
            assert_eq!(payload["schema_version"], "opendog.test.decision-brief.v1");
            assert_eq!(payload["scope"], "project");
            assert_eq!(payload["selected_project_id"], "demo");
        }
        other => panic!("unexpected response: {:?}", other),
    }
}

#[test]
fn handle_request_returns_data_risk_payloads() {
    let (dir, mut controller) = test_controller();
    let project_root = dir.path().join("project");
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    std::fs::write(
        project_root.join("src/customer_seed.rs"),
        r#"const CUSTOMER: &str = "Acme Corp"; const EMAIL: &str = "ops@corp.com"; const ADDRESS: &str = "1 Market Street";"#,
    )
    .unwrap();
    controller.take_snapshot("demo").unwrap();

    let data_risk = controller.handle_request(ControlRequest::GetDataRiskCandidates {
        id: "demo".to_string(),
        candidate_type: "all".to_string(),
        min_review_priority: "low".to_string(),
        limit: 5,
        schema_version: "opendog.test.data-risk.v1".to_string(),
    });
    match data_risk {
        ControlResponse::DataRisk { payload } => {
            assert_eq!(payload["schema_version"], "opendog.test.data-risk.v1");
            assert_eq!(payload["project_id"], "demo");
            assert!(payload["hardcoded_candidate_count"].as_u64().unwrap_or(0) >= 1);
        }
        other => panic!("unexpected response: {:?}", other),
    }

    let workspace = controller.handle_request(ControlRequest::GetWorkspaceDataRiskOverview {
        candidate_type: "all".to_string(),
        min_review_priority: "low".to_string(),
        project_limit: 5,
        schema_version: "opendog.test.workspace-data-risk.v1".to_string(),
    });
    match workspace {
        ControlResponse::WorkspaceDataRisk { payload } => {
            assert_eq!(
                payload["schema_version"],
                "opendog.test.workspace-data-risk.v1"
            );
            assert_eq!(payload["total_registered_projects"], 1);
            assert_eq!(payload["matched_project_count"], 1);
        }
        other => panic!("unexpected response: {:?}", other),
    }
}

#[test]
fn handle_request_stop_reports_not_running_when_missing() {
    let (_dir, mut controller) = test_controller();
    let response = controller.handle_request(ControlRequest::StopMonitor {
        id: "demo".to_string(),
    });

    match response {
        ControlResponse::Stopped { was_running, .. } => assert!(!was_running),
        other => panic!("unexpected response: {:?}", other),
    }
}

#[test]
fn start_monitor_is_idempotent() {
    let (_dir, mut controller) = test_controller();
    let first = controller.start_monitor("demo").unwrap();
    let second = controller.start_monitor("demo").unwrap();

    assert!(!first.already_running);
    assert!(second.already_running);
    controller.stop_all();
}

#[test]
fn handle_request_start_returns_error_for_unknown_project() {
    let (_dir, mut controller) = test_controller();
    let response = controller.handle_request(ControlRequest::StartMonitor {
        id: "missing".to_string(),
    });

    match response {
        ControlResponse::Error { message } => assert!(message.contains("not found")),
        other => panic!("unexpected response: {:?}", other),
    }
}

#[test]
fn update_project_config_reloads_running_monitor_state() {
    let (_dir, mut controller) = test_controller();
    controller.start_monitor("demo").unwrap();

    let update = controller
        .update_project_config(
            "demo",
            ProjectConfigPatch {
                ignore_patterns: Some(vec!["logs".to_string()]),
                process_whitelist: Some(vec!["codex".to_string()]),
                inherit_ignore_patterns: false,
                inherit_process_whitelist: false,
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(update.effective.ignore_patterns, vec!["logs".to_string()]);
    assert_eq!(
        update.reload.changed_fields,
        vec![
            "ignore_patterns".to_string(),
            "process_whitelist".to_string()
        ]
    );
    assert!(update.reload.monitor_running);
    assert!(update.reload.runtime_reloaded);
    assert!(update.reload.snapshot_refreshed);

    let handle = controller.monitors.get("demo").unwrap();
    let live = handle.current_config();
    assert_eq!(live.ignore_patterns, vec!["logs".to_string()]);
    assert_eq!(live.process_whitelist, vec!["codex".to_string()]);
    controller.stop_all();
}

#[test]
fn reload_project_config_picks_up_latest_global_defaults() {
    let (_dir, mut controller) = test_controller();
    controller.start_monitor("demo").unwrap();

    controller
        .project_manager()
        .update_global_config(crate::config::ConfigPatch {
            ignore_patterns: Some(vec!["generated".to_string()]),
            process_whitelist: Some(vec!["claude".to_string()]),
            ..Default::default()
        })
        .unwrap();

    let outcome = controller.reload_project_config("demo").unwrap();
    assert!(outcome.monitor_running);
    assert!(outcome.runtime_reloaded);
    assert_eq!(
        outcome.changed_fields,
        vec![
            "ignore_patterns".to_string(),
            "process_whitelist".to_string()
        ]
    );

    let handle = controller.monitors.get("demo").unwrap();
    let live = handle.current_config();
    assert_eq!(live.ignore_patterns, vec!["generated".to_string()]);
    assert_eq!(live.process_whitelist, vec!["claude".to_string()]);
    controller.stop_all();
}

#[test]
fn handle_request_applies_incremental_global_config_updates() {
    let (_dir, mut controller) = test_controller();

    controller
        .project_manager()
        .update_global_config(ConfigPatch {
            ignore_patterns: Some(vec!["node_modules".to_string()]),
            ..Default::default()
        })
        .unwrap();

    let response = controller.handle_request(ControlRequest::UpdateGlobalConfig(ConfigPatch {
        ignore_patterns: None,
        process_whitelist: Some(vec!["claude".to_string(), "codex".to_string()]),
        add_ignore_patterns: vec!["logs".to_string()],
        remove_ignore_patterns: vec!["node_modules".to_string()],
        add_process_whitelist: vec!["roo".to_string()],
        remove_process_whitelist: vec!["claude".to_string()],
    }));

    match response {
        ControlResponse::GlobalConfigUpdated { result } => {
            assert_eq!(
                result.global_defaults.ignore_patterns,
                vec!["logs".to_string()]
            );
            assert_eq!(
                result.global_defaults.process_whitelist,
                vec!["codex".to_string(), "roo".to_string()]
            );
        }
        other => panic!("unexpected response: {:?}", other),
    }
}

#[test]
fn control_request_deserialization_defaults_omitted_incremental_patch_vectors() {
    let global_request = serde_json::from_value::<ControlRequest>(json!({
        "UpdateGlobalConfig": {
            "ignore_patterns": null,
            "process_whitelist": ["claude"]
        }
    }))
    .unwrap();

    match global_request {
        ControlRequest::UpdateGlobalConfig(patch) => {
            assert_eq!(patch.ignore_patterns, None);
            assert_eq!(patch.process_whitelist, Some(vec!["claude".to_string()]));
            assert!(patch.add_ignore_patterns.is_empty());
            assert!(patch.remove_ignore_patterns.is_empty());
            assert!(patch.add_process_whitelist.is_empty());
            assert!(patch.remove_process_whitelist.is_empty());
        }
        other => panic!("unexpected response: {:?}", other),
    }

    let project_request = serde_json::from_value::<ControlRequest>(json!({
        "UpdateProjectConfig": {
            "id": "demo",
            "ignore_patterns": null,
            "process_whitelist": ["codex"],
            "inherit_ignore_patterns": false,
            "inherit_process_whitelist": true
        }
    }))
    .unwrap();

    match project_request {
        ControlRequest::UpdateProjectConfig(fields) => {
            assert_eq!(fields.id, "demo");
            assert_eq!(fields.patch.ignore_patterns, None);
            assert_eq!(
                fields.patch.process_whitelist,
                Some(vec!["codex".to_string()])
            );
            assert!(fields.patch.add_ignore_patterns.is_empty());
            assert!(fields.patch.remove_ignore_patterns.is_empty());
            assert!(fields.patch.add_process_whitelist.is_empty());
            assert!(fields.patch.remove_process_whitelist.is_empty());
            assert!(!fields.patch.inherit_ignore_patterns);
            assert!(fields.patch.inherit_process_whitelist);
        }
        other => panic!("unexpected response: {:?}", other),
    }
}

#[test]
fn map_connect_error_marks_missing_socket_as_daemon_unavailable() {
    let error = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    assert!(matches!(
        map_connect_error_with_liveness(error, false),
        OpenDogError::DaemonUnavailable
    ));
}

#[test]
fn map_connect_error_marks_live_daemon_without_socket_as_control_unavailable() {
    let error = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    assert!(matches!(
        map_connect_error_with_liveness(error, true),
        OpenDogError::DaemonControlUnavailable
    ));
}

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
