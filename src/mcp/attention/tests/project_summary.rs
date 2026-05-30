use super::*;

#[test]
fn project_attention_summary_minimal_clean_state() {
    let overview = minimal_overview();
    let summary = project_attention_summary(&overview);
    // Base for "inspect_hot_files" = 30, repo_risk low = +0, all fresh = +0
    // hardcoded=0, mock=0, safe_cleanup=true, safe_refactor=true => +0
    assert_eq!(summary.attention_score, 30);
    assert_eq!(summary.attention_band, "low");
    assert_eq!(summary.evidence_quality, "ready");
    assert_eq!(
        summary.priority_basis.recommended_next_action,
        "inspect_hot_files"
    );
    assert_eq!(summary.priority_basis.recommended_action_base, 30);
    assert_eq!(summary.priority_basis.repo_risk_level, "low");
    assert!(!summary.priority_basis.repo_in_operation);
    assert!(!summary.priority_basis.repo_is_dirty);
    assert!(summary.priority_basis.safe_for_cleanup);
    assert!(summary.priority_basis.safe_for_refactor);
    // Routine reason
    assert_eq!(
        summary.attention_reasons,
        vec!["Current evidence supports routine review sequencing.".to_string()]
    );
}

#[test]
fn project_attention_summary_high_repo_risk_and_dirty() {
    let overview = json!({
        "recommended_next_action": "stabilize_repository_state",
        "repo_status_risk": {
            "risk_level": "high",
            "operation_states": [],
            "is_dirty": true,
        },
        "verification_evidence": {
            "status": "recorded",
            "failing_runs": [],
        },
        "observation": {
            "freshness": {
                "snapshot": { "status": "fresh" },
                "activity": { "status": "fresh" },
                "verification": { "status": "fresh" },
            },
            "coverage_state": "active",
        },
        "mock_data_summary": {
            "hardcoded_candidate_count": 0,
            "mock_candidate_count": 0,
        },
        "safe_for_cleanup": true,
        "safe_for_refactor": true,
    });
    let summary = project_attention_summary(&overview);
    // Base 100 + high_risk 18 + dirty 6 = 124
    assert_eq!(summary.attention_score, 124);
    assert_eq!(summary.attention_band, "critical");
    // evidence_quality is "ready" since no blockers
    assert_eq!(summary.evidence_quality, "ready");
}

#[test]
fn project_attention_summary_repo_in_operation_adds_30() {
    let mut overview = minimal_overview();
    overview["repo_status_risk"]["operation_states"] = json!(["merge"]);
    let summary = project_attention_summary(&overview);
    // Base 30 + operation 30 = 60
    assert_eq!(summary.attention_score, 60);
    assert!(summary
        .attention_reasons
        .iter()
        .any(|r| r.contains("mid-operation")));
    assert_eq!(summary.evidence_quality, "blocked");
}

#[test]
fn project_attention_summary_failing_verification_adds_25() {
    let mut overview = minimal_overview();
    overview["verification_evidence"]["failing_runs"] =
        json!([{"command": "cargo test", "status": "failed"}]);
    let summary = project_attention_summary(&overview);
    // Base 30 + failing 25 = 55
    assert_eq!(summary.attention_score, 55);
    assert!(summary
        .attention_reasons
        .iter()
        .any(|r| r.contains("failing")));
    assert_eq!(summary.evidence_quality, "blocked");
}

#[test]
fn project_attention_summary_missing_snapshot_adds_14() {
    let mut overview = minimal_overview();
    overview["observation"]["freshness"]["snapshot"]["status"] = json!("missing");
    let summary = project_attention_summary(&overview);
    // Base 30 + missing_snapshot 14 = 44
    assert_eq!(summary.attention_score, 44);
    assert!(summary
        .attention_reasons
        .iter()
        .any(|r| r.contains("Snapshot baseline is missing")));
}

#[test]
fn project_attention_summary_stale_activity_adds_8() {
    let mut overview = minimal_overview();
    overview["observation"]["freshness"]["activity"]["status"] = json!("stale");
    let summary = project_attention_summary(&overview);
    // Base 30 + stale_activity 8 = 38
    assert_eq!(summary.attention_score, 38);
    assert!(summary
        .attention_reasons
        .iter()
        .any(|r| r.contains("Activity evidence is stale")));
}

