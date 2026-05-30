use super::*;

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
