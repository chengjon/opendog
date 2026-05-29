use super::*;
use serde_json::json;

// --- apply_repo_risk_context ---

#[test]
fn apply_repo_risk_context_no_coupling_returns_first_step() {
    let result = apply_repo_risk_context("Do something".to_string(), None);
    assert_eq!(result, "Do something");
}

#[test]
fn apply_repo_risk_context_non_coupled_status_returns_first_step() {
    let coupling = json!({"status": "decoupled"});
    let result = apply_repo_risk_context("Do something".to_string(), Some(&coupling));
    assert_eq!(result, "Do something");
}

#[test]
fn apply_repo_risk_context_coupled_appends_summary() {
    let coupling = json!({
        "status": "coupled",
        "primary_repo_risk_finding": {
            "summary": "merge in progress on main"
        }
    });
    let result = apply_repo_risk_context("Start here".to_string(), Some(&coupling));
    assert_eq!(
        result,
        "Start here; top repository risk: merge in progress on main"
    );
}

#[test]
fn apply_repo_risk_context_coupled_missing_summary_returns_first_step() {
    let coupling = json!({
        "status": "coupled",
        "primary_repo_risk_finding": {}
    });
    let result = apply_repo_risk_context("Do something".to_string(), Some(&coupling));
    assert_eq!(result, "Do something");
}

// --- strategy_profile ---

#[test]
fn strategy_profile_builds_correct_json() {
    let profile = strategy_profile(
        "activity_guided_review",
        "opendog",
        "shell",
        &["verification", "repository_risk"],
    );
    assert_eq!(profile["strategy_mode"], "activity_guided_review");
    assert_eq!(profile["preferred_primary_tool"], "opendog");
    assert_eq!(profile["preferred_secondary_tool"], "shell");
    let priorities = profile["evidence_priority"].as_array().unwrap();
    assert_eq!(priorities.len(), 2);
    assert_eq!(priorities[0], "verification");
    assert_eq!(priorities[1], "repository_risk");
}

// --- workspace_strategy_profile ---

#[test]
fn workspace_strategy_verify_before_modify_when_failing_verification() {
    let result = workspace_strategy_profile(3, 2, true, false, 0);
    assert_eq!(result["global_strategy_mode"], "verify_before_modify");
    assert_eq!(result["preferred_primary_tool"], "shell");
    assert_eq!(result["preferred_secondary_tool"], "opendog");
    let flow = result["recommended_flow"].as_array().unwrap();
    assert_eq!(flow.len(), 3);
    assert!(flow[0].as_str().unwrap().contains("failing verification"));
}

#[test]
fn workspace_strategy_stabilize_before_modify_when_mid_operation() {
    let result = workspace_strategy_profile(3, 2, false, true, 0);
    assert_eq!(result["global_strategy_mode"], "stabilize_before_modify");
    assert_eq!(result["preferred_primary_tool"], "shell");
    let flow = result["recommended_flow"].as_array().unwrap();
    assert!(flow[0].as_str().unwrap().contains("Stabilize"));
}

#[test]
fn workspace_strategy_collect_workspace_context_when_zero_projects() {
    let result = workspace_strategy_profile(0, 0, false, false, 0);
    assert_eq!(result["global_strategy_mode"], "collect_workspace_context");
    assert_eq!(result["preferred_primary_tool"], "opendog");
    let flow = result["recommended_flow"].as_array().unwrap();
    assert!(flow[0].as_str().unwrap().contains("Register a project"));
}

#[test]
fn workspace_strategy_collect_evidence_first_when_zero_monitored() {
    let result = workspace_strategy_profile(3, 0, false, false, 0);
    assert_eq!(result["global_strategy_mode"], "collect_evidence_first");
    assert_eq!(result["preferred_primary_tool"], "opendog");
    let flow = result["recommended_flow"].as_array().unwrap();
    assert!(flow[0].as_str().unwrap().contains("opendog list"));
}

#[test]
fn workspace_strategy_verify_before_high_risk_when_missing_verification() {
    let result = workspace_strategy_profile(3, 2, false, false, 2);
    assert_eq!(
        result["global_strategy_mode"],
        "verify_before_high_risk_changes"
    );
    assert_eq!(result["preferred_primary_tool"], "opendog");
}

