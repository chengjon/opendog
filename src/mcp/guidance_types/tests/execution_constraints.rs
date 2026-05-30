use super::*;

#[test]
fn execution_strategy_layer_serializes() {
    let mut data_risk_distribution = DataRiskFocusDistribution::default();
    data_risk_distribution.increment_focus("hardcoded");
    let mut repo_truth_distribution = RepoTruthGapDistribution::default();
    repo_truth_distribution.increment_gap("missing_test");

    let e = ExecutionStrategyLayer {
        status: ExecutionStrategyLayerStatus::Available,
        recommended_flow: vec!["refresh evidence".to_string(), "review risk".to_string()],
        project_recommendations: vec![],
        global_strategy_mode: RepoRiskStrategyMode::Other("evidence_first".to_string()),
        preferred_primary_tool: RepoRiskPreferredTool::Opendog,
        preferred_secondary_tool: RepoRiskPreferredTool::Shell,
        evidence_priority: vec![
            ExecutionEvidencePriority::Verification,
            ExecutionEvidencePriority::Other("activity".to_string()),
        ],
        risk_strategy_coupling: RepoRiskCoupling::no_signal(
            None,
            Some(RepoRiskStrategyMode::Other("evidence_first".to_string())),
            Some(RepoRiskPreferredTool::Opendog),
        ),
        external_truth_boundary: ExternalTruthBoundary::no_priority_project(),
        review_focus_projection: ReviewFocusProjection::no_priority_project(),
        when_to_use_opendog: vec![],
        when_to_use_shell: vec![],
        guardrails: vec![],
        projects_not_ready_for_cleanup: 0,
        projects_not_ready_for_refactor: 0,
        projects_with_hardcoded_data_candidates: 0,
        projects_missing_snapshot: 0,
        projects_with_stale_snapshot: 0,
        projects_missing_activity: 0,
        projects_with_stale_activity: 0,
        projects_missing_verification: 0,
        projects_with_stale_verification: 0,
        projects_with_storage_maintenance_candidates: 0,
        projects_with_vacuum_candidates: 0,
        review_opendog_retention_before_large_cleanup: false,
        recommend_manual_review_for_hardcoded_data: false,
        data_risk_focus_distribution: data_risk_distribution,
        projects_requiring_hardcoded_review: 1,
        projects_requiring_mock_review: 2,
        projects_requiring_mixed_file_review: 3,
        projects_requiring_monitor_start: 4,
        projects_requiring_snapshot_refresh: 5,
        projects_requiring_activity_generation: 6,
        projects_with_repo_truth_gaps: 7,
        repo_truth_gap_distribution: repo_truth_distribution,
        mandatory_shell_check_examples: vec!["cargo test".to_string()],
        projects_requiring_verification_run: 8,
        projects_requiring_failing_verification_repair: 9,
        projects_requiring_repo_stabilization: 10,
        repo_stabilization_priority_projects: vec!["proj_a".to_string()],
    };
    let v = serde_json::to_value(&e).unwrap();
    assert_eq!(v["status"], "available");
    assert_eq!(v["recommended_flow"][0], "refresh evidence");
    assert_eq!(v["recommended_flow"][1], "review risk");
    assert_eq!(v["global_strategy_mode"], "evidence_first");
    assert_eq!(v["preferred_primary_tool"], "opendog");
    assert_eq!(v["preferred_secondary_tool"], "shell");
    assert_eq!(v["evidence_priority"][0], "verification");
    assert_eq!(v["evidence_priority"][1], "activity");
    assert_eq!(
        v["external_truth_boundary"]["status"],
        "no_priority_project"
    );
    assert!(v["external_truth_boundary"]["mode"].is_null());
    assert!(v["external_truth_boundary"]["triggers"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(
        v["review_focus_projection"]["status"],
        "no_priority_project"
    );
    assert!(v["review_focus_projection"]["review_focus"].is_null());
    assert_eq!(v["data_risk_focus_distribution"]["hardcoded"], 1);
    assert_eq!(v["projects_requiring_hardcoded_review"], 1);
    assert_eq!(v["projects_requiring_monitor_start"], 4);
    assert_eq!(v["repo_truth_gap_distribution"]["missing_test"], 1);
    assert_eq!(v["mandatory_shell_check_examples"][0], "cargo test");
    assert_eq!(v["projects_requiring_verification_run"], 8);
    assert_eq!(v["projects_requiring_repo_stabilization"], 10);
    assert_eq!(v["repo_stabilization_priority_projects"][0], "proj_a");
}

#[test]
fn constraints_boundaries_layer_skips_none_optionals() {
    let c = ConstraintsBoundariesLayer {
        status: ConstraintsBoundariesLayerStatus::Available,
        direct_observations: vec![],
        inferences: vec![],
        blind_spots: vec![],
        guardrails: vec![],
        destructive_operations_requiring_confirmation: vec![],
        human_review_required_for: vec![],
        cleanup_blockers: vec![],
        refactor_blockers: vec![],
        requires_shell_verification: vec![],
        projects_not_ready_for_cleanup: None,
        projects_not_ready_for_refactor: None,
        projects_with_hardcoded_data_candidates: None,
        projects_missing_snapshot: None,
        projects_with_stale_snapshot: None,
        projects_missing_activity: None,
        projects_with_stale_activity: None,
        projects_missing_verification: None,
        projects_with_stale_verification: None,
        projects_with_storage_maintenance_candidates: None,
    };
    let v = serde_json::to_value(&c).unwrap();
    assert!(v.get("projects_not_ready_for_cleanup").is_none());
    assert!(v
        .get("projects_with_storage_maintenance_candidates")
        .is_none());
}

#[test]
fn constraints_boundaries_layer_includes_some_optionals() {
    let c = ConstraintsBoundariesLayer {
        status: ConstraintsBoundariesLayerStatus::Available,
        direct_observations: vec![],
        inferences: vec![],
        blind_spots: vec![],
        guardrails: vec![],
        destructive_operations_requiring_confirmation: vec![],
        human_review_required_for: vec![],
        cleanup_blockers: vec![],
        refactor_blockers: vec![],
        requires_shell_verification: vec![],
        projects_not_ready_for_cleanup: Some(2),
        projects_not_ready_for_refactor: Some(1),
        projects_with_hardcoded_data_candidates: Some(3),
        projects_missing_snapshot: Some(0),
        projects_with_stale_snapshot: Some(1),
        projects_missing_activity: Some(0),
        projects_with_stale_activity: Some(0),
        projects_missing_verification: Some(1),
        projects_with_stale_verification: Some(0),
        projects_with_storage_maintenance_candidates: Some(5),
    };
    let v = serde_json::to_value(&c).unwrap();
    assert_eq!(v["projects_not_ready_for_cleanup"], 2);
    assert_eq!(v["projects_with_storage_maintenance_candidates"], 5);
}
