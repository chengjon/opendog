use super::*;

#[test]
fn workspace_data_risk_overview_payload_prioritizes_hardcoded_projects() {
    let payload = workspace_data_risk_overview_payload(
        &[
            json!({
                "project_id": "alpha",
                "mock_candidate_count": 1,
                "hardcoded_candidate_count": 0,
                "mixed_review_file_count": 0,
                "rule_groups_summary": [
                    {"group": "path", "severity": "low", "count": 1}
                ],
                "rule_hits_summary": [
                    {"rule": "path.mock_token", "group": "path", "severity": "low", "description": "mock", "count": 1}
                ],
            }),
            json!({
                "project_id": "beta",
                "mock_candidate_count": 2,
                "hardcoded_candidate_count": 3,
                "mixed_review_file_count": 1,
                "rule_groups_summary": [
                    {"group": "classification", "severity": "medium", "count": 2},
                    {"group": "content", "severity": "medium", "count": 1}
                ],
                "rule_hits_summary": [
                    {"rule": "path.runtime_shared", "group": "classification", "severity": "high", "description": "runtime", "count": 2},
                    {"rule": "content.business_literal_combo", "group": "content", "severity": "high", "description": "content", "count": 1}
                ],
            }),
        ],
        4,
    );

    assert_eq!(
        payload["layers"]["workspace_observation"]["projects_with_hardcoded_candidates"],
        json!(1)
    );
    assert_eq!(
        payload["layers"]["workspace_observation"]["total_registered_projects"],
        json!(4)
    );
    assert_eq!(
        payload["layers"]["workspace_observation"]["matched_project_count"],
        json!(2)
    );
    assert_eq!(
        payload["layers"]["multi_project_portfolio"]["rule_hits_summary"][0]["rule"],
        json!("path.runtime_shared")
    );
    assert_eq!(
        payload["layers"]["multi_project_portfolio"]["priority_projects"][0]["priority_reason"],
        json!("runtime-shared hardcoded candidates with high-severity content matches")
    );
    assert_eq!(
        payload["layers"]["multi_project_portfolio"]["priority_projects"][0]["project_id"],
        json!("beta")
    );
    assert_eq!(
        payload["layers"]["multi_project_portfolio"]["priority_projects"][0]["dominant_rule_group"]
            ["group"],
        json!("classification")
    );
    assert_eq!(
        payload["layers"]["execution_strategy"]["review_mock_data_before_cleanup"],
        json!(true)
    );
    assert_eq!(
        payload["recommended_flow"][0],
        json!("Start with the highest-priority project in the workspace queue.")
    );
}

#[test]
fn workspace_rule_aggregation_sums_counts_across_projects() {
    let payload = workspace_data_risk_overview_payload(
        &[
            json!({
                "project_id": "alpha",
                "mock_candidate_count": 1,
                "hardcoded_candidate_count": 1,
                "mixed_review_file_count": 0,
                "rule_groups_summary": [
                    {"group": "classification", "severity": "medium", "count": 1},
                    {"group": "content", "severity": "medium", "count": 1}
                ],
                "rule_hits_summary": [
                    {"rule": "path.runtime_shared", "group": "classification", "severity": "high", "description": "runtime", "count": 1},
                    {"rule": "content.business_literal_combo", "group": "content", "severity": "high", "description": "content", "count": 1}
                ],
            }),
            json!({
                "project_id": "beta",
                "mock_candidate_count": 0,
                "hardcoded_candidate_count": 2,
                "mixed_review_file_count": 1,
                "rule_groups_summary": [
                    {"group": "classification", "severity": "medium", "count": 2}
                ],
                "rule_hits_summary": [
                    {"rule": "path.runtime_shared", "group": "classification", "severity": "high", "description": "runtime", "count": 2}
                ],
            }),
        ],
        2,
    );

    assert_eq!(
        payload["layers"]["workspace_observation"]["rule_groups_summary"][0],
        json!({"group": "classification", "severity": "medium", "count": 3})
    );
    assert_eq!(
        payload["layers"]["multi_project_portfolio"]["rule_hits_summary"][0],
        json!({
            "rule": "path.runtime_shared",
            "group": "classification",
            "severity": "high",
            "description": "Candidate appears in a runtime/shared source path rather than a test-only area.",
            "count": 3
        })
    );
}
