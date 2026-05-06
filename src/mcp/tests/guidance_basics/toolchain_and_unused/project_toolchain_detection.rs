use super::*;

#[test]
fn project_toolchain_layer_detects_additional_project_shapes() {
    let go_dir = TempDir::new().unwrap();
    std::fs::write(
        go_dir.path().join("go.mod"),
        "module example.com/demo\n\ngo 1.22\n",
    )
    .unwrap();
    let go_value = project_toolchain_layer(go_dir.path());
    assert_eq!(go_value["project_type"], json!("go"));
    assert_eq!(go_value["confidence"], json!("high"));
    assert!(go_value["recommended_test_commands"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "go test ./..."));

    let monorepo_dir = TempDir::new().unwrap();
    std::fs::write(
        monorepo_dir.path().join("package.json"),
        r#"{"name":"workspace","private":true,"workspaces":["apps/*","packages/*"]}"#,
    )
    .unwrap();
    let monorepo_value = project_toolchain_layer(monorepo_dir.path());
    assert_eq!(monorepo_value["project_type"], json!("mono_repo"));
    assert_eq!(monorepo_value["confidence"], json!("high"));

    let mixed_dir = TempDir::new().unwrap();
    std::fs::write(
        mixed_dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(
        mixed_dir.path().join("package.json"),
        r#"{"name":"demo","version":"1.0.0"}"#,
    )
    .unwrap();
    let mixed_value = project_toolchain_layer(mixed_dir.path());
    assert_eq!(mixed_value["project_type"], json!("mixed_workspace"));
    assert!(mixed_value["recommended_test_commands"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "cargo test"));
    assert!(mixed_value["recommended_test_commands"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "npm test"));

    let docs_dir = TempDir::new().unwrap();
    std::fs::create_dir(docs_dir.path().join("docs")).unwrap();
    std::fs::write(docs_dir.path().join("mkdocs.yml"), "site_name: Demo\n").unwrap();
    std::fs::write(docs_dir.path().join("docs/index.md"), "# Demo\n").unwrap();
    let docs_value = project_toolchain_layer(docs_dir.path());
    assert_eq!(docs_value["project_type"], json!("docs_only"));
    assert_eq!(docs_value["confidence"], json!("medium-high"));
}

#[test]
fn mixed_workspace_without_workspace_corroboration_stays_medium() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("package.json"),
        r#"{"name":"demo","version":"1.0.0"}"#,
    )
    .unwrap();

    let value = project_toolchain_layer(dir.path());
    assert_eq!(value["project_type"], json!("mixed_workspace"));
    assert_eq!(value["confidence"], json!("medium"));
}

#[test]
fn mixed_workspace_with_workspace_corroboration_becomes_medium_high() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("package.json"),
        r#"{"name":"demo","private":true,"workspaces":["apps/*"]}"#,
    )
    .unwrap();

    let value = project_toolchain_layer(dir.path());
    assert_eq!(value["project_type"], json!("mixed_workspace"));
    assert_eq!(value["confidence"], json!("medium-high"));
}

#[test]
fn mono_repo_high_confidence_workspace_paths_remain_high() {
    let rust_dir = TempDir::new().unwrap();
    std::fs::write(
        rust_dir.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .unwrap();

    let rust_value = project_toolchain_layer(rust_dir.path());
    assert_eq!(rust_value["project_type"], json!("mono_repo"));
    assert_eq!(rust_value["confidence"], json!("high"));

    let node_dir = TempDir::new().unwrap();
    std::fs::write(
        node_dir.path().join("package.json"),
        r#"{"name":"workspace","private":true,"workspaces":["apps/*","packages/*"]}"#,
    )
    .unwrap();

    let node_value = project_toolchain_layer(node_dir.path());
    assert_eq!(node_value["project_type"], json!("mono_repo"));
    assert_eq!(node_value["confidence"], json!("high"));
}

#[test]
fn generic_mono_repo_with_only_workspace_marker_becomes_low_confidence() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("pnpm-workspace.yaml"),
        "packages:\n  - apps/*\n",
    )
    .unwrap();

    let value = project_toolchain_layer(dir.path());
    assert_eq!(value["project_type"], json!("mono_repo"));
    assert_eq!(value["confidence"], json!("low"));
}

#[test]
fn docs_only_profile_moves_into_medium_high_confidence() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join("docs")).unwrap();
    std::fs::write(dir.path().join("mkdocs.yml"), "site_name: Demo\n").unwrap();
    std::fs::write(dir.path().join("docs/index.md"), "# Demo\n").unwrap();

    let value = project_toolchain_layer(dir.path());
    assert_eq!(value["project_type"], json!("docs_only"));
    assert_eq!(value["confidence"], json!("medium-high"));
    assert_eq!(value["recommended_test_commands"], json!([]));
    assert_eq!(value["recommended_lint_commands"], json!([]));
    assert_eq!(value["recommended_build_commands"], json!([]));
    assert_eq!(
        value["recommended_search_commands"],
        json!(["rg \"<pattern>\" docs README.md"])
    );
}

#[test]
fn unknown_profile_stays_low_with_current_fallback_commands() {
    let dir = TempDir::new().unwrap();

    let value = project_toolchain_layer(dir.path());
    assert_eq!(value["project_type"], json!("unknown"));
    assert_eq!(value["confidence"], json!("low"));
    assert_eq!(value["recommended_test_commands"], json!([]));
    assert_eq!(value["recommended_lint_commands"], json!([]));
    assert_eq!(value["recommended_build_commands"], json!([]));
    assert_eq!(
        value["recommended_search_commands"],
        json!(["rg \"<pattern>\" .", "git diff", "git status"])
    );
}
