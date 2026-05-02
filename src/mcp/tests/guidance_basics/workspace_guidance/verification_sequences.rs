use super::*;

#[test]
fn agent_guidance_summarizes_verification_sequences() {
    let value = agent_guidance_payload(
        3,
        3,
        &[
            "missing".to_string(),
            "failing".to_string(),
            "stabilizing".to_string(),
        ],
        &[],
        &[
            json!({
                "project_id": "missing",
                "recommended_next_action": "run_verification_before_high_risk_changes",
                "reason": "Verification evidence is missing.",
                "confidence": "medium",
                "recommended_flow": ["Run verification before risky changes."],
                "execution_sequence": {
                    "mode": "run_project_verification_then_resume",
                    "current_phase": "verify",
                    "resume_with": "refresh_guidance_after_verification",
                    "verification_commands": ["cargo test"],
                    "resume_conditions": ["required_verification_recorded", "verification_evidence_fresh"]
                },
                "repo_truth_gaps": [],
                "mandatory_shell_checks": []
            }),
            json!({
                "project_id": "failing",
                "recommended_next_action": "review_failing_verification",
                "reason": "Verification evidence is failing.",
                "confidence": "high",
                "recommended_flow": ["Repair the failing verification first."],
                "execution_sequence": {
                    "mode": "resolve_failing_verification_then_resume",
                    "current_phase": "repair_and_verify",
                    "resume_with": "refresh_guidance_after_verification",
                    "verification_commands": ["cargo test -p api"],
                    "resume_conditions": ["no_failing_verification_runs", "verification_evidence_fresh"]
                },
                "repo_truth_gaps": [],
                "mandatory_shell_checks": []
            }),
            json!({
                "project_id": "stabilizing",
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
            }),
        ],
        &[
            workspace_verification_overview(
                "missing",
                "not_recorded",
                "missing",
                &[],
                false,
                false,
            ),
            workspace_verification_overview(
                "failing",
                "available",
                "fresh",
                &[json!({"kind": "test", "status": "failed"})],
                false,
                false,
            ),
            workspace_verification_overview("stabilizing", "available", "fresh", &[], false, false),
        ],
    );

    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_verification_run"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]
            ["projects_requiring_failing_verification_repair"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_repo_stabilization"],
        json!(1)
    );
}
