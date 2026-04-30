#[path = "integration_test/common.rs"]
mod common;
#[path = "integration_test/storage_project_snapshot.rs"]
mod storage_project_snapshot;

#[cfg(unix)]
#[path = "integration_test/cli_export.rs"]
mod cli_export;
#[cfg(unix)]
#[path = "integration_test/cli_guidance.rs"]
mod cli_guidance;
#[cfg(unix)]
#[path = "integration_test/daemon_control.rs"]
mod daemon_control;
#[cfg(unix)]
#[path = "integration_test/daemon_process_cli.rs"]
mod daemon_process_cli;
