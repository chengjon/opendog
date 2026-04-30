use std::fs;

use tempfile::TempDir;

use super::run_cli;

#[test]
fn test_cli_agent_guidance_text_includes_workspace_verification_summary() {
    let dir = TempDir::new().unwrap();
    let home = dir.path();

    let risky_project_dir = dir.path().join("risky-project");
    fs::create_dir_all(risky_project_dir.join("src")).unwrap();
    fs::write(risky_project_dir.join("src/main.rs"), "fn main() {}").unwrap();

    let clean_project_dir = dir.path().join("clean-project");
    fs::create_dir_all(clean_project_dir.join("src")).unwrap();
    fs::write(clean_project_dir.join("src/lib.rs"), "pub fn ok() {}").unwrap();

    let create_risky = run_cli(
        home,
        &[
            "create",
            "--id",
            "risky",
            "--path",
            risky_project_dir.to_str().unwrap(),
        ],
    );
    assert!(create_risky.status.success(), "{:?}", create_risky);

    let create_clean = run_cli(
        home,
        &[
            "create",
            "--id",
            "clean",
            "--path",
            clean_project_dir.to_str().unwrap(),
        ],
    );
    assert!(create_clean.status.success(), "{:?}", create_clean);

    assert!(run_cli(home, &["snapshot", "--id", "risky"])
        .status
        .success());
    assert!(run_cli(home, &["snapshot", "--id", "clean"])
        .status
        .success());

    let risky_verification = run_cli(
        home,
        &[
            "record-verification",
            "--id",
            "risky",
            "--kind",
            "test",
            "--status",
            "failed",
            "--command",
            "cargo test",
        ],
    );
    assert!(
        risky_verification.status.success(),
        "{:?}",
        risky_verification
    );

    let guidance = run_cli(home, &["agent-guidance", "--top", "3"]);
    assert!(guidance.status.success(), "{:?}", guidance);
    let stdout = String::from_utf8_lossy(&guidance.stdout);
    assert!(stdout.contains("Workspace observation:"), "{stdout}");
    assert!(stdout.contains("snapshot_missing=0"), "{stdout}");
    assert!(stdout.contains("verification_missing=1"), "{stdout}");
    assert!(stdout.contains("Verification evidence:"), "{stdout}");
    assert!(stdout.contains("recorded=1"), "{stdout}");
    assert!(stdout.contains("missing=1"), "{stdout}");
    assert!(stdout.contains("failing=1"), "{stdout}");
    assert!(stdout.contains("Blocking projects:"), "{stdout}");
    assert!(stdout.contains("coverage="), "{stdout}");
    assert!(stdout.contains("verification_status="), "{stdout}");
    assert!(stdout.contains("repo_status="), "{stdout}");
    assert!(stdout.contains("toolchain_type="), "{stdout}");
    assert!(stdout.contains("risky"), "{stdout}");
}