#[test]
fn project_attention_summary_not_recorded_verification_adds_18_plus_freshness_18() {
    let mut overview = minimal_overview();
    overview["verification_evidence"]["status"] = json!("not_recorded");
    overview["observation"]["freshness"]["verification"]["status"] = json!("missing");
    let summary = project_attention_summary(&overview);
    // Base 30 + not_recorded 18 + missing_verification_freshness 18 = 66
    assert_eq!(summary.attention_score, 66);
}

#[test]
fn project_attention_summary_hardcoded_candidates_capped_at_3() {
    let mut overview = minimal_overview();
    overview["mock_data_summary"]["hardcoded_candidate_count"] = json!(10);
    let summary = project_attention_summary(&overview);
    // Base 30 + min(10,3)*5 = 30 + 15 = 45
    assert_eq!(summary.attention_score, 45);
    assert!(summary
        .attention_reasons
        .iter()
        .any(|r| r.contains("Hardcoded-data candidates")));
}

#[test]
fn project_attention_summary_mock_candidates_capped_at_3() {
    let mut overview = minimal_overview();
    overview["mock_data_summary"]["mock_candidate_count"] = json!(5);
    let summary = project_attention_summary(&overview);
    // Base 30 + min(5,3)*2 = 30 + 6 = 36
    assert_eq!(summary.attention_score, 36);
    assert!(summary
        .attention_reasons
        .iter()
        .any(|r| r.contains("Mock-style data candidates")));
}

#[test]
fn project_attention_summary_not_safe_for_cleanup_adds_6() {
    let mut overview = minimal_overview();
    overview["safe_for_cleanup"] = json!(false);
    let summary = project_attention_summary(&overview);
    // Base 30 + not_safe_cleanup 6 = 36
    assert_eq!(summary.attention_score, 36);
}

#[test]
fn project_attention_summary_not_safe_for_refactor_adds_6() {
    let mut overview = minimal_overview();
    overview["safe_for_refactor"] = json!(false);
    let summary = project_attention_summary(&overview);
    // Base 30 + not_safe_refactor 6 = 36
    assert_eq!(summary.attention_score, 36);
}

#[test]
fn project_attention_summary_evidence_quality_missing_snapshot() {
    let mut overview = minimal_overview();
    overview["observation"]["coverage_state"] = json!("missing_snapshot");
    let summary = project_attention_summary(&overview);
    assert_eq!(summary.evidence_quality, "missing");
}

#[test]
fn project_attention_summary_evidence_quality_stale() {
    let mut overview = minimal_overview();
    overview["observation"]["freshness"]["snapshot"]["status"] = json!("stale");
    let summary = project_attention_summary(&overview);
    assert_eq!(summary.evidence_quality, "stale");
}

#[test]
fn project_attention_summary_missing_key_fields_uses_defaults() {
    let overview = json!({});
    let summary = project_attention_summary(&overview);
    // Default action "inspect_workspace_state" => base 20
    // Default repo_risk_level "unknown" => +4
    // Default verification_status "not_recorded" => +18
    // Default snapshot "unknown" => freshness_attention_score("unknown", 14, 9) = 9
    // Default activity "unknown" => freshness_attention_score("unknown", 12, 8) = 8
    // Default verification freshness "unknown" => freshness_attention_score("unknown", 18, 12) = 12
    // safe_for_cleanup default false => +6
    // safe_for_refactor default false => +6
    // Total: 20 + 4 + 18 + 9 + 8 + 12 + 6 + 6 = 83
    assert_eq!(summary.attention_score, 83);
    assert_eq!(
        summary.priority_basis.recommended_next_action,
        "inspect_workspace_state"
    );
    assert_eq!(summary.priority_basis.repo_risk_level, "unknown");
    assert!(!summary.priority_basis.safe_for_cleanup);
    assert!(!summary.priority_basis.safe_for_refactor);
}

// --- workspace_portfolio_layer ---
