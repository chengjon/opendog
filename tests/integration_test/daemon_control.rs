use opendog::config::ProjectConfigPatch;
use opendog::contracts::{CLI_DATA_RISK_V1, CLI_DECISION_BRIEF_V1, CLI_WORKSPACE_DATA_RISK_V1};
use opendog::control::{spawn_control_server_at, DaemonClient, MonitorController};
use opendog::core::project::ProjectManager;
use opendog::core::report::ReportWindow;
use opendog::core::retention::{CleanupScope, ProjectDataCleanupRequest};
use opendog::storage::database::Database;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_daemon_control_roundtrip() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    let socket_path = dir.path().join("daemon.sock");
    let project_dir = dir.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();
    fs::write(project_dir.join("main.rs"), "fn main() {}").unwrap();

    let pm = ProjectManager::with_data_dir(&data_dir).unwrap();

    let controller = Arc::new(Mutex::new(MonitorController::with_project_manager(pm)));
    let running = Arc::new(AtomicBool::new(true));
    let server = spawn_control_server_at(socket_path.clone(), controller, running.clone()).unwrap();
    let client = DaemonClient::with_socket_path(socket_path);

    std::thread::sleep(Duration::from_millis(100));

    client.ping().unwrap();
    let info = client
        .create_project("demo", project_dir.to_str().unwrap())
        .unwrap();
    assert_eq!(info.id, "demo");

    let initial_config = client.get_project_config("demo").unwrap();
    assert!(!initial_config.effective.ignore_patterns.is_empty());

    let snapshot = client.take_snapshot("demo").unwrap();
    assert_eq!(snapshot.total_files, 1);
    assert_eq!(snapshot.new_files, 1);
    assert_eq!(snapshot.removed_files, 0);

    let (summary, entries) = client.get_stats("demo").unwrap();
    assert_eq!(summary.total_files, 1);
    assert_eq!(summary.accessed_files, 0);
    assert_eq!(summary.unused_files, 1);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_path, "main.rs");

    let unused = client.get_unused_files("demo").unwrap();
    assert_eq!(unused.len(), 1);
    assert_eq!(unused[0].file_path, "main.rs");

    let guidance = client.get_agent_guidance(Some("demo"), 1).unwrap();
    assert_eq!(guidance["guidance"]["project_count"], 1);
    assert!(guidance["guidance"]["recommended_flow"].is_array());

    let data_risk = client
        .get_data_risk_candidates("demo", "all", "low", 5, CLI_DATA_RISK_V1)
        .unwrap();
    assert_eq!(data_risk["schema_version"], CLI_DATA_RISK_V1);
    assert_eq!(data_risk["project_id"], "demo");

    let workspace_risk = client
        .get_workspace_data_risk_overview("all", "low", 5, CLI_WORKSPACE_DATA_RISK_V1)
        .unwrap();
    assert_eq!(workspace_risk["schema_version"], CLI_WORKSPACE_DATA_RISK_V1);
    assert_eq!(workspace_risk["total_registered_projects"], 1);

    let decision_brief = client
        .get_decision_brief(Some("demo"), 1, CLI_DECISION_BRIEF_V1)
        .unwrap();
    assert_eq!(decision_brief["schema_version"], CLI_DECISION_BRIEF_V1);
    assert_eq!(decision_brief["scope"], "project");
    assert_eq!(decision_brief["selected_project_id"], "demo");

    let report_db = Database::open_project(&info.db_path).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    report_db
        .execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["main.rs", "codex", 111i64, (now - 60).to_string()],
        )
        .unwrap();
    report_db
        .execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["main.rs", "codex", 111i64, (now - 30).to_string()],
        )
        .unwrap();
    report_db
        .execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["main.rs", "codex", 111i64, (now - 86_400 - 30).to_string()],
        )
        .unwrap();
    report_db
        .execute(
            "INSERT INTO file_events (file_path, event_type, event_time) VALUES (?1, 'modify', ?2)",
            rusqlite::params!["main.rs", (now - 20).to_string()],
        )
        .unwrap();

    let time_window = client
        .get_time_window_report("demo", ReportWindow::Hours24, 10)
        .unwrap();
    assert_eq!(time_window.window, "24h");
    assert_eq!(time_window.summary.total_sightings, 2);
    assert_eq!(time_window.files[0].file_path, "main.rs");

    fs::write(
        project_dir.join("main.rs"),
        "fn main() { println!(\"hi\"); }",
    )
    .unwrap();
    fs::write(project_dir.join("lib.rs"), "pub fn helper() {}").unwrap();
    let second_snapshot = client.take_snapshot("demo").unwrap();
    assert_eq!(second_snapshot.total_files, 2);

    let comparison = client.compare_snapshots("demo", None, None, 10).unwrap();
    assert_eq!(comparison.summary.added_files, 1);
    assert_eq!(comparison.summary.modified_files, 1);
    assert!(comparison
        .changes
        .iter()
        .any(|entry| entry.file_path == "lib.rs" && entry.change_type == "added"));

    let trends = client
        .get_usage_trends("demo", ReportWindow::Days7, 10)
        .unwrap();
    assert_eq!(trends.window, "7d");
    assert!(trends
        .files
        .iter()
        .any(|entry| entry.file_path == "main.rs"));

    let cleanup_preview = client
        .cleanup_project_data(
            "demo",
            ProjectDataCleanupRequest {
                scope: CleanupScope::Activity,
                older_than_days: Some(1),
                keep_snapshot_runs: None,
                vacuum: false,
                dry_run: true,
            },
        )
        .unwrap();
    assert!(cleanup_preview.dry_run);
    assert!(cleanup_preview.deleted.file_sightings >= 1);

    let start = client.start_monitor("demo").unwrap();
    assert!(!start.already_running);
    assert!(!start.snapshot_taken);

    let projects = client.list_projects().unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].id, "demo");
    assert_eq!(projects[0].status, "monitoring");

    let ids = client.list_monitors().unwrap();
    assert_eq!(ids, vec!["demo".to_string()]);

    let second_start = client.start_monitor("demo").unwrap();
    assert!(second_start.already_running);
    assert!(!second_start.snapshot_taken);

    let config_update = client
        .update_project_config(
            "demo",
            ProjectConfigPatch {
                ignore_patterns: Some(vec!["logs".to_string()]),
                process_whitelist: Some(vec!["codex".to_string()]),
                inherit_ignore_patterns: false,
                inherit_process_whitelist: false,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        config_update.effective.ignore_patterns,
        vec!["logs".to_string()]
    );
    assert_eq!(
        config_update.reload.changed_fields,
        vec![
            "ignore_patterns".to_string(),
            "process_whitelist".to_string()
        ]
    );
    assert!(config_update.reload.runtime_reloaded);

    let reloaded_config = client.get_project_config("demo").unwrap();
    assert_eq!(
        reloaded_config.effective.ignore_patterns,
        vec!["logs".to_string()]
    );

    assert!(client.stop_monitor("demo").unwrap());
    assert!(!client.stop_monitor("demo").unwrap());
    assert!(client.delete_project("demo").unwrap());
    assert!(!client.delete_project("demo").unwrap());

    running.store(false, Ordering::Relaxed);
    server.join().unwrap();
}
