use super::*;

#[test]
fn decision_brief_payload_exposes_external_truth_boundary_for_verification_only() {
    let project_overview = fixtures::demo_project_overview();
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": "run_verification_before_high_risk_changes",
        "reason": "Verification evidence is missing.",
        "confidence": "high",
        "recommended_flow": ["Run project verification before broader edits."],
        "execution_sequence": {
            "mode": "run_project_verification_then_resume",
            "current_phase": "verify",
            "resume_with": "refresh_guidance_after_verification",
            "verification_commands": ["cargo test"],
            "resume_conditions": ["verification_evidence_fresh"]
        },
        "repo_truth_gaps": [],
        "mandatory_shell_checks": []
    });
    let agent_guidance = agent_guidance_payload(
        1,
        1,
        &["demo".to_string()],
        &["demo".to_string()],
        std::slice::from_ref(&recommendation),
        std::slice::from_ref(&project_overview),
        default_governance_layer(),
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
        brief["decision"]["external_truth_boundary"],
        json!({
            "status": "available",
            "source": "top_priority_project",
            "source_project_id": "demo",
            "mode": "must_switch_to_external_truth",
            "repo_state_required": false,
            "verification_required": true,
            "triggers": ["verification_run_required"],
            "minimum_external_checks": ["cargo test"],
            "summary": "Top project needs fresh project-native verification truth before broader changes."
        })
    );
}

#[test]
fn decision_brief_payload_keeps_not_git_repository_advisory_for_external_truth_boundary() {
    let project_overview = fixtures::demo_project_overview();
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": "review_unused_files",
        "reason": "Unused candidates should be reviewed first.",
        "confidence": "medium",
        "recommended_flow": ["Review the unused candidates before cleanup."],
        "execution_sequence": Value::Null,
        "repo_truth_gaps": ["not_git_repository"],
        "mandatory_shell_checks": []
    });
    let agent_guidance = agent_guidance_payload(
        1,
        1,
        &["demo".to_string()],
        &["demo".to_string()],
        std::slice::from_ref(&recommendation),
        std::slice::from_ref(&project_overview),
        default_governance_layer(),
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
        brief["decision"]["external_truth_boundary"],
        json!({
            "status": "available",
            "source": "top_priority_project",
            "source_project_id": "demo",
            "mode": "opendog_guidance_can_continue",
            "repo_state_required": false,
            "verification_required": false,
            "triggers": [],
            "minimum_external_checks": [],
            "summary": "Current top recommendation can continue under OPENDOG guidance until a repository or verification boundary is reached."
        })
    );
}

#[test]
fn decision_brief_payload_marks_external_truth_boundary_absent_when_no_priority_project() {
    let agent_guidance =
        agent_guidance_payload(0, 0, &[], &[], &[], &[], default_governance_layer());

    let brief = decision_brief_payload(
        MCP_DECISION_BRIEF_V1,
        "workspace",
        None,
        1,
        &agent_guidance,
        None,
    );

    assert_eq!(
        brief["layers"]["execution_strategy"]["external_truth_boundary"],
        json!({
            "status": "no_priority_project",
            "source": Value::Null,
            "source_project_id": Value::Null,
            "mode": Value::Null,
            "repo_state_required": false,
            "verification_required": false,
            "triggers": [],
            "minimum_external_checks": [],
            "summary": Value::Null
        })
    );
    assert_eq!(
        brief["decision"]["external_truth_boundary"],
        brief["layers"]["execution_strategy"]["external_truth_boundary"]
    );
}
