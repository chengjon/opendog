use super::*;

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
