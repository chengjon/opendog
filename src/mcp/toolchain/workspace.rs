use serde_json::{json, Value};
use std::collections::BTreeMap;

use super::{push_unique_command, toolchain_confidence_is_trusted};

pub(in crate::mcp) fn workspace_toolchain_layer(project_overviews: &[Value]) -> Value {
    let mut project_type_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut low_confidence_projects = Vec::new();
    let mut recommended_test_commands = Vec::new();
    let mut recommended_lint_commands = Vec::new();
    let mut recommended_build_commands = Vec::new();
    let mut projects_with_test_commands = 0_u64;
    let mut projects_with_lint_commands = 0_u64;
    let mut projects_with_build_commands = 0_u64;
    let mut projects_without_detected_toolchain = 0_u64;

    for overview in project_overviews {
        let project_id = overview["project_id"].as_str().unwrap_or("-");
        let toolchain = &overview["project_toolchain"];
        let project_type = toolchain["project_type"].as_str().unwrap_or("unknown");
        let confidence = toolchain["confidence"].as_str().unwrap_or("low");

        *project_type_counts
            .entry(project_type.to_string())
            .or_insert(0) += 1;

        if project_type == "unknown" {
            projects_without_detected_toolchain += 1;
        }
        if !toolchain_confidence_is_trusted(confidence) || project_type == "unknown" {
            low_confidence_projects.push(json!({
                "project_id": project_id,
                "project_type": project_type,
                "confidence": confidence,
            }));
        }

        if let Some(commands) = toolchain["recommended_test_commands"].as_array() {
            if !commands.is_empty() {
                projects_with_test_commands += 1;
            }
            for command in commands.iter().filter_map(Value::as_str) {
                push_unique_command(&mut recommended_test_commands, command);
            }
        }
        if let Some(commands) = toolchain["recommended_lint_commands"].as_array() {
            if !commands.is_empty() {
                projects_with_lint_commands += 1;
            }
            for command in commands.iter().filter_map(Value::as_str) {
                push_unique_command(&mut recommended_lint_commands, command);
            }
        }
        if let Some(commands) = toolchain["recommended_build_commands"].as_array() {
            if !commands.is_empty() {
                projects_with_build_commands += 1;
            }
            for command in commands.iter().filter_map(Value::as_str) {
                push_unique_command(&mut recommended_build_commands, command);
            }
        }
    }

    let known_project_types = project_type_counts
        .keys()
        .filter(|project_type| project_type.as_str() != "unknown")
        .count();
    let summary = if project_overviews.is_empty() {
        "No registered projects are available for workspace-level toolchain detection.".to_string()
    } else {
        format!(
            "Detected {} known toolchain type(s) across {} project(s); {} project(s) still need low-confidence or unknown-toolchain review.",
            known_project_types,
            project_overviews.len(),
            low_confidence_projects.len()
        )
    };

    json!({
        "status": "available",
        "summary": summary,
        "project_type_counts": project_type_counts,
        "known_project_types": known_project_types,
        "projects_without_detected_toolchain": projects_without_detected_toolchain,
        "projects_with_test_commands": projects_with_test_commands,
        "projects_with_lint_commands": projects_with_lint_commands,
        "projects_with_build_commands": projects_with_build_commands,
        "recommended_test_commands": recommended_test_commands,
        "recommended_lint_commands": recommended_lint_commands,
        "recommended_build_commands": recommended_build_commands,
        "low_confidence_projects": low_confidence_projects,
    })
}
