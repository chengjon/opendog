use super::*;

#[test]
fn agent_guidance_includes_shell_and_tool_advice() {
    let value = agent_guidance_payload(
        2,
        1,
        &["demo".to_string()],
        &["demo".to_string()],
        &[json!({
            "project_id": "demo",
            "recommended_next_action": "review_failing_verification",
            "reason": "Test evidence is failing.",
            "confidence": "high",
            "recommended_flow": ["Inspect verification state before broader edits."],
            "repo_truth_gaps": ["working_tree_conflicted"],
            "mandatory_shell_checks": ["git status", "git diff"]
        })],
        &[json!({
            "project_id": "demo",
            "safe_for_cleanup": false,
            "safe_for_refactor": false,
            "safe_for_cleanup_reason": "Verification is failing.",
            "safe_for_refactor_reason": "Verification is failing.",
            "verification_evidence": {
                "status": "available",
                "failing_runs": [{"kind":"test","status":"failed"}]
            },
            "repo_status_risk": {
                "status": "available",
                "risk_level": "medium",
                "is_dirty": true,
                "operation_states": []
            },
            "mock_data_summary": {
                "hardcoded_candidate_count": 1,
                "mock_candidate_count": 0,
                "data_risk_focus": {
                    "primary_focus": "hardcoded",
                    "priority_order": ["hardcoded", "mixed", "mock"],
                    "basis": [
                        "hardcoded_candidates_present",
                        "runtime_shared_candidates_present",
                        "high_severity_content_hits_present"
                    ]
                }
            },
            "storage_maintenance": {
                "maintenance_candidate": true,
                "approx_reclaimable_bytes": 1024,
                "reclaim_ratio": 0.25
            },
            "project_toolchain": {
                "project_type": "rust",
                "recommended_test_commands": ["cargo test"],
                "recommended_lint_commands": ["cargo clippy --all-targets --all-features -- -D warnings"],
                "recommended_build_commands": ["cargo check"]
            },
            "observation": {
                "coverage_state": "ready",
                "freshness": {
                    "snapshot": { "status": "fresh" },
                    "activity": { "status": "fresh" },
                    "verification": { "status": "fresh" }
                }
            }
        })],
    );

    assert_eq!(value["guidance"]["schema_version"], MCP_GUIDANCE_V1);
    assert_eq!(
        value["guidance"]["project_recommendations"][0]["project_id"],
        "demo"
    );
    assert!(value["guidance"]["example_commands"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("cargo test")));
    assert!(value["guidance"]["when_to_use_opendog"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("active, unused")));
    assert!(value["guidance"]["when_to_use_shell"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("change inspection")));
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["preferred_primary_tool"],
        json!("shell")
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["project_recommendations"][0]
            ["recommended_next_action"],
        json!("review_failing_verification")
    );
    assert_eq!(
        value["guidance"]["project_recommendations"][0]["reason"],
        value["guidance"]["layers"]["execution_strategy"]["project_recommendations"][0]["reason"]
    );
    assert!(
        value["guidance"]["layers"]["execution_strategy"]["recommended_flow"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item.as_str().unwrap().contains("verification"))
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_with_repo_truth_gaps"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["repo_truth_gap_distribution"]
            ["working_tree_conflicted"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["mandatory_shell_check_examples"],
        json!(["git status", "git diff"])
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["data_risk_focus_distribution"],
        json!({
            "hardcoded": 1,
            "mixed": 0,
            "mock": 0,
            "none": 0
        })
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_hardcoded_review"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_mock_review"],
        json!(0)
    );
    assert_eq!(
        value["guidance"]["layers"]["workspace_observation"]["projects_requiring_hardcoded_review"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["constraints_boundaries"]["status"],
        json!("available")
    );
}
