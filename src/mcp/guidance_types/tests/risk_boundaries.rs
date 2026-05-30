use super::*;

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
