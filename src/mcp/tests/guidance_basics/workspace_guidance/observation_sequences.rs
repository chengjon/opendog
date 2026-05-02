use super::*;

fn recommendation(
    project_id: &str,
    action: &str,
    sequence: serde_json::Value,
) -> serde_json::Value {
    json!({
        "project_id": project_id,
        "recommended_next_action": action,
        "reason": action,
        "confidence": "medium",
        "recommended_flow": [action],
        "execution_sequence": sequence,
        "repo_truth_gaps": [],
        "mandatory_shell_checks": []
    })
}

#[test]
fn agent_guidance_summarizes_observation_sequences() {
    let value = agent_guidance_payload(
        5,
        3,
        &[
            "monitor".to_string(),
            "snapshot".to_string(),
            "activity".to_string(),
        ],
        &[],
        &[
            recommendation(
                "monitor",
                "start_monitor",
                json!({
                    "mode": "start_monitor_then_resume",
                    "current_phase": "enable_monitoring",
                    "resume_with": "refresh_guidance_after_observation",
                    "observation_steps": ["start_monitor", "generate_real_project_activity"],
                    "resume_conditions": ["monitoring_active", "activity_evidence_recorded"]
                }),
            ),
            recommendation(
                "snapshot",
                "take_snapshot",
                json!({
                    "mode": "refresh_snapshot_then_resume",
                    "current_phase": "snapshot",
                    "resume_with": "refresh_guidance_after_snapshot",
                    "observation_steps": ["take_snapshot"],
                    "resume_conditions": ["snapshot_available", "snapshot_evidence_fresh"]
                }),
            ),
            recommendation(
                "activity",
                "generate_activity_then_stats",
                json!({
                    "mode": "generate_activity_then_resume",
                    "current_phase": "generate_activity",
                    "resume_with": "refresh_guidance_after_activity",
                    "observation_steps": ["generate_real_project_activity", "refresh_stats"],
                    "resume_conditions": ["activity_evidence_recorded", "activity_evidence_fresh"]
                }),
            ),
            recommendation(
                "verify",
                "run_verification_before_high_risk_changes",
                json!({
                    "mode": "run_project_verification_then_resume",
                    "current_phase": "verify",
                    "resume_with": "refresh_guidance_after_verification",
                    "verification_commands": ["cargo test"],
                    "resume_conditions": ["required_verification_recorded", "verification_evidence_fresh"]
                }),
            ),
            json!({
                "project_id": "stabilize",
                "recommended_next_action": "stabilize_repository_state",
                "reason": "stabilize_repository_state",
                "confidence": "high",
                "recommended_flow": ["stabilize_repository_state"],
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
                "monitor",
                "not_recorded",
                "missing",
                &[],
                false,
                false,
            ),
            workspace_verification_overview("snapshot", "available", "fresh", &[], false, false),
            workspace_verification_overview("activity", "available", "fresh", &[], false, false),
            workspace_verification_overview("verify", "not_recorded", "missing", &[], false, false),
            workspace_verification_overview("stabilize", "available", "fresh", &[], false, false),
        ],
    );

    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_monitor_start"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_snapshot_refresh"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_activity_generation"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_verification_run"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_repo_stabilization"],
        json!(1)
    );
}
