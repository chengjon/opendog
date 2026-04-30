use opendog::contracts::{
    CLI_CLEANUP_PROJECT_DATA_V1, CLI_DECISION_BRIEF_V1, CLI_GLOBAL_CONFIG_V1,
    CLI_PROJECT_CONFIG_V1, CLI_RELOAD_PROJECT_CONFIG_V1, CLI_SNAPSHOT_COMPARE_V1,
    CLI_TIME_WINDOW_REPORT_V1, CLI_UPDATE_GLOBAL_CONFIG_V1, CLI_UPDATE_PROJECT_CONFIG_V1,
    CLI_USAGE_TRENDS_V1,
};
use opendog::storage::database::Database;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};
use tempfile::TempDir;

use super::common::run_cli;

fn wait_for_daemon_ready(home: &Path) {
    let socket = home.join(".opendog/data/daemon.sock");
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if socket.exists() {
            let output = run_cli(home, &["list"]);
            if output.status.success() {
                return;
            }
        }
        sleep(Duration::from_millis(100));
    }
    panic!("daemon socket did not become ready: {}", socket.display());
}

fn terminate_daemon(child: &mut Child) {
    let pid = child.id().to_string();
    let _ = Command::new("kill").args(["-TERM", &pid]).status();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(Some(_)) = child.try_wait() {
            return;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return;
        }
        sleep(Duration::from_millis(100));
    }
}

#[test]
fn test_daemon_process_cli_smoke() {
    let dir = TempDir::new().unwrap();
    let home = dir.path();
    let project_dir = dir.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();
    fs::write(project_dir.join("main.rs"), "fn main() {}").unwrap();

    let mut daemon = Command::new(env!("CARGO_BIN_EXE_opendog"))
        .env("HOME", home)
        .args(["daemon"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    wait_for_daemon_ready(home);

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

    let start = run_cli(home, &["start", "--id", "demo"]);
    assert!(start.status.success(), "{:?}", start);
    let start_stdout = String::from_utf8_lossy(&start.stdout);
    assert!(start_stdout.contains("daemon-managed"));
    assert!(start_stdout.contains("initial snapshot"));

    let list = run_cli(home, &["list"]);
    assert!(list.status.success(), "{:?}", list);
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(list_stdout.contains("demo"));
    assert!(list_stdout.contains("monitoring"));

    let stats = run_cli(home, &["stats", "--id", "demo"]);
    assert!(stats.status.success(), "{:?}", stats);
    assert!(String::from_utf8_lossy(&stats.stdout).contains("demo"));

    let guidance = run_cli(home, &["agent-guidance", "--project", "demo", "--json"]);
    assert!(guidance.status.success(), "{:?}", guidance);
    let guidance_json: Value = serde_json::from_slice(&guidance.stdout).unwrap();
    assert_eq!(guidance_json["guidance"]["monitoring_count"], 1);
    assert_eq!(
        guidance_json["guidance"]["monitored_projects"][0].as_str(),
        Some("demo")
    );

    let brief = run_cli(
        home,
        &[
            "decision-brief",
            "--project",
            "demo",
            "--top",
            "1",
            "--json",
        ],
    );
    assert!(brief.status.success(), "{:?}", brief);
    let brief_json: Value = serde_json::from_slice(&brief.stdout).unwrap();
    assert_eq!(
        brief_json["schema_version"].as_str(),
        Some(CLI_DECISION_BRIEF_V1)
    );
    assert_eq!(
        brief_json["decision"]["signals"]["monitoring_count"].as_u64(),
        Some(1)
    );

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

    let start_again = run_cli(home, &["start", "--id", "demo"]);
    assert!(start_again.status.success(), "{:?}", start_again);
    assert!(String::from_utf8_lossy(&start_again.stdout).contains("already running"));

    let global_config = run_cli(home, &["config", "show", "--json"]);
    assert!(global_config.status.success(), "{:?}", global_config);
    let global_config_json: Value = serde_json::from_slice(&global_config.stdout).unwrap();
    assert_eq!(
        global_config_json["schema_version"].as_str(),
        Some(CLI_GLOBAL_CONFIG_V1)
    );
    assert!(global_config_json["global_defaults"]["ignore_patterns"].is_array());

    let project_config = run_cli(home, &["config", "show", "--id", "demo", "--json"]);
    assert!(project_config.status.success(), "{:?}", project_config);
    let project_config_json: Value = serde_json::from_slice(&project_config.stdout).unwrap();
    assert_eq!(
        project_config_json["schema_version"].as_str(),
        Some(CLI_PROJECT_CONFIG_V1)
    );
    assert_eq!(project_config_json["project_id"], "demo");
    assert!(project_config_json["effective"]["process_whitelist"].is_array());

    let update_project_config = run_cli(
        home,
        &[
            "config",
            "set-project",
            "--id",
            "demo",
            "--ignore-pattern",
            "logs",
            "--process",
            "codex",
            "--json",
        ],
    );
    assert!(
        update_project_config.status.success(),
        "{:?}",
        update_project_config
    );
    let update_project_config_json: Value =
        serde_json::from_slice(&update_project_config.stdout).unwrap();
    assert_eq!(
        update_project_config_json["schema_version"].as_str(),
        Some(CLI_UPDATE_PROJECT_CONFIG_V1)
    );
    assert_eq!(
        update_project_config_json["effective"]["ignore_patterns"][0].as_str(),
        Some("logs")
    );
    assert_eq!(
        update_project_config_json["reload"]["runtime_reloaded"].as_bool(),
        Some(true)
    );

    let update_global_config = run_cli(
        home,
        &[
            "config",
            "set-global",
            "--ignore-pattern",
            "generated",
            "--process",
            "claude",
            "--json",
        ],
    );
    assert!(
        update_global_config.status.success(),
        "{:?}",
        update_global_config
    );
    let update_global_config_json: Value =
        serde_json::from_slice(&update_global_config.stdout).unwrap();
    assert_eq!(
        update_global_config_json["schema_version"].as_str(),
        Some(CLI_UPDATE_GLOBAL_CONFIG_V1)
    );
    assert!(update_global_config_json["reloaded_projects"].is_array());

    let reload_project_config = run_cli(home, &["config", "reload", "--id", "demo", "--json"]);
    assert!(
        reload_project_config.status.success(),
        "{:?}",
        reload_project_config
    );
    let reload_project_config_json: Value =
        serde_json::from_slice(&reload_project_config.stdout).unwrap();
    assert_eq!(
        reload_project_config_json["schema_version"].as_str(),
        Some(CLI_RELOAD_PROJECT_CONFIG_V1)
    );
    assert_eq!(reload_project_config_json["project_id"], "demo");
    assert!(reload_project_config_json["reload"].is_object());

    let stop = run_cli(home, &["stop", "--id", "demo"]);
    assert!(stop.status.success(), "{:?}", stop);

    let delete = run_cli(home, &["delete", "--id", "demo"]);
    assert!(delete.status.success(), "{:?}", delete);

    terminate_daemon(&mut daemon);
}
