use serde_json::{json, Value};

use crate::contracts::MCP_GUIDANCE_V1;

use super::constraints::{
    build_constraints_boundaries_layer, default_boundary_guardrails, default_destructive_operations,
};

pub(super) fn tool_guidance(
    summary: &str,
    suggested_commands: &[&str],
    next_tools: &[&str],
    when_to_use_shell: Option<&str>,
) -> Value {
    let mut value = json!({
        "schema_version": MCP_GUIDANCE_V1,
        "summary": summary,
        "suggested_commands": suggested_commands,
        "next_tools": next_tools,
        "layers": base_guidance_layers(),
    });
    value["layers"]["execution_strategy"] = json!({
        "status": "available",
        "recommended_flow": [summary],
        "suggested_commands": suggested_commands,
        "next_tools": next_tools,
        "guardrails": default_boundary_guardrails(),
        "destructive_operations_requiring_confirmation": default_destructive_operations(),
    });
    value["layers"]["verification_evidence"] = json!({
        "status": "partial",
        "direct_observations": ["This tool response reflects OPENDOG control/state data only."],
        "inferences": [],
        "confidence": "medium",
    });
    value["layers"]["constraints_boundaries"] = build_constraints_boundaries_layer(
        None,
        None,
        vec!["OPENDOG can report its own monitoring, snapshot, and activity-derived state."
            .to_string()],
        Vec::new(),
        vec![
            "Repository diff, test, and build state are not collected by this response unless stated explicitly.".to_string(),
            "Very brief file accesses may be missed because monitoring is sampling-based.".to_string(),
        ],
        default_shell_verification_commands(),
        None,
    );
    if let Some(shell) = when_to_use_shell {
        value["when_to_use_shell"] = json!(shell);
        value["layers"]["execution_strategy"]["when_to_use_shell"] = json!(shell);
    }
    value
}

pub(super) fn set_recommended_flow(value: &mut Value, steps: &[&str]) {
    value["recommended_flow"] = json!(steps);
    value["layers"]["execution_strategy"]["recommended_flow"] = json!(steps);
}

pub(super) fn default_shell_verification_commands() -> Vec<String> {
    vec![
        "git status".to_string(),
        "git diff".to_string(),
        "project-native tests and builds".to_string(),
    ]
}

pub(super) fn base_guidance_layers() -> Value {
    json!({
        "workspace_observation": {
            "status": "not_assessed",
        },
        "repo_status_risk": {
            "status": "not_collected",
            "summary": "Repository git/diff risk signals are not yet collected by this MCP response.",
        },
        "execution_strategy": {
            "status": "not_assessed",
        },
        "verification_evidence": {
            "status": "not_assessed",
        },
        "multi_project_portfolio": {
            "status": "not_assessed",
        },
        "storage_maintenance": {
            "status": "not_assessed",
        },
        "cleanup_refactor_candidates": {
            "status": "not_assessed",
            "candidates": [],
        },
        "project_toolchain": {
            "status": "not_assessed",
        },
        "constraints_boundaries": {
            "status": "not_assessed",
        },
        "governance": {
            "status": "not_assessed",
        },
    })
}

#[cfg(test)]
mod tests {
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
}
