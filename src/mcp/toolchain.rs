use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

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

fn file_exists(root: &Path, name: &str) -> bool {
    root.join(name).exists()
}

fn read_project_file(root: &Path, name: &str) -> Option<String> {
    fs::read_to_string(root.join(name)).ok()
}

fn package_json_has_workspaces(root: &Path) -> bool {
    read_project_file(root, "package.json")
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|value| value.get("workspaces").cloned())
        .map(|workspaces| match workspaces {
            Value::Array(items) => !items.is_empty(),
            Value::Object(fields) => !fields.is_empty(),
            _ => false,
        })
        .unwrap_or(false)
}

fn cargo_toml_has_workspace(root: &Path) -> bool {
    read_project_file(root, "Cargo.toml")
        .map(|text| text.contains("[workspace]"))
        .unwrap_or(false)
}

fn node_workspace_marker_exists(root: &Path) -> bool {
    file_exists(root, "pnpm-workspace.yaml")
        || file_exists(root, "lerna.json")
        || file_exists(root, "nx.json")
        || file_exists(root, "turbo.json")
        || package_json_has_workspaces(root)
}

fn docs_only_marker_exists(root: &Path) -> bool {
    let has_docs_config = file_exists(root, "mkdocs.yml")
        || file_exists(root, "mkdocs.yaml")
        || file_exists(root, "docusaurus.config.js")
        || file_exists(root, "docusaurus.config.ts");
    let has_docs_content = file_exists(root, "README.md")
        || file_exists(root, "docs/index.md")
        || root.join("docs").is_dir();
    has_docs_config && has_docs_content
}

fn workspace_signal_present(root: &Path) -> bool {
    cargo_toml_has_workspace(root)
        || node_workspace_marker_exists(root)
        || file_exists(root, "go.work")
}

fn detected_stack_markers(root: &Path) -> Vec<&'static str> {
    let mut markers = Vec::new();
    if file_exists(root, "Cargo.toml") {
        markers.push("rust");
    }
    if file_exists(root, "package.json") || node_workspace_marker_exists(root) {
        markers.push("node");
    }
    if file_exists(root, "pyproject.toml")
        || file_exists(root, "requirements.txt")
        || file_exists(root, "pytest.ini")
        || file_exists(root, "Pipfile")
    {
        markers.push("python");
    }
    if file_exists(root, "go.mod") || file_exists(root, "go.work") {
        markers.push("go");
    }
    markers
}

fn manifest_backed_stack_markers(root: &Path) -> Vec<&'static str> {
    let mut markers = Vec::new();
    if file_exists(root, "Cargo.toml") {
        markers.push("rust");
    }
    if file_exists(root, "package.json") {
        markers.push("node");
    }
    if file_exists(root, "pyproject.toml")
        || file_exists(root, "requirements.txt")
        || file_exists(root, "pytest.ini")
        || file_exists(root, "Pipfile")
    {
        markers.push("python");
    }
    if file_exists(root, "go.mod") || file_exists(root, "go.work") {
        markers.push("go");
    }
    markers
}

fn node_workspace_has_manifest_context(root: &Path) -> bool {
    file_exists(root, "package.json") && node_workspace_marker_exists(root)
}

