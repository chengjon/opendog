use super::*;
use crate::mcp::constraints::repo_truth_gap_projection;

#[test]
fn repo_truth_gap_projection_maps_repo_truth_blind_spots() {
    let (gaps, checks) = repo_truth_gap_projection(&json!({
        "status": "error",
        "operation_states": ["rebase"],
        "conflicted_count": 2,
        "lockfile_anomalies": ["package-lock.json without package.json"],
        "large_diff": true
    }));

    assert_eq!(
        gaps,
        vec![
            "git_metadata_unavailable".to_string(),
            "repository_mid_operation".to_string(),
            "working_tree_conflicted".to_string(),
            "dependency_state_requires_repo_review".to_string(),
        ]
    );
    assert_eq!(
        checks,
        vec!["git status".to_string(), "git diff".to_string()]
    );
}

#[test]
fn repo_truth_gap_projection_keeps_non_git_projects_out_of_mandatory_git_checks() {
    let (gaps, checks) = repo_truth_gap_projection(&json!({
        "status": "not_git_repository",
        "operation_states": [],
        "conflicted_count": 0,
        "lockfile_anomalies": [],
        "large_diff": true
    }));

    assert_eq!(gaps, vec!["not_git_repository".to_string()]);
    assert!(checks.is_empty());
}
