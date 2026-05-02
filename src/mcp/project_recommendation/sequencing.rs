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

fn monitor_start_sequence() -> Value {
    json!({
        "mode": "start_monitor_then_resume",
        "current_phase": "enable_monitoring",
        "resume_with": "refresh_guidance_after_observation",
        "observation_steps": ["start_monitor", "generate_real_project_activity"],
        "resume_conditions": [
            "monitoring_active",
            "activity_evidence_recorded"
        ]
    })
}

fn snapshot_refresh_sequence() -> Value {
    json!({
        "mode": "refresh_snapshot_then_resume",
        "current_phase": "snapshot",
        "resume_with": "refresh_guidance_after_snapshot",
        "observation_steps": ["take_snapshot"],
        "resume_conditions": [
            "snapshot_available",
            "snapshot_evidence_fresh"
        ]
    })
}

fn activity_generation_sequence() -> Value {
    json!({
        "mode": "generate_activity_then_resume",
        "current_phase": "generate_activity",
        "resume_with": "refresh_guidance_after_activity",
        "observation_steps": ["generate_real_project_activity", "refresh_stats"],
        "resume_conditions": [
            "activity_evidence_recorded",
            "activity_evidence_fresh"
        ]
    })
}

fn missing_verification_sequence(project_toolchain: &Value) -> Value {
    let mut verification_commands =
        commands_from_array(project_toolchain, "recommended_test_commands");
    if verification_commands.is_empty() {
        verification_commands =
            commands_from_array(project_toolchain, "recommended_build_commands");
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

fn failing_verification_sequence(
    verification_runs: &[VerificationRun],
    project_toolchain: &Value,
) -> Value {
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
    selected_action: &str,
    repo_risk: &Value,
    verification_runs: &[VerificationRun],
    project_toolchain: &Value,
) -> Value {
    match selected_action {
        "review_failing_verification" => {
            failing_verification_sequence(verification_runs, project_toolchain)
        }
        "run_verification_before_high_risk_changes" => {
            missing_verification_sequence(project_toolchain)
        }
        "stabilize_repository_state" => repo_stabilization_sequence(repo_risk),
        "start_monitor" => monitor_start_sequence(),
        "take_snapshot" => snapshot_refresh_sequence(),
        "generate_activity_then_stats" => activity_generation_sequence(),
        _ => Value::Null,
    }
}
