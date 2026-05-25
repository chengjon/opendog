use serde_json::Value;

fn push_once(items: &mut Vec<String>, value: &str) {
    if !items.iter().any(|item| item == value) {
        items.push(value.to_string());
    }
}

pub(crate) fn repo_truth_gap_projection(repo_risk: &Value) -> (Vec<String>, Vec<String>) {
    let mut gaps = Vec::new();
    let mut mandatory_shell_checks = Vec::new();
    let status = repo_risk["status"].as_str().unwrap_or("unknown");

    match status {
        "not_git_repository" => push_once(&mut gaps, "not_git_repository"),
        "error" => {
            push_once(&mut gaps, "git_metadata_unavailable");
            push_once(&mut mandatory_shell_checks, "git status");
        }
        _ => {}
    }

    if repo_risk["operation_states"]
        .as_array()
        .map(|states| !states.is_empty())
        .unwrap_or(false)
    {
        push_once(&mut gaps, "repository_mid_operation");
        push_once(&mut mandatory_shell_checks, "git status");
        push_once(&mut mandatory_shell_checks, "git diff");
    }

    if repo_risk["conflicted_count"].as_u64().unwrap_or(0) > 0 {
        push_once(&mut gaps, "working_tree_conflicted");
        push_once(&mut mandatory_shell_checks, "git status");
        push_once(&mut mandatory_shell_checks, "git diff");
    }

    if repo_risk["lockfile_anomalies"]
        .as_array()
        .map(|items| !items.is_empty())
        .unwrap_or(false)
    {
        push_once(&mut gaps, "dependency_state_requires_repo_review");
        push_once(&mut mandatory_shell_checks, "git status");
        push_once(&mut mandatory_shell_checks, "git diff");
    }

    (gaps, mandatory_shell_checks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_json_returns_no_gaps() {
        let (gaps, checks) = repo_truth_gap_projection(&json!({}));
        assert!(gaps.is_empty());
        assert!(checks.is_empty());
    }

    #[test]
    fn not_git_repository_status() {
        let input = json!({"status": "not_git_repository"});
        let (gaps, checks) = repo_truth_gap_projection(&input);
        assert_eq!(gaps, vec!["not_git_repository"]);
        assert!(checks.is_empty());
    }

    #[test]
    fn error_status_adds_gap_and_shell_check() {
        let input = json!({"status": "error"});
        let (gaps, checks) = repo_truth_gap_projection(&input);
        assert!(gaps.contains(&"git_metadata_unavailable".to_string()));
        assert!(checks.contains(&"git status".to_string()));
    }

    #[test]
    fn unknown_status_adds_nothing() {
        let input = json!({"status": "clean"});
        let (gaps, checks) = repo_truth_gap_projection(&input);
        assert!(gaps.is_empty());
        assert!(checks.is_empty());
    }

    #[test]
    fn operation_states_non_empty_adds_gap_and_checks() {
        let input = json!({
            "status": "clean",
            "operation_states": ["rebase"]
        });
        let (gaps, checks) = repo_truth_gap_projection(&input);
        assert!(gaps.contains(&"repository_mid_operation".to_string()));
        assert!(checks.contains(&"git status".to_string()));
        assert!(checks.contains(&"git diff".to_string()));
    }

    #[test]
    fn operation_states_empty_adds_nothing() {
        let input = json!({
            "status": "clean",
            "operation_states": []
        });
        let (gaps, checks) = repo_truth_gap_projection(&input);
        assert!(!gaps.contains(&"repository_mid_operation".to_string()));
    }

    #[test]
    fn conflicted_count_positive_adds_gap_and_checks() {
        let input = json!({
            "status": "clean",
            "conflicted_count": 3
        });
        let (gaps, checks) = repo_truth_gap_projection(&input);
        assert!(gaps.contains(&"working_tree_conflicted".to_string()));
        assert!(checks.contains(&"git status".to_string()));
        assert!(checks.contains(&"git diff".to_string()));
    }

    #[test]
    fn conflicted_count_zero_adds_nothing() {
        let input = json!({
            "status": "clean",
            "conflicted_count": 0
        });
        let (gaps, checks) = repo_truth_gap_projection(&input);
        assert!(!gaps.contains(&"working_tree_conflicted".to_string()));
    }

    #[test]
    fn lockfile_anomalies_non_empty_adds_gap_and_checks() {
        let input = json!({
            "status": "clean",
            "lockfile_anomalies": ["package-lock.json changed"]
        });
        let (gaps, checks) = repo_truth_gap_projection(&input);
        assert!(gaps.contains(&"dependency_state_requires_repo_review".to_string()));
        assert!(checks.contains(&"git status".to_string()));
        assert!(checks.contains(&"git diff".to_string()));
    }

    #[test]
    fn lockfile_anomalies_empty_adds_nothing() {
        let input = json!({
            "status": "clean",
            "lockfile_anomalies": []
        });
        let (gaps, checks) = repo_truth_gap_projection(&input);
        assert!(!gaps.contains(&"dependency_state_requires_repo_review".to_string()));
    }

    #[test]
    fn multiple_conditions_accumulate_gaps_without_duplicates() {
        let input = json!({
            "status": "error",
            "operation_states": ["merge"],
            "conflicted_count": 2,
            "lockfile_anomalies": ["anomaly"]
        });
        let (gaps, checks) = repo_truth_gap_projection(&input);
        // Each condition adds a gap
        assert!(gaps.contains(&"git_metadata_unavailable".to_string()));
        assert!(gaps.contains(&"repository_mid_operation".to_string()));
        assert!(gaps.contains(&"working_tree_conflicted".to_string()));
        assert!(gaps.contains(&"dependency_state_requires_repo_review".to_string()));
        // "git status" and "git diff" appear from multiple sources but push_once deduplicates
        assert_eq!(checks.iter().filter(|c| *c == "git status").count(), 1);
        assert_eq!(checks.iter().filter(|c| *c == "git diff").count(), 1);
    }

    #[test]
    fn missing_keys_is_treated_as_defaults() {
        let input = json!({"status": "clean"});
        let (gaps, checks) = repo_truth_gap_projection(&input);
        // No operation_states key -> missing -> treated as empty -> no gap
        // No conflicted_count key -> missing -> unwrap_or(0) -> no gap
        // No lockfile_anomalies key -> missing -> treated as empty -> no gap
        assert!(gaps.is_empty());
        assert!(checks.is_empty());
    }

    #[test]
    fn push_once_prevents_duplicates() {
        let mut items: Vec<String> = Vec::new();
        push_once(&mut items, "alpha");
        push_once(&mut items, "alpha");
        push_once(&mut items, "beta");
        assert_eq!(items, vec!["alpha", "beta"]);
    }
}