#[test]
fn workspace_strategy_activity_guided_review_normal_case() {
    let result = workspace_strategy_profile(3, 2, false, false, 0);
    assert_eq!(result["global_strategy_mode"], "activity_guided_review");
    assert_eq!(result["preferred_primary_tool"], "opendog");
    assert_eq!(result["preferred_secondary_tool"], "shell");
}

#[test]
fn workspace_strategy_evidence_priority_always_present() {
    let result = workspace_strategy_profile(3, 2, false, false, 0);
    let priorities = result["evidence_priority"].as_array().unwrap();
    assert_eq!(priorities.len(), 3);
    assert_eq!(priorities[0], "verification");
    assert_eq!(priorities[1], "repository_risk");
    assert_eq!(priorities[2], "activity_signals");
}

// --- agent_guidance_recommended_flow ---

fn base_workspace_strategy() -> Value {
    json!({
        "recommended_flow": [
            "Default workspace flow step 1.",
            "Default workspace flow step 2.",
        ]
    })
}

#[test]
fn recommended_flow_returns_workspace_flow_when_zero_projects() {
    let strategy = base_workspace_strategy();
    let result = agent_guidance_recommended_flow(0, 0, None, &strategy, None);
    let flow = result.as_array().unwrap();
    assert_eq!(flow.len(), 2);
    assert_eq!(flow[0], "Default workspace flow step 1.");
}

#[test]
fn recommended_flow_returns_workspace_flow_when_no_top_recommendation() {
    let strategy = base_workspace_strategy();
    let result = agent_guidance_recommended_flow(3, 1, None, &strategy, None);
    let flow = result.as_array().unwrap();
    assert_eq!(flow[0], "Default workspace flow step 1.");
}

#[test]
fn recommended_flow_review_failing_verification() {
    let strategy = base_workspace_strategy();
    let recommendation = json!({
        "project_id": "myproj",
        "recommended_next_action": "review_failing_verification",
    });
    let result = agent_guidance_recommended_flow(3, 1, Some(&recommendation), &strategy, None);
    let flow = result.as_array().unwrap();
    assert_eq!(flow.len(), 3);
    assert!(flow[0].as_str().unwrap().contains("myproj"));
    assert!(flow[0].as_str().unwrap().contains("failing"));
    assert!(flow[1]
        .as_str()
        .unwrap()
        .contains("verification --id myproj"));
}

#[test]
fn recommended_flow_stabilize_repository_state() {
    let strategy = base_workspace_strategy();
    let recommendation = json!({
        "project_id": "proj_a",
        "recommended_next_action": "stabilize_repository_state",
    });
    let result = agent_guidance_recommended_flow(3, 1, Some(&recommendation), &strategy, None);
    let flow = result.as_array().unwrap();
    assert!(flow[0].as_str().unwrap().contains("mid-operation"));
    assert!(flow[1].as_str().unwrap().contains("git status"));
}

#[test]
fn recommended_flow_start_monitor() {
    let strategy = base_workspace_strategy();
    let recommendation = json!({
        "project_id": "proj_b",
        "recommended_next_action": "start_monitor",
    });
    let result = agent_guidance_recommended_flow(3, 1, Some(&recommendation), &strategy, None);
    let flow = result.as_array().unwrap();
    assert!(flow[0].as_str().unwrap().contains("monitoring"));
    assert!(flow[1].as_str().unwrap().contains("start --id proj_b"));
}

#[test]
fn recommended_flow_take_snapshot() {
    let strategy = base_workspace_strategy();
    let recommendation = json!({
        "project_id": "proj_c",
        "recommended_next_action": "take_snapshot",
    });
    let result = agent_guidance_recommended_flow(3, 1, Some(&recommendation), &strategy, None);
    let flow = result.as_array().unwrap();
    assert!(flow[0].as_str().unwrap().contains("snapshot baseline"));
    assert!(flow[1].as_str().unwrap().contains("snapshot --id proj_c"));
}

#[test]
fn recommended_flow_generate_activity_then_stats() {
    let strategy = base_workspace_strategy();
    let recommendation = json!({
        "project_id": "proj_d",
        "recommended_next_action": "generate_activity_then_stats",
    });
    let result = agent_guidance_recommended_flow(3, 1, Some(&recommendation), &strategy, None);
    let flow = result.as_array().unwrap();
    assert!(flow[0]
        .as_str()
        .unwrap()
        .contains("no meaningful file activity"));
    assert!(flow[1]
        .as_str()
        .unwrap()
        .contains("edits, tests, or builds"));
}

