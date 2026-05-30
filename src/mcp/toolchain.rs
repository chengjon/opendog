use serde_json::{json, Value};
use std::path::Path;

mod catalog;
mod markers;
mod workspace;

pub(super) use workspace::workspace_toolchain_layer;

#[cfg(test)]
use catalog::mixed_workspace_confidence;
use catalog::{
    docs_only_profile, mixed_workspace_profile, mono_repo_profile, single_stack_profile,
    unknown_profile,
};
use markers::{detected_stack_markers, docs_only_marker_exists, workspace_signal_present};

struct ProjectToolchainProfile {
    project_type: String,
    confidence: &'static str,
    test_commands: Vec<String>,
    lint_commands: Vec<String>,
    build_commands: Vec<String>,
    search_commands: Vec<String>,
}

impl ProjectToolchainProfile {
    fn all_recommended_commands(&self) -> Vec<String> {
        let mut commands = Vec::new();
        commands.extend(self.test_commands.clone());
        commands.extend(self.lint_commands.clone());
        commands.extend(self.build_commands.clone());
        commands.extend(self.search_commands.clone());
        commands
    }
}

fn push_unique_command(target: &mut Vec<String>, command: &str) {
    if !target.iter().any(|existing| existing == command) {
        target.push(command.to_string());
    }
}

fn extend_unique_commands(target: &mut Vec<String>, commands: &[String]) {
    for command in commands {
        push_unique_command(target, command);
    }
}

fn toolchain_confidence_is_trusted(confidence: &str) -> bool {
    matches!(confidence, "high" | "medium-high")
}

fn detect_project_profile(root: &Path) -> ProjectToolchainProfile {
    let stacks = detected_stack_markers(root);
    if stacks.len() > 1 {
        mixed_workspace_profile(root, &stacks)
    } else if workspace_signal_present(root) {
        mono_repo_profile(root, &stacks)
    } else if stacks.len() == 1 {
        single_stack_profile(stacks[0]).unwrap_or_else(unknown_profile)
    } else if docs_only_marker_exists(root) {
        docs_only_profile()
    } else {
        unknown_profile()
    }
}

pub(super) fn detect_project_commands(root: &Path) -> Vec<String> {
    let commands = detect_project_profile(root).all_recommended_commands();
    if commands.is_empty() {
        unknown_profile().all_recommended_commands()
    } else {
        commands
    }
}

pub(super) fn project_toolchain_layer(root: &Path) -> Value {
    let profile = detect_project_profile(root);
    json!({
        "status": "available",
        "project_type": profile.project_type,
        "confidence": profile.confidence,
        "recommended_test_commands": profile.test_commands,
        "recommended_lint_commands": profile.lint_commands,
        "recommended_build_commands": profile.build_commands,
        "recommended_search_commands": profile.search_commands,
    })
}

#[cfg(test)]
mod tests;
