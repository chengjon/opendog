use super::*;
use crate::storage::queries::VerificationRun;

fn make_verification_run(status: &str, command: &str) -> VerificationRun {
    VerificationRun {
        id: 1,
        kind: "test".to_string(),
        status: status.to_string(),
        command: command.to_string(),
        exit_code: Some(1),
        summary: None,
        source: "cli".to_string(),
        started_at: None,
        finished_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

fn low_risk_repo() -> Value {
    json!({
        "risk_level": "low",
        "operation_states": [],
        "large_diff": false,
    })
}

fn mid_operation_repo() -> Value {
    json!({
        "risk_level": "low",
        "operation_states": ["merge"],
        "large_diff": false,
    })
}

fn toolchain_with_test_commands() -> Value {
    json!({
        "recommended_test_commands": ["cargo test", "npm test"],
        "recommended_build_commands": ["cargo build"],
    })
}

fn toolchain_with_build_commands_only() -> Value {
    json!({
        "recommended_test_commands": [],
        "recommended_build_commands": ["cargo build"],
    })
}

fn empty_toolchain() -> Value {
    json!({
        "recommended_test_commands": [],
        "recommended_build_commands": [],
    })
}

// --- repo_stabilization_sequence (via execution dispatch) ---

#[test]
fn repo_stabilization_returns_null_when_no_operation() {
    let result = execution_sequence_for_recommendation(
        "stabilize_repository_state",
        &low_risk_repo(),
        &[],
        &empty_toolchain(),
    );
    assert!(result.is_null());
}

#[test]
fn repo_stabilization_returns_structure_when_operation_active() {
    let result = execution_sequence_for_recommendation(
        "stabilize_repository_state",
        &mid_operation_repo(),
        &[],
        &empty_toolchain(),
    );
    assert_eq!(result["mode"], "shell_stabilize_then_resume");
    assert_eq!(result["current_phase"], "stabilize");
    assert_eq!(result["resume_with"], "refresh_guidance_after_repo_stable");
    assert!(result["stability_checks"].is_array());
    assert!(result["resume_conditions"].is_array());
}

// --- monitor_start_sequence ---

#[test]
fn monitor_start_sequence_structure() {
    let result = execution_sequence_for_recommendation(
        "start_monitor",
        &low_risk_repo(),
        &[],
        &empty_toolchain(),
    );
    assert_eq!(result["mode"], "start_monitor_then_resume");
    assert_eq!(result["current_phase"], "enable_monitoring");
    assert_eq!(result["resume_with"], "refresh_guidance_after_observation");
    assert!(result["observation_steps"].is_array());
    assert!(result["resume_conditions"].is_array());
}

// --- snapshot_refresh_sequence ---

#[test]
fn snapshot_refresh_sequence_structure() {
    let result = execution_sequence_for_recommendation(
        "take_snapshot",
        &low_risk_repo(),
        &[],
        &empty_toolchain(),
    );
    assert_eq!(result["mode"], "refresh_snapshot_then_resume");
    assert_eq!(result["current_phase"], "snapshot");
    assert_eq!(result["resume_with"], "refresh_guidance_after_snapshot");
    assert!(result["observation_steps"].is_array());
    assert!(result["resume_conditions"].is_array());
}

// --- activity_generation_sequence ---

#[test]
fn activity_generation_sequence_structure() {
    let result = execution_sequence_for_recommendation(
        "generate_activity_then_stats",
        &low_risk_repo(),
        &[],
        &empty_toolchain(),
    );
    assert_eq!(result["mode"], "generate_activity_then_resume");
    assert_eq!(result["current_phase"], "generate_activity");
    assert_eq!(result["resume_with"], "refresh_guidance_after_activity");
    assert!(result["observation_steps"].is_array());
    assert!(result["resume_conditions"].is_array());
}

// --- missing_verification_sequence ---

#[test]
fn missing_verification_uses_test_commands() {
    let result = execution_sequence_for_recommendation(
        "run_verification_before_high_risk_changes",
        &low_risk_repo(),
        &[],
        &toolchain_with_test_commands(),
    );
    assert_eq!(result["mode"], "run_project_verification_then_resume");
    assert_eq!(result["current_phase"], "verify");
    let cmds = result["verification_commands"].as_array().unwrap();
    assert_eq!(cmds.len(), 2);
    assert_eq!(cmds[0], "cargo test");
    assert_eq!(cmds[1], "npm test");
}

#[test]
fn missing_verification_falls_back_to_build_commands() {
    let result = execution_sequence_for_recommendation(
        "run_verification_before_high_risk_changes",
        &low_risk_repo(),
        &[],
        &toolchain_with_build_commands_only(),
    );
    let cmds = result["verification_commands"].as_array().unwrap();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], "cargo build");
}

#[test]
fn missing_verification_empty_when_no_commands() {
    let result = execution_sequence_for_recommendation(
        "run_verification_before_high_risk_changes",
        &low_risk_repo(),
        &[],
        &empty_toolchain(),
    );
    let cmds = result["verification_commands"].as_array().unwrap();
    assert!(cmds.is_empty());
}

// --- failing_verification_sequence ---

#[test]
fn failing_verification_uses_latest_failing_command() {
    let runs = vec![
        make_verification_run("passed", "cargo test"),
        make_verification_run("failed", "cargo test --ignored"),
    ];
    let result = execution_sequence_for_recommendation(
        "review_failing_verification",
        &low_risk_repo(),
        &runs,
        &toolchain_with_test_commands(),
    );
    assert_eq!(result["mode"], "resolve_failing_verification_then_resume");
    assert_eq!(result["current_phase"], "repair_and_verify");
    let cmds = result["verification_commands"].as_array().unwrap();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], "cargo test --ignored");
}

#[test]
fn failing_verification_falls_back_to_test_commands() {
    let runs = vec![make_verification_run("passed", "cargo test")];
    let result = execution_sequence_for_recommendation(
        "review_failing_verification",
        &low_risk_repo(),
        &runs,
        &toolchain_with_test_commands(),
    );
    let cmds = result["verification_commands"].as_array().unwrap();
    assert_eq!(cmds[0], "cargo test");
}

#[test]
fn failing_verification_skips_empty_commands() {
    let runs = vec![make_verification_run("failed", "   ")];
    let result = execution_sequence_for_recommendation(
        "review_failing_verification",
        &low_risk_repo(),
        &runs,
        &toolchain_with_test_commands(),
    );
    // Empty-trimmed command should be skipped, fallback to test commands
    let cmds = result["verification_commands"].as_array().unwrap();
    assert_eq!(cmds[0], "cargo test");
}

// --- unknown action returns null ---

#[test]
fn unknown_action_returns_null() {
    let result = execution_sequence_for_recommendation(
        "some_unknown_action",
        &low_risk_repo(),
        &[],
        &empty_toolchain(),
    );
    assert!(result.is_null());
}
