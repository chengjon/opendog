use super::*;

#[test]
fn stabilization_summary_empty() {
    let summary = execution_strategy_stabilization_summary(&[]);
    assert_eq!(summary.projects_requiring_repo_stabilization, 0);
    assert!(summary.repo_stabilization_priority_projects.is_empty());
}

#[test]
fn stabilization_summary_matching_action_with_execution_sequence() {
    let recs = vec![json!({
        "project_id": "proj_a",
        "recommended_next_action": "stabilize_repository_state",
        "execution_sequence": {"mode": "resolve_rebase_then_resume"}
    })];
    let summary = execution_strategy_stabilization_summary(&recs);
    assert_eq!(summary.projects_requiring_repo_stabilization, 1);
    assert_eq!(summary.repo_stabilization_priority_projects, vec!["proj_a"]);
}

#[test]
fn stabilization_summary_matching_action_null_execution_sequence() {
    let recs = vec![json!({
        "project_id": "proj_a",
        "recommended_next_action": "stabilize_repository_state",
        "execution_sequence": null
    })];
    let summary = execution_strategy_stabilization_summary(&recs);
    assert_eq!(summary.projects_requiring_repo_stabilization, 0);
}

#[test]
fn stabilization_summary_different_action() {
    let recs = vec![json!({
        "project_id": "proj_a",
        "recommended_next_action": "start_monitor",
        "execution_sequence": {"mode": "start_monitor_then_resume"}
    })];
    let summary = execution_strategy_stabilization_summary(&recs);
    assert_eq!(summary.projects_requiring_repo_stabilization, 0);
}

#[test]
fn stabilization_summary_mixed() {
    let recs = vec![
        json!({
            "project_id": "proj_a",
            "recommended_next_action": "stabilize_repository_state",
            "execution_sequence": {"mode": "resolve_rebase_then_resume"}
        }),
        json!({
            "project_id": "proj_b",
            "recommended_next_action": "start_monitor",
            "execution_sequence": {"mode": "start_monitor_then_resume"}
        }),
        json!({
            "project_id": "proj_c",
            "recommended_next_action": "stabilize_repository_state",
            "execution_sequence": {"mode": "resolve_merge_then_resume"}
        }),
    ];
    let summary = execution_strategy_stabilization_summary(&recs);
    assert_eq!(summary.projects_requiring_repo_stabilization, 2);
    assert_eq!(
        summary.repo_stabilization_priority_projects,
        vec!["proj_a", "proj_c"]
    );
}

// ---------------------------------------------------------------------------
// execution_strategy_verification_summary
// ---------------------------------------------------------------------------
