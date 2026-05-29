use super::*;
use serde_json::json;

// ---------------------------------------------------------------------------
// execution_strategy_repo_truth_summary
// ---------------------------------------------------------------------------

#[test]
fn repo_truth_summary_empty_input() {
    let summary = execution_strategy_repo_truth_summary(&[]);
    assert_eq!(summary.projects_with_repo_truth_gaps, 0);
    assert_eq!(
        serde_json::to_value(&summary.repo_truth_gap_distribution).unwrap(),
        json!({})
    );
    assert!(summary.mandatory_shell_check_examples.is_empty());
}

#[test]
fn repo_truth_summary_empty_gaps() {
    let recs = vec![json!({"repo_truth_gaps": []})];
    let summary = execution_strategy_repo_truth_summary(&recs);
    assert_eq!(summary.projects_with_repo_truth_gaps, 0);
    assert_eq!(
        serde_json::to_value(&summary.repo_truth_gap_distribution).unwrap(),
        json!({})
    );
    assert!(summary.mandatory_shell_check_examples.is_empty());
}

#[test]
fn repo_truth_summary_single_gap_with_shell_checks() {
    let recs = vec![json!({
        "repo_truth_gaps": ["missing_test"],
        "mandatory_shell_checks": ["cargo test"]
    })];
    let summary = execution_strategy_repo_truth_summary(&recs);
    assert_eq!(summary.projects_with_repo_truth_gaps, 1);
    assert_eq!(summary.repo_truth_gap_distribution.count("missing_test"), 1);
    assert_eq!(summary.mandatory_shell_check_examples, vec!["cargo test"]);
}

#[test]
fn repo_truth_summary_two_same_gap_key() {
    let recs = vec![
        json!({"repo_truth_gaps": ["missing_test"]}),
        json!({"repo_truth_gaps": ["missing_test"]}),
    ];
    let summary = execution_strategy_repo_truth_summary(&recs);
    assert_eq!(summary.projects_with_repo_truth_gaps, 2);
    assert_eq!(summary.repo_truth_gap_distribution.count("missing_test"), 2);
}

#[test]
fn repo_truth_summary_non_string_gaps_skipped() {
    let recs = vec![json!({
        "repo_truth_gaps": [42, true, null, {"a": 1}]
    })];
    let summary = execution_strategy_repo_truth_summary(&recs);
    // The array is non-empty so the project counts as having gaps
    assert_eq!(summary.projects_with_repo_truth_gaps, 1);
    // But none of the entries are strings, so distribution is empty
    assert_eq!(
        serde_json::to_value(&summary.repo_truth_gap_distribution).unwrap(),
        json!({})
    );
}

#[test]
fn repo_truth_summary_duplicate_shell_checks_deduplicated() {
    let recs = vec![
        json!({"repo_truth_gaps": [], "mandatory_shell_checks": ["cargo test", "cargo clippy"]}),
        json!({"repo_truth_gaps": [], "mandatory_shell_checks": ["cargo test"]}),
    ];
    let summary = execution_strategy_repo_truth_summary(&recs);
    assert_eq!(
        summary.mandatory_shell_check_examples,
        vec!["cargo test", "cargo clippy"]
    );
}

// ---------------------------------------------------------------------------
// execution_strategy_repo_risk_coupling
// ---------------------------------------------------------------------------

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

#[test]
fn data_risk_summary_empty() {
    let summary = execution_strategy_data_risk_focus_summary(&[]);
    assert_eq!(
        summary.data_risk_focus_distribution.to_value(),
        json!({"hardcoded": 0, "mixed": 0, "mock": 0, "none": 0})
    );
    assert_eq!(summary.projects_requiring_hardcoded_review, 0);
    assert_eq!(summary.projects_requiring_mock_review, 0);
    assert_eq!(summary.projects_requiring_mixed_file_review, 0);
}

#[test]
fn data_risk_summary_hardcoded() {
    let overviews = vec![json!({
        "mock_data_summary": {"data_risk_focus": {"primary_focus": "hardcoded"}}
    })];
    let summary = execution_strategy_data_risk_focus_summary(&overviews);
    assert_eq!(summary.projects_requiring_hardcoded_review, 1);
    assert_eq!(summary.projects_requiring_mock_review, 0);
    assert_eq!(summary.projects_requiring_mixed_file_review, 0);
    assert_eq!(summary.data_risk_focus_distribution.hardcoded, 1);
}

#[test]
fn data_risk_summary_mock() {
    let overviews = vec![json!({
        "mock_data_summary": {"data_risk_focus": {"primary_focus": "mock"}}
    })];
    let summary = execution_strategy_data_risk_focus_summary(&overviews);
    assert_eq!(summary.projects_requiring_hardcoded_review, 0);
    assert_eq!(summary.projects_requiring_mock_review, 1);
    assert_eq!(summary.projects_requiring_mixed_file_review, 0);
    assert_eq!(summary.data_risk_focus_distribution.mock, 1);
}

#[test]
fn data_risk_summary_mixed() {
    let overviews = vec![json!({
        "mock_data_summary": {"data_risk_focus": {"primary_focus": "mixed"}}
    })];
    let summary = execution_strategy_data_risk_focus_summary(&overviews);
    assert_eq!(summary.projects_requiring_hardcoded_review, 0);
    assert_eq!(summary.projects_requiring_mock_review, 0);
    assert_eq!(summary.projects_requiring_mixed_file_review, 1);
    assert_eq!(summary.data_risk_focus_distribution.mixed, 1);
}

#[test]
fn data_risk_summary_none_focus() {
    let overviews = vec![json!({
        "mock_data_summary": {"data_risk_focus": {"primary_focus": "none"}}
    })];
    let summary = execution_strategy_data_risk_focus_summary(&overviews);
    assert_eq!(summary.projects_requiring_hardcoded_review, 0);
    assert_eq!(summary.projects_requiring_mock_review, 0);
    assert_eq!(summary.projects_requiring_mixed_file_review, 0);
    assert_eq!(summary.data_risk_focus_distribution.none, 1);
}

#[test]
fn data_risk_summary_missing_focus() {
    // Missing primary_focus field defaults to "none"
    let overviews = vec![json!({
        "mock_data_summary": {"data_risk_focus": {}}
    })];
    let summary = execution_strategy_data_risk_focus_summary(&overviews);
    assert_eq!(summary.data_risk_focus_distribution.none, 1);
}

#[test]
fn data_risk_summary_mixed_overviews() {
    let overviews = vec![
        json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "hardcoded"}}}),
        json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "mock"}}}),
        json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "hardcoded"}}}),
        json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "mixed"}}}),
        json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "none"}}}),
        json!({"mock_data_summary": {}}),
    ];
    let summary = execution_strategy_data_risk_focus_summary(&overviews);
    assert_eq!(summary.projects_requiring_hardcoded_review, 2);
    assert_eq!(summary.projects_requiring_mock_review, 1);
    assert_eq!(summary.projects_requiring_mixed_file_review, 1);
    assert_eq!(summary.data_risk_focus_distribution.hardcoded, 2);
    assert_eq!(summary.data_risk_focus_distribution.mock, 1);
    assert_eq!(summary.data_risk_focus_distribution.mixed, 1);
    assert_eq!(summary.data_risk_focus_distribution.none, 2);
}
