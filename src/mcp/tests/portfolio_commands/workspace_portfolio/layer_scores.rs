use super::*;

#[test]
fn workspace_portfolio_layer_exposes_attention_scores_and_reasons() {
    let value = workspace_portfolio_layer(&[json!({
        "project_id": "alpha",
        "unused_files": 2,
        "recommended_next_action": "run_verification_before_high_risk_changes",
        "observation": {
            "coverage_state": "stale_evidence",
            "freshness": {
                "snapshot": {"status": "fresh"},
                "activity": {"status": "fresh"},
                "verification": {"status": "missing"}
            }
        },
        "mock_data_summary": {
            "hardcoded_candidate_count": 2,
            "mock_candidate_count": 1
        },
        "verification_evidence": {
            "status": "not_recorded",
            "failing_runs": []
        },
        "repo_status_risk": {
            "status": "available",
            "risk_level": "medium",
            "is_dirty": true,
            "operation_states": []
        },
        "safe_for_cleanup": false,
        "safe_for_refactor": false
    })]);

    assert!(value["attention_queue"][0]["attention_score"].is_i64());
    assert!(value["attention_queue"][0]["attention_band"].is_string());
    assert!(value["attention_queue"][0]["attention_reasons"].is_array());
    assert_eq!(
        value["attention_queue"][0]["priority_basis"]["recommended_next_action"],
        json!("run_verification_before_high_risk_changes")
    );
}
