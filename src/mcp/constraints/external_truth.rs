use serde_json::{json, Value};

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

pub(crate) fn external_truth_boundary_for_top_project(top_recommendation: Option<&Value>) -> Value {
    let Some(recommendation) = top_recommendation else {
        return json!({
            "status": "no_priority_project",
            "source": Value::Null,
            "source_project_id": Value::Null,
            "mode": Value::Null,
            "repo_state_required": false,
            "verification_required": false,
            "triggers": [],
            "minimum_external_checks": [],
            "summary": Value::Null
        });
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

    json!({
        "status": "available",
        "source": "top_priority_project",
        "source_project_id": recommendation["project_id"].clone(),
        "mode": mode,
        "repo_state_required": repo_state_required,
        "verification_required": verification_required,
        "triggers": triggers,
        "minimum_external_checks": minimum_external_checks,
        "summary": summary,
    })
}
