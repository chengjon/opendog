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
    assert_eq!(docs_value["confidence"], json!("medium"));
}
