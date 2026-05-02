use crate::storage::queries::VerificationRun;
use serde_json::{json, Value};

fn commands_from_array(value: &Value, key: &str) -> Vec<String> {
    value[key]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item.as_str())
        .map(|item| item.to_string())
        .collect()
}

fn latest_failing_command(runs: &[VerificationRun]) -> Option<String> {
    runs.iter()
        .find(|run| run.status != "passed" && !run.command.trim().is_empty())
        .map(|run| run.command.trim().to_string())
}

fn repo_stabilization_sequence(repo_risk: &Value) -> Value {
    let operation_active = repo_risk["operation_states"]
        .as_array()
        .map(|states| !states.is_empty())
        .unwrap_or(false);

    if !operation_active {
        return Value::Null;
    }

    json!({
        "mode": "shell_stabilize_then_resume",
        "current_phase": "stabilize",
        "resume_with": "refresh_guidance_after_repo_stable",
        "stability_checks": ["git status", "git diff"],
        "resume_conditions": [
            "operation_states_cleared",
            "conflicted_count_zero"
        ]
    })
}

fn missing_verification_sequence(project_toolchain: &Value) -> Value {
    let mut verification_commands = commands_from_array(project_toolchain, "recommended_test_commands");
    if verification_commands.is_empty() {
        verification_commands = commands_from_array(project_toolchain, "recommended_build_commands");
    }

    json!({
        "mode": "run_project_verification_then_resume",
        "current_phase": "verify",
        "resume_with": "refresh_guidance_after_verification",
        "verification_commands": verification_commands,
        "resume_conditions": [
            "required_verification_recorded",
            "verification_evidence_fresh"
        ]
    })
}

fn failing_verification_sequence(verification_runs: &[VerificationRun], project_toolchain: &Value) -> Value {
    let verification_commands = latest_failing_command(verification_runs)
        .map(|command| vec![command])
        .filter(|commands| !commands.is_empty())
        .unwrap_or_else(|| commands_from_array(project_toolchain, "recommended_test_commands"));

    json!({
        "mode": "resolve_failing_verification_then_resume",
        "current_phase": "repair_and_verify",
        "resume_with": "refresh_guidance_after_verification",
        "verification_commands": verification_commands,
        "resume_conditions": [
            "no_failing_verification_runs",
            "verification_evidence_fresh"
        ]
    })
}

pub(crate) fn execution_sequence_for_recommendation(
    forced_action: Option<&str>,
    repo_risk: &Value,
    verification_runs: &[VerificationRun],
    project_toolchain: &Value,
) -> Value {
    match forced_action {
        Some("review_failing_verification") => {
            failing_verification_sequence(verification_runs, project_toolchain)
        }
        Some("run_verification_before_high_risk_changes") => {
            missing_verification_sequence(project_toolchain)
        }
        Some("stabilize_repository_state") => repo_stabilization_sequence(repo_risk),
        _ => Value::Null,
    }
}
