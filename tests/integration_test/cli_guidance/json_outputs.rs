use std::fs;

use tempfile::TempDir;

use super::run_cli;

#[path = "json_outputs/guidance_and_risk.rs"]
mod guidance_and_risk;
#[path = "json_outputs/verification.rs"]
mod verification;

#[test]
fn test_cli_json_outputs_for_guidance_and_data_risk() {
    let dir = TempDir::new().unwrap();
    let home = dir.path();

    let risky_project_dir = dir.path().join("risky-project");
    fs::create_dir_all(risky_project_dir.join("src")).unwrap();
    fs::create_dir_all(risky_project_dir.join("tests/fixtures")).unwrap();
    fs::write(
        risky_project_dir.join("src/customer_seed.json"),
        r#"{"customer":"Demo User","email":"demo@example.com","address":"1 Market Street","amount":"20 usd"}"#,
    )
    .unwrap();
    fs::write(
        risky_project_dir.join("tests/fixtures/mock_response.json"),
        r#"{"mock":true,"sample":"fixture"}"#,
    )
    .unwrap();

    let clean_project_dir = dir.path().join("clean-project");
    fs::create_dir_all(clean_project_dir.join("src")).unwrap();
    fs::write(clean_project_dir.join("src/main.rs"), "fn main() {}").unwrap();

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

    let snapshot_risky = run_cli(home, &["snapshot", "--id", "risky"]);
    assert!(snapshot_risky.status.success(), "{:?}", snapshot_risky);

    let snapshot_clean = run_cli(home, &["snapshot", "--id", "clean"]);
    assert!(snapshot_clean.status.success(), "{:?}", snapshot_clean);

    guidance_and_risk::assert_guidance_data_risk_and_decision_brief(home);
    verification::assert_verification_json_outputs(home);
}
