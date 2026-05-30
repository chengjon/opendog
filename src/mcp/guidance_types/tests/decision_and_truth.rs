use super::*;

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
