use super::*;

#[test]
fn risk_coupling_empty_recommendations() {
    let ws = json!({"global_strategy_mode": "defensive", "preferred_primary_tool": "opendog"});
    let result = execution_strategy_repo_risk_coupling(&[], &[], &ws).to_value();
    assert_eq!(result["status"], "no_repo_risk_signal");
    assert!(result["source"].is_null());
}

#[test]
fn risk_coupling_no_project_id() {
    let recs = vec![json!({"recommended_next_action": "start_monitor"})];
    let ws = json!({"global_strategy_mode": "defensive", "preferred_primary_tool": "opendog"});
    let result = execution_strategy_repo_risk_coupling(&recs, &[], &ws).to_value();
    assert_eq!(result["status"], "no_repo_risk_signal");
}

#[test]
fn risk_coupling_no_matching_overview() {
    let recs = vec![json!({"project_id": "proj_a", "recommended_next_action": "start_monitor"})];
    let overviews = vec![json!({"project_id": "proj_b"})];
    let ws = json!({"global_strategy_mode": "defensive", "preferred_primary_tool": "opendog"});
    let result = execution_strategy_repo_risk_coupling(&recs, &overviews, &ws).to_value();
    assert_eq!(result["status"], "no_repo_risk_signal");
}

#[test]
fn risk_coupling_null_risk_finding() {
    let recs = vec![json!({"project_id": "proj_a", "recommended_next_action": "start_monitor"})];
    let overviews = vec![json!({
        "project_id": "proj_a",
        "repo_status_risk": {"highest_priority_finding": null}
    })];
    let ws = json!({"global_strategy_mode": "defensive", "preferred_primary_tool": "opendog"});
    let result = execution_strategy_repo_risk_coupling(&recs, &overviews, &ws).to_value();
    assert_eq!(result["status"], "no_repo_risk_signal");
}

#[test]
fn risk_coupling_full_coupling() {
    let recs = vec![json!({
        "project_id": "proj_a",
        "recommended_next_action": "stabilize_repository_state"
    })];
    let overviews = vec![json!({
        "project_id": "proj_a",
        "repo_status_risk": {
            "highest_priority_finding": {
                "kind": "repository_operation_in_progress",
                "severity": "high",
                "priority": "immediate",
                "confidence": "high",
                "summary": "Repository is mid-operation: rebase.",
                "evidence": ["Git metadata indicates an in-progress operation: rebase."],
                "source": "git_metadata"
            }
        }
    })];
    let ws = json!({
        "global_strategy_mode": "defensive",
        "preferred_primary_tool": "opendog"
    });
    let result = execution_strategy_repo_risk_coupling(&recs, &overviews, &ws).to_value();
    assert_eq!(result["status"], "coupled");
    assert_eq!(result["source"], "primary_repo_risk_finding");
    assert_eq!(result["source_project_id"], "proj_a");
    assert_eq!(
        result["primary_repo_risk_finding"]["kind"],
        "repository_operation_in_progress"
    );
}

#[test]
fn risk_coupling_summary_includes_strategy_fields() {
    let recs = vec![json!({
        "project_id": "proj_a",
        "recommended_next_action": "stabilize_repository_state"
    })];
    let overviews = vec![json!({
        "project_id": "proj_a",
        "repo_status_risk": {
            "highest_priority_finding": {
                "kind": "conflicted_paths",
                "severity": "high",
                "priority": "immediate",
                "confidence": "high",
                "summary": "1 conflicted paths detected in the working tree.",
                "evidence": ["git status reported 1 conflicted paths."],
                "source": "git_status"
            }
        }
    })];
    let ws = json!({
        "global_strategy_mode": "stabilize_first",
        "preferred_primary_tool": "shell_verification"
    });
    let result = execution_strategy_repo_risk_coupling(&recs, &overviews, &ws).to_value();
    let summary = result["summary"].as_str().unwrap();
    assert!(
        summary.contains("stabilize_first"),
        "summary should contain strategy_mode: {summary}"
    );
    assert!(
        summary.contains("shell_verification"),
        "summary should contain preferred_primary_tool: {summary}"
    );
}

// ---------------------------------------------------------------------------
// execution_strategy_stabilization_summary
// ---------------------------------------------------------------------------
