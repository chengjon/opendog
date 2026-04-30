use std::fs;

use tempfile::TempDir;

use super::run_cli;

#[test]
fn test_cli_decision_brief_text_includes_verification_and_toolchain_summary() {
    let dir = TempDir::new().unwrap();
    let home = dir.path();

    let project_dir = dir.path().join("rust-project");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(
        project_dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(project_dir.join("src/main.rs"), "fn main() {}").unwrap();
    let git_init = std::process::Command::new("git")
        .current_dir(&project_dir)
        .args(["init"])
        .output()
        .unwrap();
    assert!(git_init.status.success(), "{:?}", git_init);

    let create = run_cli(
        home,
        &[
            "create",
            "--id",
            "demo",
            "--path",
            project_dir.to_str().unwrap(),
        ],
    );
    assert!(create.status.success(), "{:?}", create);

    assert!(run_cli(home, &["snapshot", "--id", "demo"])
        .status
        .success());

    let record_verification = run_cli(
        home,
        &[
            "record-verification",
            "--id",
            "demo",
            "--kind",
            "test",
            "--status",
            "failed",
            "--command",
            "cargo test",
        ],
    );
    assert!(
        record_verification.status.success(),
        "{:?}",
        record_verification
    );

    let brief = run_cli(home, &["decision-brief", "--project", "demo", "--top", "1"]);
    assert!(brief.status.success(), "{:?}", brief);
    let stdout = String::from_utf8_lossy(&brief.stdout);
    assert!(stdout.contains("Workspace observation:"), "{stdout}");
    assert!(stdout.contains("snapshot_missing=0"), "{stdout}");
    assert!(stdout.contains("Verification evidence:"), "{stdout}");
    assert!(
        stdout.contains("Target observation: project=demo"),
        "{stdout}"
    );
    assert!(
        stdout.contains("coverage=snapshot_without_activity"),
        "{stdout}"
    );
    assert!(stdout.contains("Toolchain:"), "{stdout}");
    assert!(stdout.contains("Repo findings:"), "{stdout}");
    assert!(stdout.contains("Repo hotspots:"), "{stdout}");
}
