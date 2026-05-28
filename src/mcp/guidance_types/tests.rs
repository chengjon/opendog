use super::*;
use serde_json::json;

#[test]
fn recommendation_serializes_all_fields() {
    let rec = Recommendation {
        project_id: "proj".into(),
        recommended_next_action: "take_snapshot".into(),
        recommended_flow: vec!["step1".into()],
        reason: "reason".into(),
        confidence: "high".into(),
        strategy_mode: "evidence_first".into(),
        strategy_profile: json!({"mode": "evidence_first"}),
        verification_gate_levels: json!({}),
        cleanup_blockers: Some(json!(["blocker1"])),
        refactor_blockers: Some(json!(["blocker2"])),
        repo_truth_gaps: json!({}),
        mandatory_shell_checks: json!({}),
        suggested_commands: vec!["cargo test".into()],
    };
    let v = serde_json::to_value(&rec).unwrap();
    assert_eq!(v["project_id"], "proj");
    assert!(v.get("cleanup_blockers").is_some());
    assert!(v.get("refactor_blockers").is_some());
}

#[test]
fn recommendation_skips_none_optionals() {
    let rec = Recommendation {
        project_id: "p".into(),
        recommended_next_action: "a".into(),
        recommended_flow: vec![],
        reason: "r".into(),
        confidence: "c".into(),
        strategy_mode: "s".into(),
        strategy_profile: json!(null),
        verification_gate_levels: json!(null),
        cleanup_blockers: None,
        refactor_blockers: None,
        repo_truth_gaps: json!(null),
        mandatory_shell_checks: json!(null),
        suggested_commands: vec![],
    };
    let v = serde_json::to_value(&rec).unwrap();
    assert!(v.get("cleanup_blockers").is_none());
    assert!(v.get("refactor_blockers").is_none());
}

#[test]
fn project_overview_serializes() {
    let po = ProjectOverview {
        project_id: "x".into(),
        status: "active".into(),
        snapshot_available: true,
        activity_available: false,
        unused_files: 5,
        observation: json!({}),
        repo_status_risk: json!({}),
        verification_evidence: json!({}),
        mock_data_summary: json!({}),
        storage_maintenance: json!({}),
        project_toolchain: json!({}),
        verification_safe_for_cleanup: json!(true),
        verification_safe_for_refactor: json!(true),
        verification_gate_levels: json!({}),
        safe_for_cleanup: json!(true),
        safe_for_cleanup_reason: json!("ok"),
        cleanup_blockers: json!([]),
        safe_for_refactor: json!(true),
        safe_for_refactor_reason: json!("ok"),
        refactor_blockers: json!([]),
        recommended_next_action: json!("none"),
        recommended_flow: json!([]),
        recommended_reason: json!(""),
        strategy_confidence: json!("high"),
    };
    let v = serde_json::to_value(&po).unwrap();
    assert_eq!(v["project_id"], "x");
    assert!(v["snapshot_available"].as_bool().unwrap());
}

#[test]
fn attention_summary_serializes() {
    let a = AttentionSummary {
        attention_score: 42,
        attention_band: "high".into(),
        attention_reasons: vec!["reason1".into()],
        evidence_quality: "good".into(),
        priority_basis: AttentionPriorityBasis {
            recommended_next_action: "stabilize".into(),
            recommended_action_base: 10,
            repo_risk_level: "medium".into(),
            repo_in_operation: false,
            repo_is_dirty: true,
            verification_status: "passed".into(),
            has_failing_verification: false,
            coverage_state: "partial".into(),
            snapshot_freshness: "fresh".into(),
            activity_freshness: "fresh".into(),
            verification_freshness: "stale".into(),
            hardcoded_candidate_count: 3,
            mock_candidate_count: 1,
            safe_for_cleanup: false,
            safe_for_refactor: false,
        },
    };
    let v = serde_json::to_value(&a).unwrap();
    assert_eq!(v["attention_score"], 42);
    assert!(v["priority_basis"]["repo_is_dirty"].as_bool().unwrap());
}

#[test]
fn workspace_portfolio_layer_serializes() {
    let w = WorkspacePortfolioLayer {
        status: WorkspacePortfolioLayerStatus::Available,
        project_count: 2,
        monitoring_count: 1,
        monitored_projects: vec![json!("p1")],
        priority_candidates: vec![],
        project_overviews: vec![],
        priority_model: "attention".into(),
        dirty_projects: 0,
        high_risk_projects: 1,
        projects_with_failing_verification: 0,
        projects_safe_for_cleanup: 1,
        projects_safe_for_refactor: 1,
        projects_with_hardcoded_candidates: 0,
        projects_with_hardcoded_data_candidates: 0,
        total_mock_candidates: 0,
        total_hardcoded_candidates: 0,
        projects_in_operation: vec![],
        attention_queue: vec![],
        attention_batches: json!({}),
    };
    let v = serde_json::to_value(&w).unwrap();
    assert_eq!(v["project_count"], 2);
    assert_eq!(v["priority_model"], "attention");
}

