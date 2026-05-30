use super::*;

#[test]
fn verification_summary_empty() {
    let summary = execution_strategy_verification_summary(&[]);
    assert_eq!(summary.projects_requiring_verification_run, 0);
    assert_eq!(summary.projects_requiring_failing_verification_repair, 0);
}

#[test]
fn verification_summary_matching_verification_run() {
    let recs = vec![json!({
        "recommended_next_action": "run_verification_before_high_risk_changes",
        "execution_sequence": {"mode": "run_project_verification_then_resume"}
    })];
    let summary = execution_strategy_verification_summary(&recs);
    assert_eq!(summary.projects_requiring_verification_run, 1);
    assert_eq!(summary.projects_requiring_failing_verification_repair, 0);
}

#[test]
fn verification_summary_matching_failing_verification() {
    let recs = vec![json!({
        "recommended_next_action": "review_failing_verification",
        "execution_sequence": {"mode": "resolve_failing_verification_then_resume"}
    })];
    let summary = execution_strategy_verification_summary(&recs);
    assert_eq!(summary.projects_requiring_verification_run, 0);
    assert_eq!(summary.projects_requiring_failing_verification_repair, 1);
}

#[test]
fn verification_summary_wrong_mode() {
    let recs = vec![json!({
        "recommended_next_action": "run_verification_before_high_risk_changes",
        "execution_sequence": {"mode": "wrong_mode"}
    })];
    let summary = execution_strategy_verification_summary(&recs);
    assert_eq!(summary.projects_requiring_verification_run, 0);
    assert_eq!(summary.projects_requiring_failing_verification_repair, 0);
}

#[test]
fn verification_summary_mixed() {
    let recs = vec![
        json!({
            "recommended_next_action": "run_verification_before_high_risk_changes",
            "execution_sequence": {"mode": "run_project_verification_then_resume"}
        }),
        json!({
            "recommended_next_action": "review_failing_verification",
            "execution_sequence": {"mode": "resolve_failing_verification_then_resume"}
        }),
        json!({
            "recommended_next_action": "run_verification_before_high_risk_changes",
            "execution_sequence": {"mode": "run_project_verification_then_resume"}
        }),
        json!({
            "recommended_next_action": "start_monitor",
            "execution_sequence": {"mode": "start_monitor_then_resume"}
        }),
    ];
    let summary = execution_strategy_verification_summary(&recs);
    assert_eq!(summary.projects_requiring_verification_run, 2);
    assert_eq!(summary.projects_requiring_failing_verification_repair, 1);
}

// ---------------------------------------------------------------------------
// execution_strategy_observation_summary
// ---------------------------------------------------------------------------

#[test]
fn observation_summary_empty() {
    let summary = execution_strategy_observation_summary(&[]);
    assert_eq!(summary.projects_requiring_monitor_start, 0);
    assert_eq!(summary.projects_requiring_snapshot_refresh, 0);
    assert_eq!(summary.projects_requiring_activity_generation, 0);
}

#[test]
fn observation_summary_start_monitor() {
    let recs = vec![json!({
        "recommended_next_action": "start_monitor",
        "execution_sequence": {"mode": "start_monitor_then_resume"}
    })];
    let summary = execution_strategy_observation_summary(&recs);
    assert_eq!(summary.projects_requiring_monitor_start, 1);
    assert_eq!(summary.projects_requiring_snapshot_refresh, 0);
    assert_eq!(summary.projects_requiring_activity_generation, 0);
}

#[test]
fn observation_summary_take_snapshot() {
    let recs = vec![json!({
        "recommended_next_action": "take_snapshot",
        "execution_sequence": {"mode": "refresh_snapshot_then_resume"}
    })];
    let summary = execution_strategy_observation_summary(&recs);
    assert_eq!(summary.projects_requiring_monitor_start, 0);
    assert_eq!(summary.projects_requiring_snapshot_refresh, 1);
    assert_eq!(summary.projects_requiring_activity_generation, 0);
}

#[test]
fn observation_summary_generate_activity() {
    let recs = vec![json!({
        "recommended_next_action": "generate_activity_then_stats",
        "execution_sequence": {"mode": "generate_activity_then_resume"}
    })];
    let summary = execution_strategy_observation_summary(&recs);
    assert_eq!(summary.projects_requiring_monitor_start, 0);
    assert_eq!(summary.projects_requiring_snapshot_refresh, 0);
    assert_eq!(summary.projects_requiring_activity_generation, 1);
}

#[test]
fn observation_summary_wrong_mode() {
    let recs = vec![json!({
        "recommended_next_action": "start_monitor",
        "execution_sequence": {"mode": "wrong_mode"}
    })];
    let summary = execution_strategy_observation_summary(&recs);
    assert_eq!(summary.projects_requiring_monitor_start, 0);
    assert_eq!(summary.projects_requiring_snapshot_refresh, 0);
    assert_eq!(summary.projects_requiring_activity_generation, 0);
}

#[test]
fn observation_summary_mixed() {
    let recs = vec![
        json!({
            "recommended_next_action": "start_monitor",
            "execution_sequence": {"mode": "start_monitor_then_resume"}
        }),
        json!({
            "recommended_next_action": "take_snapshot",
            "execution_sequence": {"mode": "refresh_snapshot_then_resume"}
        }),
        json!({
            "recommended_next_action": "generate_activity_then_stats",
            "execution_sequence": {"mode": "generate_activity_then_resume"}
        }),
        json!({
            "recommended_next_action": "start_monitor",
            "execution_sequence": {"mode": "start_monitor_then_resume"}
        }),
    ];
    let summary = execution_strategy_observation_summary(&recs);
    assert_eq!(summary.projects_requiring_monitor_start, 2);
    assert_eq!(summary.projects_requiring_snapshot_refresh, 1);
    assert_eq!(summary.projects_requiring_activity_generation, 1);
}

// ---------------------------------------------------------------------------
// execution_strategy_data_risk_focus_summary
// ---------------------------------------------------------------------------
