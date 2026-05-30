use super::*;

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
