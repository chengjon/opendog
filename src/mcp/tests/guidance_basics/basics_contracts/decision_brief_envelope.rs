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
    let mut project_overview = fixtures::demo_project_overview();
    project_overview["repo_status_risk"]["risk_findings"] = json!([{
        "kind": "working_tree_conflicted",
        "severity": "high",
        "priority": "immediate",
        "confidence": "high",
        "summary": "2 conflicted paths detected in the working tree."
    }]);
    project_overview["repo_status_risk"]["highest_priority_finding"] = json!({
        "kind": "working_tree_conflicted",
        "severity": "high",
        "priority": "immediate",
        "confidence": "high",
        "summary": "2 conflicted paths detected in the working tree."
    });
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
    let agent_guidance = agent_guidance_payload(0, 0, &[], &[], &[], &[]);

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

#[test]
fn decision_brief_review_focus_projection_mirrors_review_focus_for_unused_review() {
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": "review_unused_files",
        "reason": "Unused candidates should be reviewed first.",
        "confidence": "medium",
        "recommended_flow": ["Review the unused candidates before cleanup."],
        "execution_sequence": Value::Null,
        "repo_truth_gaps": [],
        "mandatory_shell_checks": [],
        "review_focus": {
            "candidate_family": "unused_candidate",
            "candidate_basis": ["zero_recorded_access", "snapshot_present"],
            "candidate_risk_hints": []
        }
    });
    let mut project_overview = fixtures::demo_project_overview();
    project_overview["recommended_next_action"] = json!("review_unused_files");
    project_overview["recommended_reason"] = json!("Unused candidates should be reviewed first.");
    project_overview["safe_for_cleanup"] = json!(true);
    project_overview["safe_for_refactor"] = json!(true);
    project_overview["safe_for_cleanup_reason"] = json!("No cleanup blockers are present.");
    project_overview["safe_for_refactor_reason"] = json!("No refactor blockers are present.");
    project_overview["cleanup_blockers"] = json!([]);
    project_overview["refactor_blockers"] = json!([]);
    project_overview["verification_gate_levels"]["cleanup"] = json!("allow");
    project_overview["verification_gate_levels"]["refactor"] = json!("allow");
    project_overview["verification_evidence"]["status"] = json!("available");
    project_overview["verification_evidence"]["failing_runs"] = json!([]);
    project_overview["verification_evidence"]["gate_assessment"]["cleanup"]["level"] =
        json!("allow");
    project_overview["verification_evidence"]["gate_assessment"]["refactor"]["level"] =
        json!("allow");
    project_overview["repo_status_risk"]["risk_level"] = json!("low");
    project_overview["repo_status_risk"]["is_dirty"] = json!(false);
    project_overview["repo_status_risk"]["risk_findings"] = json!([]);
    project_overview["repo_status_risk"]["highest_priority_finding"] = Value::Null;
    project_overview["mock_data_summary"]["hardcoded_candidate_count"] = json!(0);
    project_overview["mock_data_summary"]["mock_candidate_count"] = json!(0);
    project_overview["mock_data_summary"]["mixed_review_file_count"] = json!(0);
    project_overview["mock_data_summary"]["data_risk_focus"]["primary_focus"] = json!("none");
    project_overview["mock_data_summary"]["data_risk_focus"]["basis"] = json!([]);
    project_overview["storage_maintenance"]["maintenance_candidate"] = json!(false);
    project_overview["storage_maintenance"]["vacuum_candidate"] = json!(false);
    project_overview["storage_maintenance"]["approx_reclaimable_bytes"] = json!(0);
    project_overview["storage_maintenance"]["approx_db_size_bytes"] = json!(0);
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
        brief["decision"]["review_focus"],
        json!({
            "candidate_family": "unused_candidate",
            "candidate_basis": ["zero_recorded_access", "snapshot_present"],
            "candidate_risk_hints": []
        })
    );
    assert_eq!(
        brief["layers"]["execution_strategy"]["review_focus_projection"],
        json!({
            "status": "available",
            "source": "top_priority_project",
            "source_project_id": "demo",
            "review_focus": {
                "candidate_family": "unused_candidate",
                "candidate_basis": ["zero_recorded_access", "snapshot_present"],
                "candidate_risk_hints": []
            }
        })
    );
    assert_eq!(
        brief["decision"]["review_focus"],
        brief["layers"]["execution_strategy"]["review_focus_projection"]["review_focus"]
    );
}

