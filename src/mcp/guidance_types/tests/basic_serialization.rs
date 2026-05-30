use super::*;

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
