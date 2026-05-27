use serde_json::Value;

use super::super::guidance_types::ExternalTruthBoundary;
use super::string_array_field;

fn push_once(items: &mut Vec<String>, value: &str) {
    if !items.iter().any(|item| item == value) {
        items.push(value.to_string());
    }
}

fn repo_state_triggers_for(recommendation: &Value) -> Vec<String> {
    string_array_field(recommendation, "repo_truth_gaps")
        .into_iter()
        .filter(|gap| {
            matches!(
                gap.as_str(),
                "repository_mid_operation"
                    | "working_tree_conflicted"
                    | "dependency_state_requires_repo_review"
                    | "git_metadata_unavailable"
            )
        })
        .collect()
}

fn verification_trigger_for(recommendation: &Value) -> Option<String> {
    match recommendation["execution_sequence"]["mode"]
        .as_str()
        .unwrap_or_default()
    {
        "run_project_verification_then_resume" => Some("verification_run_required".to_string()),
        "resolve_failing_verification_then_resume" => {
            Some("failing_verification_repair_required".to_string())
        }
        _ => None,
    }
}

fn verification_commands_for(recommendation: &Value) -> Vec<String> {
    recommendation["execution_sequence"]["verification_commands"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn external_truth_boundary_for_top_project(
    top_recommendation: Option<&Value>,
) -> ExternalTruthBoundary {
    let Some(recommendation) = top_recommendation else {
        return ExternalTruthBoundary::no_priority_project();
    };

    let repo_triggers = repo_state_triggers_for(recommendation);
    let verification_trigger = verification_trigger_for(recommendation);
    let repo_state_required = !repo_triggers.is_empty();
    let verification_required = verification_trigger.is_some();

    let mut triggers = repo_triggers.clone();
    if let Some(trigger) = &verification_trigger {
        triggers.push(trigger.clone());
    }

    let mut minimum_external_checks = Vec::new();
    for command in string_array_field(recommendation, "mandatory_shell_checks") {
        push_once(&mut minimum_external_checks, &command);
    }
    for command in verification_commands_for(recommendation) {
        push_once(&mut minimum_external_checks, &command);
    }

    let mode = if repo_state_required || verification_required {
        "must_switch_to_external_truth"
    } else {
        "opendog_guidance_can_continue"
    };

    let summary = match (repo_state_required, verification_required) {
        (true, true) => {
            "Top project needs direct repository and verification truth before broader changes."
        }
        (true, false) => "Top project needs direct repository truth before broader changes.",
        (false, true) => {
            "Top project needs fresh project-native verification truth before broader changes."
        }
        (false, false) => {
            "Current top recommendation can continue under OPENDOG guidance until a repository or verification boundary is reached."
        }
    };

    ExternalTruthBoundary::available(
        recommendation["project_id"].clone(),
        mode,
        repo_state_required,
        verification_required,
        triggers,
        minimum_external_checks,
        summary,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boundary_value(boundary: ExternalTruthBoundary) -> Value {
        serde_json::to_value(boundary).unwrap()
    }

    // --- repo_state_triggers_for ---

    #[test]
    fn repo_state_triggers_filters_known_gaps() {
        let rec = serde_json::json!({
            "repo_truth_gaps": ["repository_mid_operation", "working_tree_conflicted", "some_unknown_gap"]
        });
        let triggers = repo_state_triggers_for(&rec);
        assert!(triggers.contains(&"repository_mid_operation".to_string()));
        assert!(triggers.contains(&"working_tree_conflicted".to_string()));
        assert!(!triggers.contains(&"some_unknown_gap".to_string()));
    }

    #[test]
    fn repo_state_triggers_all_known_gaps() {
        let rec = serde_json::json!({
            "repo_truth_gaps": [
                "repository_mid_operation",
                "working_tree_conflicted",
                "dependency_state_requires_repo_review",
                "git_metadata_unavailable"
            ]
        });
        let triggers = repo_state_triggers_for(&rec);
        assert_eq!(triggers.len(), 4);
    }

    #[test]
    fn repo_state_triggers_empty_json() {
        let rec = serde_json::json!({});
        let triggers = repo_state_triggers_for(&rec);
        assert!(triggers.is_empty());
    }

    #[test]
    fn repo_state_triggers_non_array_returns_empty() {
        let rec = serde_json::json!({"repo_truth_gaps": "not_array"});
        let triggers = repo_state_triggers_for(&rec);
        assert!(triggers.is_empty());
    }

    // --- verification_trigger_for ---

    #[test]
    fn verification_trigger_run_then_resume() {
        let rec = serde_json::json!({
            "execution_sequence": {"mode": "run_project_verification_then_resume"}
        });
        assert_eq!(
            verification_trigger_for(&rec),
            Some("verification_run_required".to_string())
        );
    }

    #[test]
    fn verification_trigger_resolve_failing() {
        let rec = serde_json::json!({
            "execution_sequence": {"mode": "resolve_failing_verification_then_resume"}
        });
        assert_eq!(
            verification_trigger_for(&rec),
            Some("failing_verification_repair_required".to_string())
        );
    }

    #[test]
    fn verification_trigger_other_mode_returns_none() {
        let rec = serde_json::json!({
            "execution_sequence": {"mode": "proceed_normally"}
        });
        assert!(verification_trigger_for(&rec).is_none());
    }

    #[test]
    fn verification_trigger_missing_execution_sequence() {
        let rec = serde_json::json!({});
        assert!(verification_trigger_for(&rec).is_none());
    }

    // --- verification_commands_for ---

    #[test]
    fn verification_commands_extracts_strings() {
        let rec = serde_json::json!({
            "execution_sequence": {
                "verification_commands": ["cargo test", "cargo clippy"]
            }
        });
        let cmds = verification_commands_for(&rec);
        assert_eq!(cmds, vec!["cargo test", "cargo clippy"]);
    }

    #[test]
    fn verification_commands_empty_array() {
        let rec = serde_json::json!({
            "execution_sequence": {"verification_commands": []}
        });
        let cmds = verification_commands_for(&rec);
        assert!(cmds.is_empty());
    }

    #[test]
    fn verification_commands_missing_key() {
        let rec = serde_json::json!({});
        let cmds = verification_commands_for(&rec);
        assert!(cmds.is_empty());
    }

    #[test]
    fn verification_commands_non_array_returns_empty() {
        let rec = serde_json::json!({
            "execution_sequence": {"verification_commands": "not_array"}
        });
        let cmds = verification_commands_for(&rec);
        assert!(cmds.is_empty());
    }

    // --- external_truth_boundary_for_top_project ---

    #[test]
    fn none_returns_no_priority_project() {
        let result = boundary_value(external_truth_boundary_for_top_project(None));
        assert_eq!(result["status"], "no_priority_project");
        assert!(result["source"].is_null());
        assert!(result["source_project_id"].is_null());
        assert!(result["mode"].is_null());
        assert_eq!(result["repo_state_required"], false);
        assert_eq!(result["verification_required"], false);
        assert!(result["triggers"].as_array().unwrap().is_empty());
        assert!(result["minimum_external_checks"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(result["summary"].is_null());
    }

    #[test]
    fn with_repo_triggers_only() {
        let rec = serde_json::json!({
            "project_id": "proj1",
            "repo_truth_gaps": ["working_tree_conflicted"],
            "mandatory_shell_checks": ["git status"],
            "execution_sequence": {"mode": "proceed_normally", "verification_commands": []}
        });
        let result = boundary_value(external_truth_boundary_for_top_project(Some(&rec)));
        assert_eq!(result["status"], "available");
        assert_eq!(result["source"], "top_priority_project");
        assert_eq!(result["source_project_id"], "proj1");
        assert_eq!(result["mode"], "must_switch_to_external_truth");
        assert_eq!(result["repo_state_required"], true);
        assert_eq!(result["verification_required"], false);
        assert!(result["summary"]
            .as_str()
            .unwrap()
            .contains("repository truth"));
    }

    #[test]
    fn with_verification_trigger_only() {
        let rec = serde_json::json!({
            "project_id": "proj2",
            "repo_truth_gaps": [],
            "mandatory_shell_checks": [],
            "execution_sequence": {
                "mode": "run_project_verification_then_resume",
                "verification_commands": ["cargo test"]
            }
        });
        let result = boundary_value(external_truth_boundary_for_top_project(Some(&rec)));
        assert_eq!(result["status"], "available");
        assert_eq!(result["repo_state_required"], false);
        assert_eq!(result["verification_required"], true);
        assert_eq!(result["mode"], "must_switch_to_external_truth");
        assert!(result["summary"].as_str().unwrap().contains("verification"));
        // external checks include verification command
        let checks = result["minimum_external_checks"].as_array().unwrap();
        assert!(checks.iter().any(|c| c == "cargo test"));
    }

    #[test]
    fn with_both_repo_and_verification_triggers() {
        let rec = serde_json::json!({
            "project_id": "proj3",
            "repo_truth_gaps": ["repository_mid_operation"],
            "mandatory_shell_checks": ["git status", "git diff"],
            "execution_sequence": {
                "mode": "run_project_verification_then_resume",
                "verification_commands": ["npm test"]
            }
        });
        let result = boundary_value(external_truth_boundary_for_top_project(Some(&rec)));
        assert_eq!(result["repo_state_required"], true);
        assert_eq!(result["verification_required"], true);
        assert!(result["summary"]
            .as_str()
            .unwrap()
            .contains("repository and verification"));
        let checks = result["minimum_external_checks"].as_array().unwrap();
        assert!(checks.iter().any(|c| c == "git status"));
        assert!(checks.iter().any(|c| c == "git diff"));
        assert!(checks.iter().any(|c| c == "npm test"));
    }

    #[test]
    fn can_continue_when_no_triggers() {
        let rec = serde_json::json!({
            "project_id": "proj4",
            "repo_truth_gaps": [],
            "mandatory_shell_checks": [],
            "execution_sequence": {"mode": "proceed_normally", "verification_commands": []}
        });
        let result = boundary_value(external_truth_boundary_for_top_project(Some(&rec)));
        assert_eq!(result["mode"], "opendog_guidance_can_continue");
        assert!(result["summary"]
            .as_str()
            .unwrap()
            .contains("continue under OPENDOG"));
    }

    #[test]
    fn minimum_external_checks_deduplicates() {
        let rec = serde_json::json!({
            "project_id": "proj5",
            "repo_truth_gaps": [],
            "mandatory_shell_checks": ["git status"],
            "execution_sequence": {
                "mode": "run_project_verification_then_resume",
                "verification_commands": ["git status"]
            }
        });
        let result = boundary_value(external_truth_boundary_for_top_project(Some(&rec)));
        let checks = result["minimum_external_checks"].as_array().unwrap();
        let git_status_count = checks
            .iter()
            .filter(|c| c.as_str() == Some("git status"))
            .count();
        assert_eq!(git_status_count, 1);
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