#[test]
fn decision_brief_review_focus_projection_keeps_review_focus_null_for_non_review_action() {
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": "start_monitor",
        "reason": "This project is not currently being monitored.",
        "confidence": "medium",
        "recommended_flow": ["Start monitoring first."],
        "execution_sequence": Value::Null,
        "repo_truth_gaps": [],
        "mandatory_shell_checks": [],
        "review_focus": Value::Null
    });
    let mut project_overview = fixtures::demo_project_overview();
    project_overview["recommended_next_action"] = json!("start_monitor");
    project_overview["recommended_reason"] =
        json!("This project is not currently being monitored.");
    project_overview["safe_for_cleanup"] = json!(true);
    project_overview["safe_for_refactor"] = json!(true);
    project_overview["safe_for_cleanup_reason"] = json!("No cleanup blockers are present.");
    project_overview["safe_for_refactor_reason"] = json!("No refactor blockers are present.");
    project_overview["cleanup_blockers"] = json!([]);
    project_overview["refactor_blockers"] = json!([]);
    project_overview["verification_gate_levels"]["cleanup"] = json!("allow");
    project_overview["verification_gate_levels"]["refactor"] = json!("allow");
    project_overview["verification_evidence"]["status"] = json!("available");
    project_overview["verification_evidence"]["failing_runs"] = json!([]);
    project_overview["verification_evidence"]["gate_assessment"]["cleanup"]["level"] =
        json!("allow");
    project_overview["verification_evidence"]["gate_assessment"]["refactor"]["level"] =
        json!("allow");
    project_overview["repo_status_risk"]["risk_level"] = json!("low");
    project_overview["repo_status_risk"]["is_dirty"] = json!(false);
    project_overview["repo_status_risk"]["risk_findings"] = json!([]);
    project_overview["repo_status_risk"]["highest_priority_finding"] = Value::Null;
    project_overview["mock_data_summary"]["hardcoded_candidate_count"] = json!(0);
    project_overview["mock_data_summary"]["mock_candidate_count"] = json!(0);
    project_overview["mock_data_summary"]["mixed_review_file_count"] = json!(0);
    project_overview["mock_data_summary"]["data_risk_focus"]["primary_focus"] = json!("none");
    project_overview["mock_data_summary"]["data_risk_focus"]["basis"] = json!([]);
    project_overview["storage_maintenance"]["maintenance_candidate"] = json!(false);
    project_overview["storage_maintenance"]["vacuum_candidate"] = json!(false);
    project_overview["storage_maintenance"]["approx_reclaimable_bytes"] = json!(0);
    project_overview["storage_maintenance"]["approx_db_size_bytes"] = json!(0);
    let agent_guidance = agent_guidance_payload(
        1,
        0,
        &[],
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

    assert_eq!(
        brief["layers"]["execution_strategy"]["review_focus_projection"],
        json!({
            "status": "available",
            "source": "top_priority_project",
            "source_project_id": "demo",
            "review_focus": Value::Null
        })
    );
    assert_eq!(brief["decision"]["review_focus"], Value::Null);
}

#[test]
fn decision_brief_marks_review_focus_projection_absent_when_no_priority_project() {
    let agent_guidance = agent_guidance_payload(0, 0, &[], &[], &[], &[]);

    let brief = decision_brief_payload(
        MCP_DECISION_BRIEF_V1,
        "workspace",
        None,
        1,
        &agent_guidance,
        None,
    );

    assert_eq!(
        brief["layers"]["execution_strategy"]["review_focus_projection"],
        json!({
            "status": "no_priority_project",
            "source": Value::Null,
            "source_project_id": Value::Null,
            "review_focus": Value::Null
        })
    );
    assert_eq!(brief["decision"]["review_focus"], Value::Null);
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
