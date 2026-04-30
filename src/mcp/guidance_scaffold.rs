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
    })
}