#[test]
fn decision_brief_serializes_with_options() {
    let d = DecisionBrief {
        summary: "test".into(),
        recommended_next_action: "act".into(),
        reason: json!("reason"),
        repo_truth_gaps: json!([]),
        mandatory_shell_checks: json!([]),
        external_truth_boundary: json!(null),
        review_focus: json!(null),
        execution_sequence: json!({}),
        data_risk_focus: json!("none"),
        target_project_id: Some("proj".into()),
        strategy_mode: json!("evidence"),
        preferred_primary_tool: json!("opendog"),
        preferred_secondary_tool: json!("shell"),
        recommended_flow: json!([]),
        safe_for_cleanup: Some(true),
        safe_for_refactor: Some(false),
        verification_status: "passed".into(),
        requires_verification: false,
        action_profile: json!({}),
        risk_profile: json!({}),
        signals: DecisionSignals {
            repo_risk_level: "low".into(),
            repo_is_dirty: false,
            hardcoded_candidate_count: 0,
            mock_candidate_count: 0,
            mixed_review_file_count: 0,
            storage_maintenance_candidate: false,
            storage_vacuum_candidate: false,
            storage_reclaimable_bytes: 0,
            storage_db_size_bytes: 1024,
            attention_score: 10,
            attention_band: "low".into(),
            attention_reasons: vec![],
            monitoring_count: 1,
        },
    };
    let v = serde_json::to_value(&d).unwrap();
    assert_eq!(v["target_project_id"], "proj");
    assert!(v["safe_for_cleanup"].as_bool().unwrap());
    assert!(!v["safe_for_refactor"].as_bool().unwrap());
}

#[test]
fn decision_brief_skips_none_options() {
    let d = DecisionBrief {
        summary: "s".into(),
        recommended_next_action: "a".into(),
        reason: json!(null),
        repo_truth_gaps: json!(null),
        mandatory_shell_checks: json!(null),
        external_truth_boundary: json!(null),
        review_focus: json!(null),
        execution_sequence: json!(null),
        data_risk_focus: json!(null),
        target_project_id: None,
        strategy_mode: json!(null),
        preferred_primary_tool: json!(null),
        preferred_secondary_tool: json!(null),
        recommended_flow: json!(null),
        safe_for_cleanup: None,
        safe_for_refactor: None,
        verification_status: "unknown".into(),
        requires_verification: false,
        action_profile: json!(null),
        risk_profile: json!(null),
        signals: DecisionSignals {
            repo_risk_level: "low".into(),
            repo_is_dirty: false,
            hardcoded_candidate_count: 0,
            mock_candidate_count: 0,
            mixed_review_file_count: 0,
            storage_maintenance_candidate: false,
            storage_vacuum_candidate: false,
            storage_reclaimable_bytes: 0,
            storage_db_size_bytes: 0,
            attention_score: 0,
            attention_band: "low".into(),
            attention_reasons: vec![],
            monitoring_count: 0,
        },
    };
    let v = serde_json::to_value(&d).unwrap();
    assert!(v["target_project_id"].is_null());
    assert!(v["safe_for_cleanup"].is_null());
    assert!(v["safe_for_refactor"].is_null());
}

#[test]
fn decision_signals_serializes() {
    let s = DecisionSignals {
        repo_risk_level: "high".into(),
        repo_is_dirty: true,
        hardcoded_candidate_count: 5,
        mock_candidate_count: 2,
        mixed_review_file_count: 1,
        storage_maintenance_candidate: true,
        storage_vacuum_candidate: false,
        storage_reclaimable_bytes: 4096,
        storage_db_size_bytes: 8192,
        attention_score: 80,
        attention_band: "critical".into(),
        attention_reasons: vec![json!("r1")],
        monitoring_count: 3,
    };
    let v = serde_json::to_value(&s).unwrap();
    assert_eq!(v["hardcoded_candidate_count"], 5);
    assert!(v["repo_is_dirty"].as_bool().unwrap());
}

