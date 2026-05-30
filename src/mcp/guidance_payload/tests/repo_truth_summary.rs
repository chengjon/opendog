use super::*;

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
