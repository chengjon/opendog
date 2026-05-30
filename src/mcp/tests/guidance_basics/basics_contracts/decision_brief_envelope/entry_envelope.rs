use super::*;

#[test]
fn decision_brief_payload_exposes_unified_entry_envelope() {
    let mut project_overview = fixtures::demo_project_overview();
    project_overview["repo_status_risk"]["risk_findings"] = json!([{
        "kind": "working_tree_conflicted",
        "severity": "high",
        "priority": "immediate",
        "confidence": "high",
        "summary": "2 conflicted paths detected in the working tree.",
        "evidence": ["git status reported 2 conflicted paths."],
        "source": "git_status"
    }]);
    project_overview["repo_status_risk"]["highest_priority_finding"] = json!({
        "kind": "working_tree_conflicted",
        "severity": "high",
        "priority": "immediate",
        "confidence": "high",
        "summary": "2 conflicted paths detected in the working tree.",
        "evidence": ["git status reported 2 conflicted paths."],
        "source": "git_status"
    });
    let recommendation = fixtures::demo_recommendation();
    let agent_guidance = agent_guidance_payload(
        1,
        1,
        &["demo".to_string()],
        &["demo".to_string()],
        std::slice::from_ref(&recommendation),
        std::slice::from_ref(&project_overview),
        default_governance_layer(),
    );
    let workspace_data_guidance = fixtures::demo_workspace_data_guidance();

    let brief = decision_brief_payload(
        MCP_DECISION_BRIEF_V1,
        "project",
        Some("demo"),
        1,
        &agent_guidance,
        Some(&workspace_data_guidance),
    );

    assert_eq!(brief["schema_version"], MCP_DECISION_BRIEF_V1);
    assert_eq!(brief["scope"], "project");
    assert_eq!(brief["selected_project_id"], "demo");
    assert_eq!(
        brief["decision"]["recommended_next_action"],
        "review_failing_verification"
    );
    assert_eq!(
        brief["decision"]["reason"],
        json!("Test evidence is failing.")
    );
    assert!(brief["decision"]["summary"]
        .as_str()
        .unwrap()
        .contains("2 conflicted paths detected in the working tree."));
    assert_eq!(
        brief["decision"]["repo_truth_gaps"],
        json!(["working_tree_conflicted"])
    );
    assert_eq!(
        brief["decision"]["mandatory_shell_checks"],
        json!(["git status", "git diff"])
    );
    assert_eq!(brief["decision"]["execution_sequence"], Value::Null);
    assert_eq!(
        brief["decision"]["data_risk_focus"],
        json!({
            "primary_focus": "hardcoded",
            "priority_order": ["hardcoded", "mixed", "mock"],
            "basis": [
                "hardcoded_candidates_present",
                "mixed_review_files_present",
                "runtime_shared_candidates_present",
                "high_severity_content_hits_present"
            ]
        })
    );
    assert_eq!(
        brief["decision"]["action_profile"]["action_class"],
        "verification_recovery"
    );
    assert_eq!(brief["decision"]["risk_profile"]["risk_tier"], "high");
    assert_eq!(
        brief["decision"]["risk_profile"]["verification_status"],
        "available"
    );
    assert_eq!(
        brief["decision"]["risk_profile"]["cleanup_gate_level"],
        json!("blocked")
    );
    assert_eq!(
        brief["decision"]["risk_profile"]["refactor_gate_level"],
        json!("blocked")
    );
    assert_eq!(
        brief["entrypoints"]["tool_selection_policy"]["preferred_primary_tool"],
        "shell"
    );
    assert!(brief["entrypoints"]["selection_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["target"] == "get_verification_status"));
    assert!(brief["entrypoints"]["next_mcp_tools"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item != "cleanup_project_data"));
    assert!(brief["entrypoints"]["next_cli_commands"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains(
            "opendog cleanup-data --id demo --scope all --older-than-days 30 --dry-run --json"
        )));
    assert!(brief["entrypoints"]["selection_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "cli_command"
            && item["target"].as_str().unwrap().contains(
                "opendog cleanup-data --id demo --scope all --older-than-days 30 --dry-run --json"
            )));
    assert!(brief["entrypoints"]["execution_templates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["template_id"] == "verification.review_status"));
    assert!(brief["entrypoints"]["execution_templates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["template_id"] == "storage.cleanup.preview"
            && item["kind"] == "cli_command"));
    assert!(brief["decision"]["signals"]["attention_score"].is_i64());
    assert_eq!(
        brief["decision"]["signals"]["storage_maintenance_candidate"],
        json!(true)
    );
    assert_eq!(
        brief["decision"]["signals"]["mixed_review_file_count"],
        json!(1)
    );
    assert_eq!(
        brief["decision"]["signals"]["storage_reclaimable_bytes"],
        json!(2048)
    );
    assert_eq!(
        brief["layers"]["workspace_observation"]["projects_with_hardcoded_candidates"],
        json!(1)
    );
    assert_eq!(
        brief["layers"]["workspace_observation"]["data_risk_focus_distribution"],
        json!({
            "hardcoded": 1,
            "mixed": 0,
            "mock": 0,
            "none": 0
        })
    );
    assert_eq!(
        brief["layers"]["execution_strategy"]["projects_requiring_hardcoded_review"],
        json!(1)
    );
    assert_eq!(
        brief["layers"]["constraints_boundaries"]["status"],
        json!("available")
    );
    assert!(brief["layers"]["constraints_boundaries"]["guardrails"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("verification")));
    assert!(brief["layers"]["execution_strategy"]["recommended_flow"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("verification")));
}
