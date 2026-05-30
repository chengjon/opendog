use super::*;

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
