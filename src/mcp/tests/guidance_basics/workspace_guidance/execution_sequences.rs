use super::*;
use serde_json::Value;

#[test]
fn agent_guidance_summarizes_repo_stabilization_sequences() {
    let value = agent_guidance_payload(
        2,
        2,
        &["demo".to_string(), "steady".to_string()],
        &[],
        &[
            json!({
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
            }),
            json!({
                "project_id": "steady",
                "recommended_next_action": "review_unused_files",
                "reason": "Unused-file evidence is strong enough to review.",
                "confidence": "medium",
                "recommended_flow": ["Inspect unused-file candidates first."],
                "execution_sequence": Value::Null,
                "repo_truth_gaps": [],
                "mandatory_shell_checks": []
            })
        ],
        &[
            json!({
                "project_id": "demo",
                "safe_for_cleanup": false,
                "safe_for_refactor": false,
                "verification_evidence": { "status": "available", "failing_runs": [] },
                "repo_status_risk": { "status": "available", "risk_level": "high", "is_dirty": true, "operation_states": ["rebase"] },
                "mock_data_summary": { "hardcoded_candidate_count": 0, "mock_candidate_count": 0 },
                "storage_maintenance": { "maintenance_candidate": false, "vacuum_candidate": false, "approx_reclaimable_bytes": 0, "approx_db_size_bytes": 0 },
                "project_toolchain": { "project_type": "rust", "recommended_test_commands": ["cargo test"], "recommended_lint_commands": ["cargo clippy"], "recommended_build_commands": ["cargo check"] },
                "observation": {
                    "coverage_state": "ready",
                    "freshness": {
                        "snapshot": { "status": "fresh" },
                        "activity": { "status": "fresh" },
                        "verification": { "status": "fresh" }
                    }
                }
            }),
            json!({
                "project_id": "steady",
                "safe_for_cleanup": true,
                "safe_for_refactor": true,
                "verification_evidence": { "status": "available", "failing_runs": [] },
                "repo_status_risk": { "status": "available", "risk_level": "low", "is_dirty": false, "operation_states": [] },
                "mock_data_summary": { "hardcoded_candidate_count": 0, "mock_candidate_count": 0 },
                "storage_maintenance": { "maintenance_candidate": false, "vacuum_candidate": false, "approx_reclaimable_bytes": 0, "approx_db_size_bytes": 0 },
                "project_toolchain": { "project_type": "rust", "recommended_test_commands": ["cargo test"], "recommended_lint_commands": ["cargo clippy"], "recommended_build_commands": ["cargo check"] },
                "observation": {
                    "coverage_state": "ready",
                    "freshness": {
                        "snapshot": { "status": "fresh" },
                        "activity": { "status": "fresh" },
                        "verification": { "status": "fresh" }
                    }
                }
            })
        ],
    );

    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_repo_stabilization"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["repo_stabilization_priority_projects"],
        json!(["demo"])
    );
}
