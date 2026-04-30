use super::transport::map_connect_error_with_liveness;
use super::*;
use crate::config::ProjectConfigPatch;
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
