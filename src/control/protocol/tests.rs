use super::*;
use serde_json::json;

// ---- ControlRequest round-trip tests ----

#[test]
fn request_ping_round_trip() {
    let req = ControlRequest::Ping;
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::Ping));
}

#[test]
fn request_create_project_round_trip() {
    let req = ControlRequest::CreateProject {
        id: "test".to_string(),
        path: "/tmp/test".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    if let ControlRequest::CreateProject { id, path } = back {
        assert_eq!(id, "test");
        assert_eq!(path, "/tmp/test");
    } else {
        panic!("expected CreateProject variant");
    }
}

#[test]
fn request_delete_project_round_trip() {
    let req = ControlRequest::DeleteProject {
        id: "x".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::DeleteProject { .. }));
}

#[test]
fn request_list_projects_round_trip() {
    let req = ControlRequest::ListProjects;
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::ListProjects));
}

#[test]
fn request_list_monitors_round_trip() {
    let req = ControlRequest::ListMonitors;
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::ListMonitors));
}

#[test]
fn request_get_stats_round_trip() {
    let req = ControlRequest::GetStats {
        id: "proj".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    if let ControlRequest::GetStats { id } = back {
        assert_eq!(id, "proj");
    } else {
        panic!("expected GetStats variant");
    }
}

#[test]
fn request_get_global_config_round_trip() {
    let req = ControlRequest::GetGlobalConfig;
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::GetGlobalConfig));
}

#[test]
fn request_get_project_config_round_trip() {
    let req = ControlRequest::GetProjectConfig {
        id: "p".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::GetProjectConfig { .. }));
}

#[test]
fn request_update_global_config_round_trip() {
    let patch = ConfigPatch {
        ignore_patterns: Some(vec!["*.log".to_string()]),
        process_whitelist: None,
        retention: None,
        add_ignore_patterns: vec![],
        remove_ignore_patterns: vec![],
        add_process_whitelist: vec![],
        remove_process_whitelist: vec![],
    };
    let req = ControlRequest::UpdateGlobalConfig(patch);
    let json_str = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json_str).unwrap();
    if let ControlRequest::UpdateGlobalConfig(p) = back {
        assert_eq!(p.ignore_patterns.as_ref().unwrap().len(), 1);
    } else {
        panic!("expected UpdateGlobalConfig variant");
    }
}

#[test]
fn request_update_project_config_round_trip() {
    let fields = UpdateProjectConfigFields {
        id: "proj1".to_string(),
        patch: ProjectConfigPatch::default(),
    };
    let req = ControlRequest::UpdateProjectConfig(fields);
    let json_str = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json_str).unwrap();
    if let ControlRequest::UpdateProjectConfig(f) = back {
        assert_eq!(f.id, "proj1");
    } else {
        panic!("expected UpdateProjectConfig variant");
    }
}

#[test]
fn request_reload_project_config_round_trip() {
    let req = ControlRequest::ReloadProjectConfig {
        id: "r".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::ReloadProjectConfig { .. }));
}

#[test]
fn request_get_unused_files_round_trip() {
    let req = ControlRequest::GetUnusedFiles {
        id: "u".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::GetUnusedFiles { .. }));
}

