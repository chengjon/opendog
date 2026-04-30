use super::*;

#[test]
fn detect_project_commands_prefers_rust_workspace_commands() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\n",
    )
    .unwrap();

    let commands = detect_project_commands(dir.path());
    assert_eq!(commands[0], "cargo test");
    assert!(commands
        .iter()
        .any(|c| c == "cargo clippy --all-targets --all-features -- -D warnings"));
}

#[test]
fn detect_project_commands_prefers_node_when_package_json_exists() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("package.json"), "{\"name\":\"demo\"}").unwrap();

    let commands = detect_project_commands(dir.path());
    assert_eq!(commands[0], "npm test");
    assert!(commands.iter().any(|c| c == "npm run lint"));
}
