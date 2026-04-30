use super::*;

#[path = "decision_brief_envelope/fixtures.rs"]
mod fixtures;

#[test]
fn decision_brief_payload_exposes_unified_entry_envelope() {
    let project_overview = fixtures::demo_project_overview();
    let recommendation = fixtures::demo_recommendation();
    let agent_guidance = agent_guidance_payload(
        1,
        1,
        &["demo".to_string()],
        &["demo".to_string()],
        std::slice::from_ref(&recommendation),
        std::slice::from_ref(&project_overview),
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
    assert!(brief["entrypoints"]["execution_templates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["template_id"] == "verification.review_status"));
    assert!(brief["decision"]["signals"]["attention_score"].is_i64());
    assert_eq!(
        brief["decision"]["signals"]["storage_maintenance_candidate"],
        json!(true)
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
