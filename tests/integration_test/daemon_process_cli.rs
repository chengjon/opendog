use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use opendog::contracts::CLI_DECISION_BRIEF_V1;
use serde_json::Value;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

use super::common::run_cli;

#[path = "daemon_process_cli/config_flow.rs"]
mod config_flow;
#[path = "daemon_process_cli/report_flow.rs"]
mod report_flow;

fn wait_for_daemon_ready(home: &Path) {
    let socket = home.join(".opendog/data/daemon.sock");
    let socket_dir = socket.parent().expect("daemon socket should have a parent");
    fs::create_dir_all(socket_dir).expect("daemon socket directory should be creatable");

    let (tx, rx) = mpsc::channel::<()>();
    let watched_socket = socket.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res: std::result::Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if event.paths.iter().any(|path| path == &watched_socket) {
                    let _ = tx.send(());
                }
            }
        },
        Config::default(),
    )
    .expect("daemon socket watcher should start");
    watcher
        .watch(socket_dir, RecursiveMode::NonRecursive)
        .expect("daemon socket directory should be watchable");

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if socket.exists() {
            let output = run_cli(home, &["list"]);
            if output.status.success() {
                return;
            }
        }
        let now = Instant::now();
        if now >= deadline {
            break;
        }

        let remaining = deadline.saturating_duration_since(now);
        let wait_for = if socket.exists() {
            remaining.min(Duration::from_millis(100))
        } else {
            remaining
        };
        let _ = rx.recv_timeout(wait_for);
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

fn wait_for_path_exists(path: &Path, timeout: Duration) {
    if path.exists() {
        return;
    }

    let parent = path.parent().expect("watched path should have a parent");
    fs::create_dir_all(parent).expect("watched path parent should be creatable");
    let (tx, rx) = mpsc::channel::<()>();
    let mut watcher = RecommendedWatcher::new(
        move |res: std::result::Result<notify::Event, notify::Error>| {
            if res.is_ok() {
                let _ = tx.send(());
            }
        },
        Config::default(),
    )
    .expect("path watcher should start");
    watcher
        .watch(parent, RecursiveMode::NonRecursive)
        .expect("path parent should be watchable");

    let deadline = Instant::now() + timeout;
    while !path.exists() {
        let now = Instant::now();
        if now >= deadline {
            break;
        }
        let _ = rx.recv_timeout(deadline.saturating_duration_since(now));
    }
    assert!(
        path.exists(),
        "path did not become ready: {}",
        path.display()
    );
}

fn wait_for_foreground_monitor_ready(child: &mut Child) -> std::thread::JoinHandle<String> {
    let mut stdout = child
        .stdout
        .take()
        .expect("foreground monitor stdout should be piped");
    let (ready_tx, ready_rx) = mpsc::channel::<()>();
    let reader = std::thread::spawn(move || {
        let mut output = String::new();
        let mut ready_sent = false;
        let mut buf = [0_u8; 1024];
        loop {
            match stdout.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    output.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if !ready_sent && output.contains("Monitor running. Press Ctrl+C to stop.") {
                        let _ = ready_tx.send(());
                        ready_sent = true;
                    }
                }
                Err(_) => break,
            }
        }
        output
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("foreground monitor should report readiness");
    reader
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

#[test]
fn test_foreground_monitor_stops_on_sigint() {
    let dir = TempDir::new().unwrap();
    let home = dir.path();
    let project_dir = dir.path().join("foreground-project");
    fs::create_dir_all(&project_dir).unwrap();
    fs::write(project_dir.join("main.rs"), "fn main() {}").unwrap();

    let create = run_cli(
        home,
        &[
            "register",
            "--id",
            "foreground",
            "--path",
            project_dir.to_str().unwrap(),
        ],
    );
    assert!(create.status.success(), "{:?}", create);

    let mut monitor = Command::new(env!("CARGO_BIN_EXE_opendog"))
        .env("HOME", home)
        .args(["start", "--id", "foreground"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let lock_path = home
        .join(".opendog/data/projects/foreground.db")
        .with_extension("monitor.lock");
    wait_for_path_exists(&lock_path, Duration::from_secs(5));
    let reader = wait_for_foreground_monitor_ready(&mut monitor);

    let pid = monitor.id().to_string();
    let signal = Command::new("kill").args(["-INT", &pid]).status().unwrap();
    assert!(signal.success(), "failed to send SIGINT to monitor");

    let (done_tx, done_rx) = mpsc::channel();
    let waiter = std::thread::spawn(move || {
        let result = monitor.wait();
        let _ = done_tx.send(result);
    });
    let status = done_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("foreground monitor should stop after SIGINT")
        .expect("foreground monitor wait should succeed");
    let output = reader.join().expect("stdout reader should join");
    let _ = waiter.join();

    assert!(
        status.success(),
        "foreground monitor exited with {status:?}"
    );
    assert!(output.contains("Monitor stopped."));
}
