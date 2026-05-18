use super::*;

#[test]
fn project_overview_fuses_verification_and_repo_readiness() {
    let verification = verification_status_layer(&[
        verification_run(1, "test", "passed", "cargo test", Some(0), fresh_ts()),
        verification_run(2, "build", "passed", "cargo check", Some(0), fresh_ts()),
    ]);
    let overview = project_overview(
        &project_state("demo", 20, 8, 4),
        &json!({
            "status": "available",
            "risk_level": "high",
            "is_dirty": true,
            "operation_states": ["rebase"],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": false
        }),
        &json!({
            "recommended_next_action": "stabilize_repository_state",
            "recommended_flow": [
                "Stabilize repository state first."
            ],
            "reason": "rebase",
            "confidence": "high"
        }),
        &verification,
        &json!({
            "mock_candidate_count": 1,
            "hardcoded_candidate_count": 0,
            "mixed_review_file_count": 0
        }),
        &json!({
            "status": "available",
            "maintenance_candidate": false,
            "vacuum_candidate": false,
            "cleanup_review_candidate": false,
            "approx_db_size_bytes": 0,
            "approx_reclaimable_bytes": 0,
            "reclaim_ratio": 0.0,
            "suggested_mode": "none"
        }),
    );

    assert_eq!(overview["verification_safe_for_cleanup"], json!(true));
    assert_eq!(
        overview["verification_gate_levels"]["cleanup"],
        json!("caution")
    );
    assert_eq!(
        overview["verification_gate_levels"]["refactor"],
        json!("caution")
    );
    assert_eq!(overview["safe_for_cleanup"], json!(false));
    assert_eq!(
        overview["recommended_flow"][0],
        json!("Stabilize repository state first.")
    );
    assert!(overview["cleanup_blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v.as_str().unwrap().contains("mid-operation")));
}

#[test]
fn build_constraints_boundaries_layer_includes_default_guardrails_and_blockers() {
    let verification = verification_status_layer(&[verification_run(
        1,
        "test",
        "failed",
        "cargo test",
        Some(101),
        "1".to_string(),
    )]);
    let value = build_constraints_boundaries_layer(
        Some(&json!({
            "operation_states": ["rebase"],
            "conflicted_count": 0,
            "lockfile_anomalies": [{"kind": "manifest_without_lockfile_change"}],
            "large_diff": true,
            "changed_file_count": 30
        })),
        Some(&verification),
        vec!["demo".to_string()],
        Vec::new(),
        vec!["blind".to_string()],
        vec!["git diff".to_string()],
        None,
    );

    assert!(value["guardrails"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("broad cleanup or refactor")));
    assert!(value["cleanup_blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("stabilized first")));
    assert!(value["refactor_blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("large diff")));
    assert!(value["human_review_required_for"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("Dependency manifest")));
}
