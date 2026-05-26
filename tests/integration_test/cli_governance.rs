use serde_json::Value;
use std::fs;
use tempfile::TempDir;

use super::common::run_cli;

#[test]
fn test_cli_governance_lifecycle() {
    let dir = TempDir::new().unwrap();
    let home = dir.path();
    let project_dir = dir.path().join("gov-project");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(project_dir.join("src/main.rs"), "fn main() {}").unwrap();

    let create = run_cli(
        home,
        &[
            "create",
            "--id",
            "gov-test",
            "--path",
            project_dir.to_str().unwrap(),
        ],
    );
    assert!(create.status.success(), "register failed: {:?}", create);

    let snapshot = run_cli(home, &["snapshot", "--id", "gov-test"]);
    assert!(snapshot.status.success(), "snapshot failed: {:?}", snapshot);

    // Create lane
    let create_lane = run_cli(
        home,
        &[
            "governance",
            "create-lane",
            "--id",
            "gov-test",
            "--lane-id",
            "di-remediation",
            "--title",
            "DI Remediation",
            "--description",
            "Extract singletons",
            "--json",
        ],
    );
    assert!(
        create_lane.status.success(),
        "create-lane failed: {:?}",
        create_lane
    );
    let lane_json: Value = serde_json::from_slice(&create_lane.stdout).unwrap();
    assert_eq!(
        lane_json["schema_version"],
        "opendog.cli.create-governance-lane.v1"
    );

    // Upsert node (create)
    let upsert = run_cli(
        home,
        &[
            "governance",
            "upsert-node",
            "--id",
            "gov-test",
            "--lane-id",
            "di-remediation",
            "--node-id",
            "G2.46",
            "--state",
            "evidence-prepared",
            "--summary",
            "Found 8 candidates",
            "--reported-git-head",
            "abc1234",
            "--json",
        ],
    );
    assert!(upsert.status.success(), "upsert-node failed: {:?}", upsert);
    let node_json: Value = serde_json::from_slice(&upsert.stdout).unwrap();
    assert_eq!(node_json["result"]["created"], true);

    // Show state
    let show = run_cli(home, &["governance", "show", "--id", "gov-test", "--json"]);
    assert!(show.status.success(), "show failed: {:?}", show);
    let state_json: Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(
        state_json["governance"]["lanes"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        state_json["governance"]["nodes"].as_array().unwrap().len(),
        1
    );

    // Close lane (complete)
    let close = run_cli(
        home,
        &[
            "governance",
            "close-lane",
            "--id",
            "gov-test",
            "--lane-id",
            "di-remediation",
            "--action",
            "complete",
            "--json",
        ],
    );
    assert!(close.status.success(), "close-lane failed: {:?}", close);
}

#[test]
fn test_cli_governance_upsert_rejects_missing_state() {
    let dir = TempDir::new().unwrap();
    let home = dir.path();
    let project_dir = dir.path().join("gov-project2");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(project_dir.join("src/main.rs"), "fn main() {}").unwrap();

    let _ = run_cli(
        home,
        &[
            "create",
            "--id",
            "gov-test2",
            "--path",
            project_dir.to_str().unwrap(),
        ],
    );
    let _ = run_cli(home, &["snapshot", "--id", "gov-test2"]);
    let _ = run_cli(
        home,
        &[
            "governance",
            "create-lane",
            "--id",
            "gov-test2",
            "--lane-id",
            "lane-1",
            "--title",
            "Test",
        ],
    );

    let upsert = run_cli(
        home,
        &[
            "governance",
            "upsert-node",
            "--id",
            "gov-test2",
            "--lane-id",
            "lane-1",
            "--node-id",
            "N1",
        ],
    );
    assert!(!upsert.status.success(), "should have failed without state");
}
