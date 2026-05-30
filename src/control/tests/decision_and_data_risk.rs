use super::*;

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
