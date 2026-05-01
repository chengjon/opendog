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
