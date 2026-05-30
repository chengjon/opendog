use super::*;

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