fn single_stack_profile(stack: &str) -> Option<ProjectToolchainProfile> {
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

fn mixed_workspace_confidence(root: &Path, stacks: &[&'static str]) -> &'static str {
    if stacks.len() > 1
        && workspace_signal_present(root)
        && manifest_backed_stack_markers(root).len() > 1
    {
        "medium-high"
    } else {
        "medium"
    }
}

fn mixed_workspace_profile(root: &Path, stacks: &[&'static str]) -> ProjectToolchainProfile {
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

fn mono_repo_profile(root: &Path, stacks: &[&'static str]) -> ProjectToolchainProfile {
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

fn unknown_profile() -> ProjectToolchainProfile {
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

fn docs_only_profile() -> ProjectToolchainProfile {
    ProjectToolchainProfile {
        project_type: "docs_only".to_string(),
        confidence: "medium-high",
        test_commands: vec![],
        lint_commands: vec![],
        build_commands: vec![],
        search_commands: vec!["rg \"<pattern>\" docs README.md".to_string()],
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

pub(super) fn workspace_toolchain_layer(project_overviews: &[Value]) -> Value {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- push_unique_command ---

    #[test]
    fn push_unique_command_adds_new_item() {
        let mut target = vec!["cargo test".to_string()];
        push_unique_command(&mut target, "cargo clippy");
        assert_eq!(target, vec!["cargo test", "cargo clippy"]);
    }

    #[test]
    fn push_unique_command_skips_exact_duplicate() {
        let mut target = vec!["cargo test".to_string()];
        push_unique_command(&mut target, "cargo test");
        assert_eq!(target, vec!["cargo test"]);
    }

    #[test]
    fn push_unique_command_empty_target_adds() {
        let mut target: Vec<String> = Vec::new();
        push_unique_command(&mut target, "npm test");
        assert_eq!(target, vec!["npm test"]);
    }

    // --- extend_unique_commands ---

    #[test]
    fn extend_unique_commands_adds_all_new() {
        let mut target: Vec<String> = Vec::new();
        let commands = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        extend_unique_commands(&mut target, &commands);
        assert_eq!(target, vec!["a", "b", "c"]);
    }

    #[test]
    fn extend_unique_commands_dedupes_against_existing() {
        let mut target = vec!["a".to_string(), "b".to_string()];
        let commands = vec!["b".to_string(), "c".to_string(), "a".to_string()];
        extend_unique_commands(&mut target, &commands);
        assert_eq!(target, vec!["a", "b", "c"]);
    }

    #[test]
    fn extend_unique_commands_dedupes_within_batch() {
        let mut target: Vec<String> = Vec::new();
        let commands = vec!["x".to_string(), "x".to_string(), "y".to_string()];
        extend_unique_commands(&mut target, &commands);
        assert_eq!(target, vec!["x", "y"]);
    }

    #[test]
    fn extend_unique_commands_empty_batch_no_change() {
        let mut target = vec!["a".to_string()];
        let commands: Vec<String> = Vec::new();
        extend_unique_commands(&mut target, &commands);
        assert_eq!(target, vec!["a"]);
    }

    // --- single_stack_profile ---

    #[test]
    fn single_stack_profile_rust() {
        let profile = single_stack_profile("rust").unwrap();
        assert_eq!(profile.project_type, "rust");
        assert_eq!(profile.confidence, "high");
        assert!(profile.test_commands.contains(&"cargo test".to_string()));
        assert!(profile
            .lint_commands
            .iter()
            .any(|c| c.contains("clippy")));
        assert!(profile.build_commands.contains(&"cargo check".to_string()));
    }

    #[test]
    fn single_stack_profile_node() {
        let profile = single_stack_profile("node").unwrap();
        assert_eq!(profile.project_type, "node");
        assert_eq!(profile.confidence, "high");
        assert!(profile.test_commands.contains(&"npm test".to_string()));
        assert!(profile.lint_commands.contains(&"npm run lint".to_string()));
        assert!(profile.build_commands.contains(&"npm run build".to_string()));
    }

    #[test]
    fn single_stack_profile_python() {
        let profile = single_stack_profile("python").unwrap();
        assert_eq!(profile.project_type, "python");
        assert_eq!(profile.confidence, "high");
        assert!(profile.test_commands.contains(&"pytest".to_string()));
        assert!(profile.lint_commands.contains(&"ruff check .".to_string()));
        assert!(profile.build_commands.is_empty());
    }

    #[test]
    fn single_stack_profile_go() {
        let profile = single_stack_profile("go").unwrap();
        assert_eq!(profile.project_type, "go");
        assert_eq!(profile.confidence, "high");
        assert!(profile.test_commands.contains(&"go test ./...".to_string()));
        assert!(profile.lint_commands.contains(&"go vet ./...".to_string()));
        assert!(profile.build_commands.contains(&"go build ./...".to_string()));
    }

    #[test]
    fn single_stack_profile_unknown_returns_none() {
        assert!(single_stack_profile("fortran").is_none());
        assert!(single_stack_profile("").is_none());
        assert!(single_stack_profile("RUST").is_none());
    }

    // --- all_recommended_commands (through single_stack_profile) ---

    #[test]
    fn all_recommended_commands_merges_all_categories() {
        let profile = single_stack_profile("rust").unwrap();
        let all = profile.all_recommended_commands();
        assert!(all.contains(&"cargo test".to_string()));
        assert!(all.iter().any(|c| c.contains("clippy")));
        assert!(all.contains(&"cargo check".to_string()));
        assert!(all.iter().any(|c| c.contains("rg")));
    }

    // --- mixed_workspace_confidence ---

    #[test]
    fn mixed_workspace_confidence_medium_when_no_workspace_signal() {
        let dir = tempfile::tempdir().unwrap();
        // No Cargo.toml, no workspace markers
        let confidence = mixed_workspace_confidence(dir.path(), &["rust", "node"]);
        assert_eq!(confidence, "medium");
    }

    #[test]
    fn mixed_workspace_confidence_medium_when_single_stack() {
        let dir = tempfile::tempdir().unwrap();
        let confidence = mixed_workspace_confidence(dir.path(), &["rust"]);
        assert_eq!(confidence, "medium");
    }

    #[test]
    fn mixed_workspace_confidence_medium_high_with_workspace_and_manifests() {
        let dir = tempfile::tempdir().unwrap();
        // Create both Cargo.toml with [workspace] and package.json with workspaces
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers=[\"a\"]").unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"workspaces": ["packages/*"]}"#,
        )
        .unwrap();
        let confidence = mixed_workspace_confidence(dir.path(), &["rust", "node"]);
        assert_eq!(confidence, "medium-high");
    }

    #[test]
    fn mixed_workspace_confidence_medium_when_workspace_but_only_one_manifest() {
        let dir = tempfile::tempdir().unwrap();
        // Cargo.toml with [workspace] but no package.json
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers=[\"a\"]").unwrap();
        let confidence = mixed_workspace_confidence(dir.path(), &["rust"]);
        assert_eq!(confidence, "medium");
    }

    // --- toolchain_confidence_is_trusted ---

    #[test]
    fn toolchain_confidence_is_trusted_high() {
        assert!(toolchain_confidence_is_trusted("high"));
    }

    #[test]
    fn toolchain_confidence_is_trusted_medium_high() {
        assert!(toolchain_confidence_is_trusted("medium-high"));
    }

    #[test]
    fn toolchain_confidence_is_trusted_medium() {
        assert!(!toolchain_confidence_is_trusted("medium"));
    }

    #[test]
    fn toolchain_confidence_is_trusted_low() {
        assert!(!toolchain_confidence_is_trusted("low"));
    }

    #[test]
    fn toolchain_confidence_is_trusted_empty() {
        assert!(!toolchain_confidence_is_trusted(""));
    }

    // --- workspace_toolchain_layer ---

    #[test]
    fn workspace_toolchain_layer_empty_projects() {
        let result = workspace_toolchain_layer(&[]);
        assert_eq!(result["status"], "available");
        assert_eq!(result["known_project_types"], 0);
        assert_eq!(result["projects_without_detected_toolchain"], 0);
        assert!(result["summary"]
            .as_str()
            .unwrap()
            .contains("No registered projects"));
    }

    #[test]
    fn workspace_toolchain_layer_with_single_rust_project() {
        let toolchain = json!({
            "project_type": "rust",
            "confidence": "high",
            "recommended_test_commands": ["cargo test"],
            "recommended_lint_commands": ["cargo clippy"],
            "recommended_build_commands": ["cargo check"],
        });
        let overview = json!({
            "project_id": "my-rust-project",
            "project_toolchain": toolchain,
        });
        let result = workspace_toolchain_layer(&[overview]);
        assert_eq!(result["status"], "available");
        assert_eq!(result["known_project_types"], 1);
        assert_eq!(result["project_type_counts"]["rust"], 1);
        assert_eq!(result["projects_with_test_commands"], 1);
        assert_eq!(result["projects_with_lint_commands"], 1);
        assert_eq!(result["projects_with_build_commands"], 1);
        assert!(result["low_confidence_projects"].as_array().unwrap().is_empty());
        assert!(result["summary"]
            .as_str()
            .unwrap()
            .contains("1 known toolchain type(s)"));
    }

    #[test]
    fn workspace_toolchain_layer_flags_unknown_and_low_confidence() {
        let overview = json!({
            "project_id": "weird-project",
            "project_toolchain": {
                "project_type": "unknown",
                "confidence": "low",
                "recommended_test_commands": [],
                "recommended_lint_commands": [],
                "recommended_build_commands": [],
            },
        });
        let result = workspace_toolchain_layer(&[overview]);
        assert_eq!(result["projects_without_detected_toolchain"], 1);
        assert_eq!(result["known_project_types"], 0);
        let low = result["low_confidence_projects"].as_array().unwrap();
        assert_eq!(low.len(), 1);
        assert_eq!(low[0]["project_id"], "weird-project");
    }

    #[test]
    fn workspace_toolchain_layer_dedupes_commands_across_projects() {
        let overview_a = json!({
            "project_id": "proj-a",
            "project_toolchain": {
                "project_type": "rust",
                "confidence": "high",
                "recommended_test_commands": ["cargo test"],
                "recommended_lint_commands": ["cargo clippy"],
                "recommended_build_commands": ["cargo check"],
            },
        });
        let overview_b = json!({
            "project_id": "proj-b",
            "project_toolchain": {
                "project_type": "rust",
                "confidence": "high",
                "recommended_test_commands": ["cargo test"],
                "recommended_lint_commands": ["cargo clippy"],
                "recommended_build_commands": ["cargo check"],
            },
        });
        let result = workspace_toolchain_layer(&[overview_a, overview_b]);
        let test_cmds = result["recommended_test_commands"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>();
        // "cargo test" should appear exactly once
        assert_eq!(test_cmds.iter().filter(|c| **c == "cargo test").count(), 1);
        assert_eq!(result["projects_with_test_commands"], 2);
    }

    // --- detect_project_profile via project_toolchain_layer ---

    #[test]
    fn project_toolchain_layer_unknown_for_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let result = project_toolchain_layer(dir.path());
        assert_eq!(result["status"], "available");
        assert_eq!(result["project_type"], "unknown");
        assert_eq!(result["confidence"], "low");
    }

    #[test]
    fn project_toolchain_layer_detects_rust() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"").unwrap();
        let result = project_toolchain_layer(dir.path());
        assert_eq!(result["status"], "available");
        assert_eq!(result["project_type"], "rust");
        assert_eq!(result["confidence"], "high");
        assert!(result["recommended_test_commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c.as_str().unwrap() == "cargo test"));
    }

    #[test]
    fn project_toolchain_layer_detects_docs_only() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("mkdocs.yml"), "site_name: test").unwrap();
        std::fs::write(dir.path().join("README.md"), "# Hello").unwrap();
        let result = project_toolchain_layer(dir.path());
        assert_eq!(result["project_type"], "docs_only");
        assert_eq!(result["confidence"], "medium-high");
    }
}
