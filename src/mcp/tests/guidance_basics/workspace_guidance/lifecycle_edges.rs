use super::*;

#[test]
fn agent_guidance_adapts_when_no_projects_exist() {
    let value = agent_guidance_payload(0, 0, &[], &[], &[], &[]);

    assert!(value["guidance"]["recommended_flow"][0]
        .as_str()
        .unwrap()
        .contains("Register a project first"));
}

#[test]
fn agent_guidance_prefers_snapshot_flow_when_top_project_lacks_baseline() {
    let value = agent_guidance_payload(
        1,
        1,
        &["demo".to_string()],
        &[],
        &[json!({
            "project_id": "demo",
            "recommended_next_action": "take_snapshot",
            "confidence": "medium"
        })],
        &[json!({
            "project_id": "demo",
            "safe_for_cleanup": false,
            "safe_for_refactor": false,
            "mock_data_summary": {
                "hardcoded_candidate_count": 0,
                "mock_candidate_count": 0
            },
            "verification_evidence": {
                "status": "not_recorded",
                "failing_runs": []
            },
            "repo_status_risk": {
                "status": "available",
                "risk_level": "low",
                "is_dirty": false,
                "operation_states": []
            }
        })],
    );

    assert!(value["guidance"]["recommended_flow"][0]
        .as_str()
        .unwrap()
        .contains("no snapshot baseline exists"));
    assert!(value["guidance"]["recommended_flow"][1]
        .as_str()
        .unwrap()
        .contains("snapshot --id demo"));
}
