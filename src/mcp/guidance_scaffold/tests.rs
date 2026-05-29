use super::*;
use serde_json::json;

// --- default_shell_verification_commands ---

#[test]
fn default_shell_verification_commands_contains_git_commands() {
    let cmds = default_shell_verification_commands();
    assert_eq!(cmds.len(), 3);
    assert_eq!(cmds[0], "git status");
    assert_eq!(cmds[1], "git diff");
    assert_eq!(cmds[2], "project-native tests and builds");
}

// --- base_guidance_layers ---

#[test]
fn base_guidance_layers_has_all_required_layer_keys() {
    let layers = base_guidance_layers();
    assert_eq!(layers["workspace_observation"]["status"], "not_assessed");
    assert_eq!(layers["repo_status_risk"]["status"], "not_collected");
    assert_eq!(layers["execution_strategy"]["status"], "not_assessed");
    assert_eq!(layers["verification_evidence"]["status"], "not_assessed");
    assert_eq!(layers["multi_project_portfolio"]["status"], "not_assessed");
    assert_eq!(layers["storage_maintenance"]["status"], "not_assessed");
    assert_eq!(
        layers["cleanup_refactor_candidates"]["status"],
        "not_assessed"
    );
    assert_eq!(
        layers["cleanup_refactor_candidates"]["candidates"],
        json!([])
    );
    assert_eq!(layers["project_toolchain"]["status"], "not_assessed");
    assert_eq!(layers["constraints_boundaries"]["status"], "not_assessed");
    assert_eq!(layers["governance"]["status"], "not_assessed");
}

#[test]
fn base_guidance_layers_repo_status_has_summary() {
    let layers = base_guidance_layers();
    let summary = layers["repo_status_risk"]["summary"].as_str().unwrap();
    assert!(summary.contains("not yet collected"));
}

// --- tool_guidance ---

#[test]
fn tool_guidance_basic_structure() {
    let result = tool_guidance(
        "Inspect project stats.",
        &["opendog stats --id x", "git diff"],
        &["get_stats", "get_unused_files"],
        None,
    );
    assert_eq!(result["schema_version"], MCP_GUIDANCE_V1);
    assert_eq!(result["summary"], "Inspect project stats.");
    assert_eq!(
        result["suggested_commands"],
        json!(["opendog stats --id x", "git diff"])
    );
    assert_eq!(
        result["next_tools"],
        json!(["get_stats", "get_unused_files"])
    );
    assert!(result.get("when_to_use_shell").is_none());
}

#[test]
fn tool_guidance_with_shell_hint() {
    let result = tool_guidance(
        "Review unused files.",
        &["opendog unused --id x"],
        &["get_unused_files"],
        Some("Use shell when imports need manual validation."),
    );
    assert_eq!(
        result["when_to_use_shell"],
        "Use shell when imports need manual validation."
    );
    assert_eq!(
        result["layers"]["execution_strategy"]["when_to_use_shell"],
        "Use shell when imports need manual validation."
    );
}

#[test]
fn tool_guidance_overwrites_execution_strategy_layer() {
    let result = tool_guidance("Check workspace.", &["cmd1"], &["tool1"], None);
    let strategy = &result["layers"]["execution_strategy"];
    assert_eq!(strategy["status"], "available");
    assert_eq!(strategy["recommended_flow"], json!(["Check workspace."]));
    assert_eq!(strategy["suggested_commands"], json!(["cmd1"]));
    assert_eq!(strategy["next_tools"], json!(["tool1"]));
    assert!(strategy["guardrails"].is_array());
    assert!(strategy["destructive_operations_requiring_confirmation"].is_array());
}

#[test]
fn tool_guidance_sets_verification_evidence_layer() {
    let result = tool_guidance("summary", &[], &[], None);
    let ve = &result["layers"]["verification_evidence"];
    assert_eq!(ve["status"], "partial");
    assert_eq!(ve["confidence"], "medium");
    assert!(ve["direct_observations"].is_array());
    assert!(ve["inferences"].is_array());
}

#[test]
fn tool_guidance_sets_constraints_boundaries_layer() {
    let result = tool_guidance("summary", &[], &[], None);
    let cb = &result["layers"]["constraints_boundaries"];
    assert_eq!(cb["status"], "available");
    assert!(cb["guardrails"].is_array());
    assert!(cb["destructive_operations_requiring_confirmation"].is_array());
    assert!(cb["requires_shell_verification"].is_array());
}

// --- set_recommended_flow ---

#[test]
fn set_recommended_flow_updates_top_level_and_layer() {
    let mut value = json!({
        "recommended_flow": ["old step"],
        "layers": {
            "execution_strategy": {
                "recommended_flow": ["old step"],
            },
        },
    });
    set_recommended_flow(&mut value, &["step A", "step B"]);
    assert_eq!(value["recommended_flow"], json!(["step A", "step B"]));
    assert_eq!(
        value["layers"]["execution_strategy"]["recommended_flow"],
        json!(["step A", "step B"])
    );
}

#[test]
fn set_recommended_flow_empty_steps() {
    let mut value = json!({
        "recommended_flow": ["old"],
        "layers": { "execution_strategy": { "recommended_flow": ["old"] } },
    });
    set_recommended_flow(&mut value, &[]);
    assert_eq!(value["recommended_flow"], json!([]));
    assert_eq!(
        value["layers"]["execution_strategy"]["recommended_flow"],
        json!([])
    );
}

#[test]
fn set_recommended_flow_creates_missing_path() {
    let mut value = json!({});
    set_recommended_flow(&mut value, &["new"]);
    assert_eq!(value["recommended_flow"], json!(["new"]));
}
