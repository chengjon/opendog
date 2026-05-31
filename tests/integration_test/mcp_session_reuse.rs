use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use rmcp::{
    model::CallToolRequestParams,
    service::RunningService,
    transport::{ConfigureCommandExt, TokioChildProcess},
    RoleClient, ServiceExt,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn daemon_pid_path(root: &Path) -> PathBuf {
    root.join("data/daemon.pid")
}

fn terminate_background_daemon(root: &Path) {
    let pid_path = daemon_pid_path(root);
    let Ok(pid) = fs::read_to_string(&pid_path) else {
        return;
    };
    let pid = pid.trim();
    if pid.is_empty() {
        return;
    }

    let (tx, rx) = mpsc::channel::<()>();
    let watched_pid_path = pid_path.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res: std::result::Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if event.paths.iter().any(|path| path == &watched_pid_path) {
                    let _ = tx.send(());
                }
            }
        },
        Config::default(),
    )
    .expect("pid watcher should start");
    watcher
        .watch(
            pid_path.parent().unwrap_or_else(|| Path::new(".")),
            RecursiveMode::NonRecursive,
        )
        .expect("pid directory should be watchable");

    let _ = Command::new("kill").args(["-TERM", pid]).status();
    let deadline = Instant::now() + Duration::from_secs(5);
    while pid_path.exists() {
        let now = Instant::now();
        if now >= deadline {
            return;
        }
        if rx
            .recv_timeout(deadline.saturating_duration_since(now))
            .is_err()
        {
            return;
        }
    }
}

async fn spawn_mcp_client(
    home: &Path,
    opendog_home: Option<&Path>,
) -> Result<RunningService<RoleClient, ()>, Box<dyn std::error::Error>> {
    let transport = TokioChildProcess::new(
        tokio::process::Command::new(env!("CARGO_BIN_EXE_opendog")).configure(|cmd| {
            cmd.env("HOME", home);
            if let Some(opendog_home) = opendog_home {
                cmd.env("OPENDOG_HOME", opendog_home);
            }
            cmd.arg("mcp");
        }),
    )?;
    let client = ().serve(transport).await?;
    Ok(client)
}

fn structured_payload(result: rmcp::model::CallToolResult) -> Value {
    result
        .structured_content
        .expect("tool call should return structured_content")
}

#[tokio::test]
async fn mcp_sessions_reuse_daemon_backed_monitor_state_without_manual_daemon_start(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let home = dir.path();
    let opendog_home = home.join(".opendog");
    let project_dir = home.join("project");
    fs::create_dir_all(&project_dir)?;
    fs::write(project_dir.join("main.rs"), "fn main() {}")?;

    let first_client = spawn_mcp_client(home, None).await?;
    let _ = structured_payload(
        first_client
            .call_tool(
                CallToolRequestParams::new("register_project").with_arguments(
                    json!({
                        "id": "demo",
                        "path": project_dir.display().to_string()
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            )
            .await?,
    );

    let start_payload = structured_payload(
        first_client
            .call_tool(
                CallToolRequestParams::new("start_monitor").with_arguments(
                    json!({
                        "id": "demo"
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            )
            .await?,
    );
    assert_eq!(start_payload["status"], "monitoring");
    first_client.cancel().await?;

    let second_client = spawn_mcp_client(home, None).await?;
    let list_payload = structured_payload(
        second_client
            .call_tool(CallToolRequestParams::new("list_projects"))
            .await?,
    );
    second_client.cancel().await?;

    let projects = list_payload["projects"]
        .as_array()
        .expect("projects should be an array");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0]["id"], "demo");
    assert_eq!(projects[0]["status"], "monitoring");

    terminate_background_daemon(&opendog_home);
    Ok(())
}

#[tokio::test]
async fn mcp_sessions_reuse_shared_state_when_opendog_home_is_explicit(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let first_home = dir.path().join("home-a");
    let second_home = dir.path().join("home-b");
    let opendog_home = dir.path().join("shared-opendog");
    let project_dir = dir.path().join("project");
    fs::create_dir_all(&first_home)?;
    fs::create_dir_all(&second_home)?;
    fs::create_dir_all(&project_dir)?;
    fs::write(project_dir.join("main.rs"), "fn main() {}")?;

    let first_client = spawn_mcp_client(&first_home, Some(&opendog_home)).await?;
    let _ = structured_payload(
        first_client
            .call_tool(
                CallToolRequestParams::new("register_project").with_arguments(
                    json!({
                        "id": "demo",
                        "path": project_dir.display().to_string()
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            )
            .await?,
    );

    let start_payload = structured_payload(
        first_client
            .call_tool(
                CallToolRequestParams::new("start_monitor").with_arguments(
                    json!({
                        "id": "demo"
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            )
            .await?,
    );
    assert_eq!(start_payload["status"], "monitoring");
    first_client.cancel().await?;

    let second_client = spawn_mcp_client(&second_home, Some(&opendog_home)).await?;
    let list_payload = structured_payload(
        second_client
            .call_tool(CallToolRequestParams::new("list_projects"))
            .await?,
    );
    second_client.cancel().await?;

    let projects = list_payload["projects"]
        .as_array()
        .expect("projects should be an array");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0]["id"], "demo");
    assert_eq!(projects[0]["status"], "monitoring");

    terminate_background_daemon(&opendog_home);
    Ok(())
}

#[tokio::test]
async fn mcp_scan_orphans_blocks_entrypoint_referenced_file(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let home = dir.path();
    let opendog_home = home.join(".opendog");
    let project_dir = home.join("project");
    fs::create_dir_all(project_dir.join("src/api"))?;
    fs::write(project_dir.join("src/api/old.py"), "app = object()\n")?;
    fs::write(
        project_dir.join("Dockerfile"),
        "CMD [\"uvicorn\", \"src.api.old:app\"]\n",
    )?;

    let client = spawn_mcp_client(home, None).await?;
    let _ = structured_payload(
        client
            .call_tool(
                CallToolRequestParams::new("register_project").with_arguments(
                    json!({
                        "id": "demo",
                        "path": project_dir.display().to_string()
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            )
            .await?,
    );

    let scan_payload = structured_payload(
        client
            .call_tool(
                CallToolRequestParams::new("scan_orphans").with_arguments(
                    json!({
                        "id": "demo",
                        "subjects": [{
                            "subject_kind": "file",
                            "subject": "src/api/old.py",
                            "path": "src/api/old.py",
                            "display_name": null
                        }]
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            )
            .await?,
    );
    client.cancel().await?;

    assert_eq!(scan_payload["schema_version"], "opendog.mcp.orphan-scan.v1");
    assert_eq!(scan_payload["project_id"], "demo");
    assert_eq!(scan_payload["summary"]["blocked_count"], 1);
    assert_eq!(scan_payload["candidates"][0]["classification"], "blocked");
    let scanners = scan_payload["scanner_health"]
        .as_array()
        .expect("scanner_health should be an array");
    assert!(scanners
        .iter()
        .any(|entry| entry["scanner"] == "entrypoint_scanner"));

    terminate_background_daemon(&opendog_home);
    Ok(())
}