#[test]
fn request_get_time_window_report_round_trip() {
    let req = ControlRequest::GetTimeWindowReport {
        id: "tw".to_string(),
        window: "7d".to_string(),
        limit: 20,
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    if let ControlRequest::GetTimeWindowReport { id, window, limit } = back {
        assert_eq!(id, "tw");
        assert_eq!(window, "7d");
        assert_eq!(limit, 20);
    } else {
        panic!("expected GetTimeWindowReport variant");
    }
}

#[test]
fn request_compare_snapshots_round_trip() {
    let req = ControlRequest::CompareSnapshots {
        id: "cs".to_string(),
        base_run_id: Some(1),
        head_run_id: Some(2),
        limit: 10,
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    if let ControlRequest::CompareSnapshots {
        id,
        base_run_id,
        head_run_id,
        limit,
    } = back
    {
        assert_eq!(id, "cs");
        assert_eq!(base_run_id, Some(1));
        assert_eq!(head_run_id, Some(2));
        assert_eq!(limit, 10);
    } else {
        panic!("expected CompareSnapshots variant");
    }
}

#[test]
fn request_get_usage_trends_round_trip() {
    let req = ControlRequest::GetUsageTrends {
        id: "ut".to_string(),
        window: "24h".to_string(),
        limit: 5,
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    if let ControlRequest::GetUsageTrends { id, window, limit } = back {
        assert_eq!(id, "ut");
        assert_eq!(window, "24h");
        assert_eq!(limit, 5);
    } else {
        panic!("expected GetUsageTrends variant");
    }
}

#[test]
fn request_get_data_risk_candidates_round_trip() {
    let req = ControlRequest::GetDataRiskCandidates {
        id: "dr".to_string(),
        candidate_type: "all".to_string(),
        min_review_priority: "low".to_string(),
        limit: 50,
        schema_version: "v1".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    if let ControlRequest::GetDataRiskCandidates {
        id,
        candidate_type,
        schema_version,
        ..
    } = back
    {
        assert_eq!(id, "dr");
        assert_eq!(candidate_type, "all");
        assert_eq!(schema_version, "v1");
    } else {
        panic!("expected GetDataRiskCandidates variant");
    }
}

#[test]
fn request_get_workspace_data_risk_overview_round_trip() {
    let req = ControlRequest::GetWorkspaceDataRiskOverview {
        candidate_type: "mock".to_string(),
        min_review_priority: "medium".to_string(),
        project_limit: 10,
        schema_version: "v2".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    if let ControlRequest::GetWorkspaceDataRiskOverview {
        candidate_type,
        project_limit,
        ..
    } = back
    {
        assert_eq!(candidate_type, "mock");
        assert_eq!(project_limit, 10);
    } else {
        panic!("expected GetWorkspaceDataRiskOverview variant");
    }
}

#[test]
fn request_get_agent_guidance_round_trip() {
    let req = ControlRequest::GetAgentGuidance {
        project: Some("p".to_string()),
        top: 3,
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    if let ControlRequest::GetAgentGuidance { project, top } = back {
        assert_eq!(project, Some("p".to_string()));
        assert_eq!(top, 3);
    } else {
        panic!("expected GetAgentGuidance variant");
    }
}

#[test]
fn request_get_decision_brief_round_trip() {
    let req = ControlRequest::GetDecisionBrief {
        project: None,
        top: 5,
        schema_version: "v1".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    if let ControlRequest::GetDecisionBrief {
        project,
        top,
        schema_version,
    } = back
    {
        assert!(project.is_none());
        assert_eq!(top, 5);
        assert_eq!(schema_version, "v1");
    } else {
        panic!("expected GetDecisionBrief variant");
    }
}

#[test]
fn request_get_verification_status_round_trip() {
    let req = ControlRequest::GetVerificationStatus {
        id: "vs".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::GetVerificationStatus { .. }));
}

#[test]
fn request_start_monitor_round_trip() {
    let req = ControlRequest::StartMonitor {
        id: "sm".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::StartMonitor { .. }));
}

#[test]
fn request_stop_monitor_round_trip() {
    let req = ControlRequest::StopMonitor {
        id: "st".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::StopMonitor { .. }));
}

#[test]
fn request_take_snapshot_round_trip() {
    let req = ControlRequest::TakeSnapshot {
        id: "ts".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::TakeSnapshot { .. }));
}

// ---- ControlResponse round-trip tests ----

#[test]
fn response_pong_round_trip() {
    let resp = ControlResponse::Pong;
    let json = serde_json::to_string(&resp).unwrap();
    let back: ControlResponse = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlResponse::Pong));
}

#[test]
fn response_project_deleted_round_trip() {
    let resp = ControlResponse::ProjectDeleted {
        id: "x".to_string(),
        deleted: true,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: ControlResponse = serde_json::from_str(&json).unwrap();
    if let ControlResponse::ProjectDeleted { id, deleted } = back {
        assert_eq!(id, "x");
        assert!(deleted);
    } else {
        panic!("expected ProjectDeleted variant");
    }
}

#[test]
fn response_monitors_round_trip() {
    let resp = ControlResponse::Monitors {
        ids: vec!["a".to_string(), "b".to_string()],
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: ControlResponse = serde_json::from_str(&json).unwrap();
    if let ControlResponse::Monitors { ids } = back {
        assert_eq!(ids.len(), 2);
    } else {
        panic!("expected Monitors variant");
    }
}

#[test]
fn response_data_risk_round_trip() {
    let resp = ControlResponse::DataRisk {
        payload: json!({"status": "ok"}),
    };
    let json_str = serde_json::to_string(&resp).unwrap();
    let back: ControlResponse = serde_json::from_str(&json_str).unwrap();
    if let ControlResponse::DataRisk { payload } = back {
        assert_eq!(payload["status"], "ok");
    } else {
        panic!("expected DataRisk variant");
    }
}

#[test]
fn response_workspace_data_risk_round_trip() {
    let resp = ControlResponse::WorkspaceDataRisk {
        payload: json!({"count": 3}),
    };
    let json_str = serde_json::to_string(&resp).unwrap();
    let back: ControlResponse = serde_json::from_str(&json_str).unwrap();
    if let ControlResponse::WorkspaceDataRisk { payload } = back {
        assert_eq!(payload["count"], 3);
    } else {
        panic!("expected WorkspaceDataRisk variant");
    }
}

#[test]
fn response_agent_guidance_round_trip() {
    let resp = ControlResponse::AgentGuidance {
        payload: json!({"hint": "check files"}),
    };
    let json_str = serde_json::to_string(&resp).unwrap();
    let back: ControlResponse = serde_json::from_str(&json_str).unwrap();
    if let ControlResponse::AgentGuidance { payload } = back {
        assert_eq!(payload["hint"], "check files");
    } else {
        panic!("expected AgentGuidance variant");
    }
}

#[test]
fn response_decision_brief_round_trip() {
    let resp = ControlResponse::DecisionBrief {
        payload: json!({"decision": "proceed"}),
    };
    let json_str = serde_json::to_string(&resp).unwrap();
    let back: ControlResponse = serde_json::from_str(&json_str).unwrap();
    if let ControlResponse::DecisionBrief { payload } = back {
        assert_eq!(payload["decision"], "proceed");
    } else {
        panic!("expected DecisionBrief variant");
    }
}

#[test]
fn response_started_round_trip() {
    let resp = ControlResponse::Started {
        id: "s".to_string(),
        already_running: false,
        snapshot_taken: true,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: ControlResponse = serde_json::from_str(&json).unwrap();
    if let ControlResponse::Started {
        id,
        already_running,
        snapshot_taken,
    } = back
    {
        assert_eq!(id, "s");
        assert!(!already_running);
        assert!(snapshot_taken);
    } else {
        panic!("expected Started variant");
    }
}

#[test]
fn response_stopped_round_trip() {
    let resp = ControlResponse::Stopped {
        id: "s".to_string(),
        was_running: true,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: ControlResponse = serde_json::from_str(&json).unwrap();
    if let ControlResponse::Stopped { id, was_running } = back {
        assert_eq!(id, "s");
        assert!(was_running);
    } else {
        panic!("expected Stopped variant");
    }
}

#[test]
fn response_governance_lane_closed_round_trip() {
    let resp = ControlResponse::GovernanceLaneClosed {
        id: "p".to_string(),
        lane_id: "l".to_string(),
        action_taken: "complete".to_string(),
        status: "closed".to_string(),
        nodes_affected: 3,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: ControlResponse = serde_json::from_str(&json).unwrap();
    if let ControlResponse::GovernanceLaneClosed {
        id,
        lane_id,
        action_taken,
        status,
        nodes_affected,
    } = back
    {
        assert_eq!(id, "p");
        assert_eq!(lane_id, "l");
        assert_eq!(action_taken, "complete");
        assert_eq!(status, "closed");
        assert_eq!(nodes_affected, 3);
    } else {
        panic!("expected GovernanceLaneClosed variant");
    }
}

#[test]
fn response_error_round_trip() {
    let resp = ControlResponse::Error {
        message: "something went wrong".to_string(),
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: ControlResponse = serde_json::from_str(&json).unwrap();
    if let ControlResponse::Error { message } = back {
        assert_eq!(message, "something went wrong");
    } else {
        panic!("expected Error variant");
    }
}
