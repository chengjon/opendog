use opendog::contracts::{
    CLI_ACTIVITY_ROLLUPS_V1, CLI_CLEANUP_PROJECT_DATA_V1, CLI_SNAPSHOT_COMPARE_V1,
    CLI_TIME_WINDOW_REPORT_V1, CLI_USAGE_TRENDS_V1,
};
use opendog::storage::database::Database;
use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::common::run_cli;

pub fn assert_report_and_cleanup_flow(home: &Path, project_dir: &Path) {
    let project_db = home.join(".opendog/data/projects/demo.db");
    let report_db = Database::open_project(&project_db).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    report_db
        .execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["main.rs", "codex", 211i64, (now - 60).to_string()],
        )
        .unwrap();
    report_db
        .execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["main.rs", "codex", 211i64, (now - 20).to_string()],
        )
        .unwrap();
    report_db
        .execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["main.rs", "codex", 211i64, (now - 86_400 - 20).to_string()],
        )
        .unwrap();
    report_db
        .execute(
            "INSERT INTO file_events (file_path, event_type, event_time) VALUES (?1, 'modify', ?2)",
            rusqlite::params!["main.rs", (now - 10).to_string()],
        )
        .unwrap();

    let window_report = run_cli(
        home,
        &[
            "report", "window", "--id", "demo", "--window", "24h", "--json",
        ],
    );
    assert!(window_report.status.success(), "{:?}", window_report);
    let window_json: Value = serde_json::from_slice(&window_report.stdout).unwrap();
    assert_eq!(
        window_json["schema_version"].as_str(),
        Some(CLI_TIME_WINDOW_REPORT_V1)
    );
    assert_eq!(window_json["summary"]["total_sightings"].as_u64(), Some(2));

    fs::write(
        project_dir.join("main.rs"),
        "fn main() { println!(\"updated\"); }",
    )
    .unwrap();
    fs::write(project_dir.join("lib.rs"), "pub fn helper() {}").unwrap();
    let second_snapshot = run_cli(home, &["snapshot", "--id", "demo"]);
    assert!(second_snapshot.status.success(), "{:?}", second_snapshot);

    let compare_report = run_cli(home, &["report", "compare", "--id", "demo", "--json"]);
    assert!(compare_report.status.success(), "{:?}", compare_report);
    let compare_json: Value = serde_json::from_slice(&compare_report.stdout).unwrap();
    assert_eq!(
        compare_json["schema_version"].as_str(),
        Some(CLI_SNAPSHOT_COMPARE_V1)
    );
    assert_eq!(compare_json["summary"]["added_files"].as_u64(), Some(1));
    assert_eq!(compare_json["summary"]["modified_files"].as_u64(), Some(1));

    let trend_report = run_cli(
        home,
        &[
            "report", "trend", "--id", "demo", "--window", "7d", "--json",
        ],
    );
    assert!(trend_report.status.success(), "{:?}", trend_report);
    let trend_json: Value = serde_json::from_slice(&trend_report.stdout).unwrap();
    assert_eq!(
        trend_json["schema_version"].as_str(),
        Some(CLI_USAGE_TRENDS_V1)
    );
    assert_eq!(trend_json["window"].as_str(), Some("7d"));

    let cleanup_execute = run_cli(
        home,
        &[
            "cleanup-data",
            "--id",
            "demo",
            "--scope",
            "activity",
            "--older-than-days",
            "1",
            "--json",
        ],
    );
    assert!(cleanup_execute.status.success(), "{:?}", cleanup_execute);

    let rollup_report = run_cli(
        home,
        &[
            "report", "rollup", "--id", "demo", "--window", "7d", "--json",
        ],
    );
    assert!(rollup_report.status.success(), "{:?}", rollup_report);
    let rollup_json: Value = serde_json::from_slice(&rollup_report.stdout).unwrap();
    assert_eq!(
        rollup_json["schema_version"].as_str(),
        Some(CLI_ACTIVITY_ROLLUPS_V1)
    );
    assert_eq!(rollup_json["window"].as_str(), Some("7d"));
    assert!(
        rollup_json["summary"]["total_access_count"]
            .as_u64()
            .unwrap_or(0)
            >= 1
    );

    let cleanup_preview = run_cli(
        home,
        &[
            "cleanup-data",
            "--id",
            "demo",
            "--scope",
            "activity",
            "--older-than-days",
            "1",
            "--dry-run",
            "--json",
        ],
    );
    assert!(cleanup_preview.status.success(), "{:?}", cleanup_preview);
    let cleanup_json: Value = serde_json::from_slice(&cleanup_preview.stdout).unwrap();
    assert_eq!(
        cleanup_json["schema_version"].as_str(),
        Some(CLI_CLEANUP_PROJECT_DATA_V1)
    );
    assert_eq!(cleanup_json["scope"].as_str(), Some("activity"));
    assert_eq!(cleanup_json["dry_run"].as_bool(), Some(true));
    assert_eq!(cleanup_json["vacuum"].as_bool(), Some(false));
    assert_eq!(
        cleanup_json["rolled_up"]["file_sightings"].as_i64(),
        Some(0)
    );
    assert_eq!(cleanup_json["rolled_up"]["file_events"].as_i64(), Some(0));
    assert!(
        cleanup_json["storage_before"]["page_count"]
            .as_u64()
            .is_some(),
        "expected storage_before.page_count in cleanup payload: {cleanup_json:#?}"
    );
    assert!(
        cleanup_json["storage_before"]["approx_db_size_bytes"]
            .as_u64()
            .is_some(),
        "expected storage_before.approx_db_size_bytes in cleanup payload: {cleanup_json:#?}"
    );
    assert_eq!(cleanup_json["storage_after"].as_object(), None);
    assert_eq!(
        cleanup_json["maintenance"]["optimized"].as_bool(),
        Some(false)
    );
    assert_eq!(
        cleanup_json["maintenance"]["vacuumed"].as_bool(),
        Some(false)
    );
}
