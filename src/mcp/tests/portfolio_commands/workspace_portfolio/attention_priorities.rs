use super::*;

#[test]
fn workspace_portfolio_prioritizes_attention_urgency_over_data_risk_volume() {
    let value = agent_guidance_payload(
        2,
        2,
        &["alpha".to_string(), "beta".to_string()],
        &["demo".to_string()],
        &[
            json!({
                "project_id": "alpha",
                "recommended_next_action": "review_failing_verification",
                "confidence": "high"
            }),
            json!({
                "project_id": "beta",
                "recommended_next_action": "inspect_hot_files",
                "confidence": "high"
            }),
        ],
        &[
            json!({
                "project_id": "alpha",
                "unused_files": 1,
                "observation": {
                    "coverage_state": "ready",
                    "freshness": {
                        "snapshot": {"status": "fresh"},
                        "activity": {"status": "fresh"},
                        "verification": {"status": "fresh"}
                    }
                },
                "mock_data_summary": {
                    "hardcoded_candidate_count": 0,
                    "mock_candidate_count": 0
                },
                "verification_evidence": {
                    "status": "available",
                    "failing_runs": [{"kind": "test"}]
                },
                "repo_status_risk": {
                    "status": "available",
                    "risk_level": "medium",
                    "is_dirty": false,
                    "operation_states": []
                },
                "safe_for_cleanup": true,
                "safe_for_refactor": true
            }),
            json!({
                "project_id": "beta",
                "unused_files": 1,
                "observation": {
                    "coverage_state": "ready",
                    "freshness": {
                        "snapshot": {"status": "fresh"},
                        "activity": {"status": "fresh"},
                        "verification": {"status": "fresh"}
                    }
                },
                "mock_data_summary": {
                    "hardcoded_candidate_count": 3,
                    "mock_candidate_count": 4
                },
                "verification_evidence": {
                    "status": "available",
                    "failing_runs": []
                },
                "repo_status_risk": {
                    "status": "available",
                    "risk_level": "low",
                    "is_dirty": false,
                    "operation_states": []
                },
                "safe_for_cleanup": true,
                "safe_for_refactor": true
            }),
        ],
    );

    assert_eq!(
        value["guidance"]["layers"]["multi_project_portfolio"]["attention_queue"][0]["project_id"],
        json!("alpha")
    );
    assert_eq!(
        value["guidance"]["layers"]["multi_project_portfolio"]["attention_batches"]["immediate"][0]
            ["project_id"],
        json!("alpha")
    );
    assert_eq!(
        value["guidance"]["layers"]["multi_project_portfolio"]["priority_candidates"][0]
            ["project_id"],
        json!("alpha")
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]
            ["projects_with_hardcoded_data_candidates"],
        json!(1)
    );
}
