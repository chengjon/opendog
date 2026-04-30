use super::*;

#[test]
fn workspace_portfolio_prioritizes_high_risk_dirty_projects() {
    let value = agent_guidance_payload(
        2,
        1,
        &["alpha".to_string()],
        &["demo".to_string()],
        &[json!({
            "project_id": "alpha",
            "recommended_next_action": "review_unused_files",
            "reason": "unused files"
        })],
        &[
            json!({
                "project_id": "alpha",
                "unused_files": 10,
                "mock_data_summary": {
                    "hardcoded_candidate_count": 0,
                    "mock_candidate_count": 0
                },
                "repo_status_risk": {
                    "status": "available",
                    "risk_level": "high",
                    "is_dirty": true,
                    "operation_states": ["merge"]
                }
            }),
            json!({
                "project_id": "beta",
                "unused_files": 2,
                "mock_data_summary": {
                    "hardcoded_candidate_count": 2,
                    "mock_candidate_count": 2
                },
                "repo_status_risk": {
                    "status": "available",
                    "risk_level": "low",
                    "is_dirty": false,
                    "operation_states": []
                }
            }),
        ],
    );

    assert_eq!(
        value["guidance"]["layers"]["multi_project_portfolio"]["dirty_projects"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["multi_project_portfolio"]["projects_in_operation"][0]
            ["project_id"],
        json!("alpha")
    );
    assert_eq!(
        value["guidance"]["layers"]["multi_project_portfolio"]["attention_queue"][0]["project_id"],
        json!("alpha")
    );
}
