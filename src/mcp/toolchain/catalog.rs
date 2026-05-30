use std::path::Path;

use super::markers::{
    cargo_toml_has_workspace, manifest_backed_stack_markers, node_workspace_has_manifest_context,
    workspace_signal_present,
};
use super::{extend_unique_commands, ProjectToolchainProfile};

pub(super) fn single_stack_profile(stack: &str) -> Option<ProjectToolchainProfile> {
    match stack {
        "rust" => Some(ProjectToolchainProfile {
            project_type: "rust".to_string(),
            confidence: "high",
            test_commands: vec!["cargo test".to_string()],
            lint_commands: vec![
                "cargo clippy --all-targets --all-features -- -D warnings".to_string()
            ],
            build_commands: vec!["cargo check".to_string()],
            search_commands: vec!["rg \"<pattern>\" .".to_string()],
        }),
        "node" => Some(ProjectToolchainProfile {
            project_type: "node".to_string(),
            confidence: "high",
            test_commands: vec!["npm test".to_string()],
            lint_commands: vec!["npm run lint".to_string()],
            build_commands: vec!["npm run build".to_string()],
            search_commands: vec!["rg \"<pattern>\" .".to_string()],
        }),
        "python" => Some(ProjectToolchainProfile {
            project_type: "python".to_string(),
            confidence: "high",
            test_commands: vec!["pytest".to_string(), "python -m pytest".to_string()],
            lint_commands: vec!["ruff check .".to_string()],
            build_commands: vec![],
            search_commands: vec!["rg \"<pattern>\" .".to_string()],
        }),
        "go" => Some(ProjectToolchainProfile {
            project_type: "go".to_string(),
            confidence: "high",
            test_commands: vec!["go test ./...".to_string()],
            lint_commands: vec!["go vet ./...".to_string()],
            build_commands: vec!["go build ./...".to_string()],
            search_commands: vec!["rg \"<pattern>\" .".to_string()],
        }),
        _ => None,
    }
}

pub(super) fn mixed_workspace_confidence(root: &Path, stacks: &[&'static str]) -> &'static str {
    if stacks.len() > 1
        && workspace_signal_present(root)
        && manifest_backed_stack_markers(root).len() > 1
    {
        "medium-high"
    } else {
        "medium"
    }
}

pub(super) fn mixed_workspace_profile(
    root: &Path,
    stacks: &[&'static str],
) -> ProjectToolchainProfile {
    let mut test_commands = Vec::new();
    let mut lint_commands = Vec::new();
    let mut build_commands = Vec::new();
    let mut search_commands = vec!["rg \"<pattern>\" .".to_string()];

    for stack in stacks {
        if let Some(profile) = single_stack_profile(stack) {
            extend_unique_commands(&mut test_commands, &profile.test_commands);
            extend_unique_commands(&mut lint_commands, &profile.lint_commands);
            extend_unique_commands(&mut build_commands, &profile.build_commands);
            extend_unique_commands(&mut search_commands, &profile.search_commands);
        }
    }

    ProjectToolchainProfile {
        project_type: "mixed_workspace".to_string(),
        confidence: mixed_workspace_confidence(root, stacks),
        test_commands,
        lint_commands,
        build_commands,
        search_commands,
    }
}

fn generic_mono_repo_confidence(root: &Path) -> &'static str {
    if manifest_backed_stack_markers(root).is_empty() {
        "low"
    } else {
        "medium"
    }
}

pub(super) fn mono_repo_profile(root: &Path, stacks: &[&'static str]) -> ProjectToolchainProfile {
    if stacks.len() == 1 {
        if stacks[0] == "rust" && cargo_toml_has_workspace(root) {
            return ProjectToolchainProfile {
                project_type: "mono_repo".to_string(),
                confidence: "high",
                test_commands: vec!["cargo test --workspace".to_string()],
                lint_commands: vec![
                    "cargo clippy --workspace --all-targets --all-features -- -D warnings"
                        .to_string(),
                ],
                build_commands: vec!["cargo check --workspace".to_string()],
                search_commands: vec!["rg \"<pattern>\" .".to_string()],
            };
        }
        if stacks[0] == "node" && node_workspace_has_manifest_context(root) {
            return ProjectToolchainProfile {
                project_type: "mono_repo".to_string(),
                confidence: "high",
                test_commands: vec!["npm test --workspaces".to_string()],
                lint_commands: vec!["npm run lint --workspaces".to_string()],
                build_commands: vec!["npm run build --workspaces".to_string()],
                search_commands: vec!["rg \"<pattern>\" .".to_string()],
            };
        }
    }

    ProjectToolchainProfile {
        project_type: "mono_repo".to_string(),
        confidence: generic_mono_repo_confidence(root),
        test_commands: vec![],
        lint_commands: vec![],
        build_commands: vec![],
        search_commands: vec![
            "rg \"<pattern>\" .".to_string(),
            "git diff".to_string(),
            "git status".to_string(),
        ],
    }
}

pub(super) fn unknown_profile() -> ProjectToolchainProfile {
    ProjectToolchainProfile {
        project_type: "unknown".to_string(),
        confidence: "low",
        test_commands: vec![],
        lint_commands: vec![],
        build_commands: vec![],
        search_commands: vec![
            "rg \"<pattern>\" .".to_string(),
            "git diff".to_string(),
            "git status".to_string(),
        ],
    }
}

pub(super) fn docs_only_profile() -> ProjectToolchainProfile {
    ProjectToolchainProfile {
        project_type: "docs_only".to_string(),
        confidence: "medium-high",
        test_commands: vec![],
        lint_commands: vec![],
        build_commands: vec![],
        search_commands: vec!["rg \"<pattern>\" docs README.md".to_string()],
    }
}
