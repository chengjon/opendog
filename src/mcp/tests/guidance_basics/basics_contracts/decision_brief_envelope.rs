use super::*;
use serde_json::Value;

#[path = "decision_brief_envelope/fixtures.rs"]
mod fixtures;

fn assert_selected_execution_sequence(
    recommendation: serde_json::Value,
    monitoring_count: usize,
    monitored_projects: &[String],
    expected_sequence: serde_json::Value,
) {
    let project_overview = fixtures::demo_project_overview();
    let agent_guidance = agent_guidance_payload(
        1,
        monitoring_count,
        monitored_projects,
        &[],
        std::slice::from_ref(&recommendation),
        std::slice::from_ref(&project_overview),
    );

    let brief = decision_brief_payload(
        MCP_DECISION_BRIEF_V1,
        "project",
        Some("demo"),
        1,
        &agent_guidance,
        None,
    );

    assert_eq!(brief["decision"]["execution_sequence"], expected_sequence);
}

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
        brief["decision"]["repo_truth_gaps"],
        json!(["working_tree_conflicted"])
    );
    assert_eq!(
        brief["decision"]["mandatory_shell_checks"],
        json!(["git status", "git diff"])
    );
    assert_eq!(brief["decision"]["execution_sequence"], Value::Null);
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

#[test]
fn decision_brief_payload_projects_selected_execution_sequence() {
    let project_overview = fixtures::demo_project_overview();
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": "stabilize_repository_state",
        "reason": "Repository is mid-operation.",
        "confidence": "high",
        "recommended_flow": ["Stabilize the repository before broader code changes."],
        "execution_sequence": {
            "mode": "shell_stabilize_then_resume",
            "current_phase": "stabilize",
            "resume_with": "refresh_guidance_after_repo_stable",
            "stability_checks": ["git status", "git diff"],
            "resume_conditions": ["operation_states_cleared", "conflicted_count_zero"]
        },
        "repo_truth_gaps": ["repository_mid_operation"],
        "mandatory_shell_checks": ["git status", "git diff"]
    });
    let agent_guidance = agent_guidance_payload(
        1,
        1,
        &["demo".to_string()],
        &["demo".to_string()],
        std::slice::from_ref(&recommendation),
        std::slice::from_ref(&project_overview),
    );

    let brief = decision_brief_payload(
        MCP_DECISION_BRIEF_V1,
        "project",
        Some("demo"),
        1,
        &agent_guidance,
        None,
    );

    assert_eq!(
        brief["decision"]["execution_sequence"],
        json!({
            "mode": "shell_stabilize_then_resume",
            "current_phase": "stabilize",
            "resume_with": "refresh_guidance_after_repo_stable",
            "stability_checks": ["git status", "git diff"],
            "resume_conditions": ["operation_states_cleared", "conflicted_count_zero"]
        })
    );
}

#[test]
fn decision_brief_payload_projects_selected_verification_sequence() {
    let project_overview = fixtures::demo_project_overview();
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": "review_failing_verification",
        "reason": "Test evidence is failing.",
        "confidence": "high",
        "recommended_flow": ["Repair the failing verification before broader review."],
        "execution_sequence": {
            "mode": "resolve_failing_verification_then_resume",
            "current_phase": "repair_and_verify",
            "resume_with": "refresh_guidance_after_verification",
            "verification_commands": ["cargo test -p api"],
            "resume_conditions": ["no_failing_verification_runs", "verification_evidence_fresh"]
        },
        "repo_truth_gaps": ["working_tree_conflicted"],
        "mandatory_shell_checks": ["git status", "git diff"]
    });
    let agent_guidance = agent_guidance_payload(
        1,
        1,
        &["demo".to_string()],
        &["demo".to_string()],
        std::slice::from_ref(&recommendation),
        std::slice::from_ref(&project_overview),
    );

    let brief = decision_brief_payload(
        MCP_DECISION_BRIEF_V1,
        "project",
        Some("demo"),
        1,
        &agent_guidance,
        None,
    );

    assert_eq!(
        brief["decision"]["execution_sequence"],
        json!({
            "mode": "resolve_failing_verification_then_resume",
            "current_phase": "repair_and_verify",
            "resume_with": "refresh_guidance_after_verification",
            "verification_commands": ["cargo test -p api"],
            "resume_conditions": ["no_failing_verification_runs", "verification_evidence_fresh"]
        })
    );
}

#[test]
fn decision_brief_payload_projects_selected_start_monitor_sequence() {
    let expected_sequence = json!({
        "mode": "start_monitor_then_resume",
        "current_phase": "enable_monitoring",
        "resume_with": "refresh_guidance_after_observation",
        "observation_steps": ["start_monitor", "generate_real_project_activity"],
        "resume_conditions": ["monitoring_active", "activity_evidence_recorded"]
    });
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": "start_monitor",
        "reason": "This project is not currently being monitored.",
        "confidence": "medium",
        "recommended_flow": ["Start monitoring first."],
        "execution_sequence": expected_sequence.clone(),
        "repo_truth_gaps": [],
        "mandatory_shell_checks": []
    });

    assert_selected_execution_sequence(recommendation, 0, &[], expected_sequence);
}

#[test]
fn decision_brief_payload_projects_selected_snapshot_sequence() {
    let expected_sequence = json!({
        "mode": "refresh_snapshot_then_resume",
        "current_phase": "snapshot",
        "resume_with": "refresh_guidance_after_snapshot",
        "observation_steps": ["take_snapshot"],
        "resume_conditions": ["snapshot_available", "snapshot_evidence_fresh"]
    });
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": "take_snapshot",
        "reason": "Snapshot evidence is missing.",
        "confidence": "medium",
        "recommended_flow": ["Take a snapshot before review."],
        "execution_sequence": expected_sequence.clone(),
        "repo_truth_gaps": [],
        "mandatory_shell_checks": []
    });

    assert_selected_execution_sequence(recommendation, 1, &["demo".to_string()], expected_sequence);
}