#[test]
fn repo_truth_summary_serializes() {
    let mut distribution = RepoTruthGapDistribution::default();
    distribution.increment_gap("missing_test");
    distribution.increment_gap("missing_test");

    let s = RepoTruthSummary {
        projects_with_repo_truth_gaps: 2,
        repo_truth_gap_distribution: distribution,
        mandatory_shell_check_examples: vec!["cargo test".into()],
    };
    let v = serde_json::to_value(&s).unwrap();
    assert_eq!(v["projects_with_repo_truth_gaps"], 2);
    assert_eq!(v["repo_truth_gap_distribution"]["missing_test"], 2);
}

#[test]
fn repo_truth_gap_distribution_counts_dynamic_keys() {
    let mut distribution = RepoTruthGapDistribution::default();

    distribution.increment_gap("missing_test");
    distribution.increment_gap("missing_test");
    distribution.increment_gap("missing_lint");

    assert_eq!(distribution.count("missing_test"), 2);
    assert_eq!(distribution.count("missing_lint"), 1);
    assert_eq!(distribution.count("missing_build"), 0);

    let v = serde_json::to_value(&distribution).unwrap();
    assert_eq!(v["missing_test"], 2);
    assert_eq!(v["missing_lint"], 1);
}

#[test]
fn repo_risk_coupling_no_signal_serializes_null_boundaries() {
    let coupling = RepoRiskCoupling::no_signal(
        Some(RecommendedNextAction::StartMonitor),
        Some(RepoRiskStrategyMode::CollectEvidenceFirst),
        Some(RepoRiskPreferredTool::Opendog),
    );

    let v = serde_json::to_value(&coupling).unwrap();
    assert_eq!(v["status"], "no_repo_risk_signal");
    assert!(v["source"].is_null());
    assert!(v["source_project_id"].is_null());
    assert_eq!(v["recommended_next_action"], "start_monitor");
    assert_eq!(v["strategy_mode"], "collect_evidence_first");
    assert_eq!(v["preferred_primary_tool"], "opendog");
    assert!(v["primary_repo_risk_finding"].is_null());
    assert!(v["summary"].is_null());
}

#[test]
fn repo_risk_coupling_coupled_serializes_context() {
    let finding = RepoRiskFinding::from_value(&json!({
        "kind": "repository_operation_in_progress",
        "severity": "high",
        "priority": "immediate",
        "confidence": "high",
        "summary": "merge in progress",
        "evidence": ["Git metadata indicates an in-progress operation: merge."],
        "source": "git_metadata"
    }))
    .unwrap();
    let coupling = RepoRiskCoupling::coupled(
        "proj_a",
        Some(RecommendedNextAction::StabilizeRepositoryState),
        Some(RepoRiskStrategyMode::StabilizeBeforeModify),
        Some(RepoRiskPreferredTool::ShellVerification),
        finding,
        "Top repository risk keeps the workspace in stabilize_first mode.".to_string(),
    );

    assert_eq!(
        coupling.source,
        Some(RepoRiskCouplingSource::PrimaryRepoRiskFinding)
    );

    let v = serde_json::to_value(&coupling).unwrap();
    assert_eq!(v["status"], "coupled");
    assert_eq!(v["source"], "primary_repo_risk_finding");
    assert_eq!(v["source_project_id"], "proj_a");
    assert_eq!(
        v["primary_repo_risk_finding"]["summary"],
        "merge in progress"
    );
    assert_eq!(
        v["primary_repo_risk_finding"]["kind"],
        "repository_operation_in_progress"
    );
    assert_eq!(
        v["summary"],
        "Top repository risk keeps the workspace in stabilize_first mode."
    );
}

#[test]
fn repo_risk_finding_serializes_lockfile_details() {
    let finding = RepoRiskFinding::from_value(&json!({
        "kind": "dependency_lockfile_anomaly",
        "severity": "medium",
        "priority": "high",
        "confidence": "high",
        "summary": "Manifest changed without corresponding lockfile change in .",
        "evidence": ["git status flagged dependency file drift in ."],
        "source": "git_status",
        "details": {
            "kind": "manifest_without_lockfile_change",
            "directory": "."
        }
    }))
    .unwrap();

    let v = serde_json::to_value(&finding).unwrap();
    assert_eq!(v["kind"], "dependency_lockfile_anomaly");
    assert_eq!(v["details"]["kind"], "manifest_without_lockfile_change");
    assert_eq!(v["details"]["directory"], ".");
}

