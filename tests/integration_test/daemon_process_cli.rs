use opendog::contracts::CLI_DECISION_BRIEF_V1;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};
use tempfile::TempDir;

use super::common::run_cli;

#[path = "daemon_process_cli/config_flow.rs"]
mod config_flow;
#[path = "daemon_process_cli/report_flow.rs"]
mod report_flow;

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

fn terminate_daemon(mut child: Child) {
    let pid = child.id().to_string();
    let _ = Command::new("kill").args(["-TERM", &pid]).status();

    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let waiter = std::thread::spawn(move || {
        let result = child.wait();
        let _ = done_tx.send(result);
    });

    if done_rx.recv_timeout(Duration::from_secs(5)).is_err() {
        let _ = Command::new("kill").args(["-KILL", &pid]).status();
    }
    let _ = waiter.join();
}

#[test]
fn test_daemon_process_cli_smoke() {
    let dir = TempDir::new().unwrap();
    let home = dir.path();
    let project_dir = dir.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();
    fs::write(project_dir.join("main.rs"), "fn main() {}").unwrap();

    let daemon = Command::new(env!("CARGO_BIN_EXE_opendog"))
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
            "register",
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

    report_flow::assert_report_and_cleanup_flow(home, &project_dir);
    config_flow::assert_config_reload_flow(home);

    let stop = run_cli(home, &["stop", "--id", "demo"]);
    assert!(stop.status.success(), "{:?}", stop);

    let delete = run_cli(home, &["delete", "--id", "demo"]);
    assert!(delete.status.success(), "{:?}", delete);

    terminate_daemon(daemon);
}
