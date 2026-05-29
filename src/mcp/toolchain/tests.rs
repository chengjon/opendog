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
    assert!(profile.lint_commands.iter().any(|c| c.contains("clippy")));
    assert!(profile.build_commands.contains(&"cargo check".to_string()));
}

#[test]
fn single_stack_profile_node() {
    let profile = single_stack_profile("node").unwrap();
    assert_eq!(profile.project_type, "node");
    assert_eq!(profile.confidence, "high");
    assert!(profile.test_commands.contains(&"npm test".to_string()));
    assert!(profile.lint_commands.contains(&"npm run lint".to_string()));
    assert!(profile
        .build_commands
        .contains(&"npm run build".to_string()));
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
    assert!(profile
        .build_commands
        .contains(&"go build ./...".to_string()));
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
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace]\nmembers=[\"a\"]",
    )
    .unwrap();
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
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace]\nmembers=[\"a\"]",
    )
    .unwrap();
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
    assert!(result["low_confidence_projects"]
        .as_array()
        .unwrap()
        .is_empty());
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

#[test]
fn unknown_profile_returns_low_confidence_with_search_commands() {
    let profile = unknown_profile();
    assert_eq!(profile.project_type, "unknown");
    assert_eq!(profile.confidence, "low");
    assert!(profile.test_commands.is_empty());
    assert!(profile.lint_commands.is_empty());
    assert!(profile.build_commands.is_empty());
    assert_eq!(profile.search_commands.len(), 3);
    assert_eq!(profile.search_commands[0], "rg \"<pattern>\" .");
}

#[test]
fn docs_only_profile_returns_medium_high_confidence_with_docs_search() {
    let profile = docs_only_profile();
    assert_eq!(profile.project_type, "docs_only");
    assert_eq!(profile.confidence, "medium-high");
    assert!(profile.test_commands.is_empty());
    assert!(profile.lint_commands.is_empty());
    assert!(profile.build_commands.is_empty());
    assert_eq!(profile.search_commands.len(), 1);
    assert_eq!(
        profile.search_commands[0],
        "rg \"<pattern>\" docs README.md"
    );
}