#[test]
fn review_focus_projection_available_serializes_source_project() {
    let projection =
        ReviewFocusProjection::available(Some("proj_a".to_string()), json!("inspect hot files"));

    let v = serde_json::to_value(&projection).unwrap();
    assert_eq!(v["status"], "available");
    assert_eq!(v["source"], "top_priority_project");
    assert_eq!(v["source_project_id"], "proj_a");
    assert_eq!(v["review_focus"], "inspect hot files");
}

#[test]
fn external_truth_boundary_available_serializes_source_project() {
    let boundary = ExternalTruthBoundary::available(
        Some("proj_a".to_string()),
        ExternalTruthBoundaryMode::MustSwitchToExternalTruth,
        true,
        false,
        vec!["working_tree_conflicted".to_string()],
        vec!["git status".to_string()],
        "Top project needs direct repository truth before broader changes.",
    );

    let v = serde_json::to_value(&boundary).unwrap();
    assert_eq!(v["status"], "available");
    assert_eq!(v["source"], "top_priority_project");
    assert_eq!(v["source_project_id"], "proj_a");
    assert_eq!(v["mode"], "must_switch_to_external_truth");
    assert_eq!(v["repo_state_required"], true);
    assert_eq!(v["minimum_external_checks"][0], "git status");
}

#[test]
fn stabilization_summary_serializes() {
    let s = StabilizationSummary {
        projects_requiring_repo_stabilization: 1,
        repo_stabilization_priority_projects: vec!["proj1".into()],
    };
    let v = serde_json::to_value(&s).unwrap();
    assert_eq!(v["projects_requiring_repo_stabilization"], 1);
}

#[test]
fn verification_summary_serializes() {
    let s = VerificationSummary {
        projects_requiring_verification_run: 3,
        projects_requiring_failing_verification_repair: 1,
    };
    let v = serde_json::to_value(&s).unwrap();
    assert_eq!(v["projects_requiring_verification_run"], 3);
}

#[test]
fn observation_summary_serializes() {
    let s = ObservationSummary {
        projects_requiring_monitor_start: 1,
        projects_requiring_snapshot_refresh: 2,
        projects_requiring_activity_generation: 0,
    };
    let v = serde_json::to_value(&s).unwrap();
    assert_eq!(v["projects_requiring_snapshot_refresh"], 2);
}

#[test]
fn data_risk_focus_summary_serializes() {
    let mut distribution = DataRiskFocusDistribution::default();
    distribution.increment_focus("hardcoded");
    distribution.increment_focus("none");
    distribution.increment_focus("none");

    let s = DataRiskFocusSummary {
        data_risk_focus_distribution: distribution,
        projects_requiring_hardcoded_review: 1,
        projects_requiring_mock_review: 0,
        projects_requiring_mixed_file_review: 0,
    };
    let v = serde_json::to_value(&s).unwrap();
    assert_eq!(v["projects_requiring_hardcoded_review"], 1);
    assert_eq!(v["data_risk_focus_distribution"]["none"], 2);
}

#[test]
fn data_risk_focus_distribution_counts_known_and_unknown_focuses() {
    let mut distribution = DataRiskFocusDistribution::default();

    distribution.increment_focus("hardcoded");
    distribution.increment_focus("mock");
    distribution.increment_focus("mixed");
    distribution.increment_focus("unexpected");

    let v = serde_json::to_value(&distribution).unwrap();
    assert_eq!(v["hardcoded"], 1);
    assert_eq!(v["mock"], 1);
    assert_eq!(v["mixed"], 1);
    assert_eq!(v["none"], 1);
}

#[test]
fn workspace_observation_layer_serializes() {
    let w = WorkspaceObservationLayer {
        status: WorkspaceObservationLayerStatus::Available,
        project_count: 3,
        monitoring_count: 2,
        analysis_state: WorkspaceObservationAnalysisState::Ready,
        projects_missing_snapshot: 0,
        projects_with_stale_snapshot: 1,
        projects_missing_activity: 0,
        projects_with_stale_activity: 0,
        projects_missing_verification: 1,
        projects_with_stale_verification: 0,
        projects_with_storage_maintenance_candidates: 0,
        projects_with_vacuum_candidates: 0,
        total_storage_reclaimable_bytes: json!(0),
        data_risk_focus_distribution: json!({}),
        projects_requiring_hardcoded_review: json!(0),
        projects_requiring_mock_review: json!(0),
        projects_requiring_mixed_file_review: json!(0),
        notes: vec!["note1".into()],
    };
    let v = serde_json::to_value(&w).unwrap();
    assert_eq!(v["project_count"], 3);
    assert_eq!(v["analysis_state"], "ready");
}

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