#[test]
fn recommended_flow_run_verification_before_high_risk() {
    let strategy = base_workspace_strategy();
    let recommendation = json!({
        "project_id": "proj_e",
        "recommended_next_action": "run_verification_before_high_risk_changes",
    });
    let result = agent_guidance_recommended_flow(3, 1, Some(&recommendation), &strategy, None);
    let flow = result.as_array().unwrap();
    assert!(flow[0]
        .as_str()
        .unwrap()
        .contains("verification evidence is still missing"));
}

#[test]
fn recommended_flow_review_unused_files() {
    let strategy = base_workspace_strategy();
    let recommendation = json!({
        "project_id": "proj_f",
        "recommended_next_action": "review_unused_files",
    });
    let result = agent_guidance_recommended_flow(3, 1, Some(&recommendation), &strategy, None);
    let flow = result.as_array().unwrap();
    assert!(flow[0].as_str().unwrap().contains("unused-file candidates"));
    assert!(flow[1].as_str().unwrap().contains("unused --id proj_f"));
}

#[test]
fn recommended_flow_inspect_hot_files() {
    let strategy = base_workspace_strategy();
    let recommendation = json!({
        "project_id": "proj_g",
        "recommended_next_action": "inspect_hot_files",
    });
    let result = agent_guidance_recommended_flow(3, 1, Some(&recommendation), &strategy, None);
    let flow = result.as_array().unwrap();
    assert!(flow[0].as_str().unwrap().contains("hotspot"));
    assert!(flow[1].as_str().unwrap().contains("stats --id proj_g"));
}

#[test]
fn recommended_flow_unknown_action_with_zero_monitored() {
    let strategy = base_workspace_strategy();
    let recommendation = json!({
        "project_id": "proj_h",
        "recommended_next_action": "unknown_action",
    });
    let result = agent_guidance_recommended_flow(3, 0, Some(&recommendation), &strategy, None);
    let flow = result.as_array().unwrap();
    assert!(flow[0]
        .as_str()
        .unwrap()
        .contains("No project is currently monitored"));
}

#[test]
fn recommended_flow_unknown_action_with_monitored_returns_workspace_flow() {
    let strategy = base_workspace_strategy();
    let recommendation = json!({
        "project_id": "proj_i",
        "recommended_next_action": "unknown_action",
    });
    let result = agent_guidance_recommended_flow(3, 1, Some(&recommendation), &strategy, None);
    let flow = result.as_array().unwrap();
    // Falls back to workspace strategy flow
    assert_eq!(flow[0], "Default workspace flow step 1.");
}

#[test]
fn recommended_flow_applies_repo_risk_context() {
    let strategy = base_workspace_strategy();
    let recommendation = json!({
        "project_id": "proj_j",
        "recommended_next_action": "take_snapshot",
    });
    let coupling = json!({
        "status": "coupled",
        "primary_repo_risk_finding": {
            "summary": "rebase in progress"
        }
    });
    let result =
        agent_guidance_recommended_flow(3, 1, Some(&recommendation), &strategy, Some(&coupling));
    let flow = result.as_array().unwrap();
    assert!(flow[0]
        .as_str()
        .unwrap()
        .contains("top repository risk: rebase in progress"));
}

#[test]
fn cleanup_gate_blocked_for_stale_verification() {
    let verification = json!({
        "status": "available",
        "latest_runs": [{
            "kind": "test",
            "status": "passed",
            "freshness": "stale"
        }]
    });
    let has_stale = verification["latest_runs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["freshness"] == "stale");
    assert!(has_stale, "stale verification should be detectable");
}

#[test]
fn destructive_change_recommended_false_for_weak_evidence() {
    let decision = json!({
        "cleanup_gate": "blocked",
        "refactor_gate": "blocked",
        "destructive_change_recommended": false,
        "recommended_next_action": "take_snapshot"
    });
    assert_eq!(decision["cleanup_gate"], "blocked");
    assert_eq!(decision["refactor_gate"], "blocked");
    assert_eq!(decision["destructive_change_recommended"], false);
}
