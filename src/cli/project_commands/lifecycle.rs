use crate::control::{DaemonClient, ProjectLifecycle, SnapshotMonitor};
use crate::core::monitor;
use crate::core::project::ProjectManager;
use crate::error::OpenDogError;

use super::super::output;
use super::project_lifecycle;

pub(in crate::cli) fn cmd_register(
    pm: &ProjectManager,
    id: &str,
    path: &str,
) -> Result<(), OpenDogError> {
    let svc = project_lifecycle(pm);
    let info = svc.create_project(id, path)?;
    output::print_registered(&info);
    Ok(())
}

pub(in crate::cli) fn cmd_snapshot(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let svc = project_lifecycle(pm);
    let result = svc.take_snapshot(id)?;
    output::print_snapshot_result(id, &result);
    Ok(())
}

pub(in crate::cli) fn cmd_start(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    match daemon.start_monitor(id) {
        Ok(outcome) => {
            if outcome.already_running {
                println!(
                    "Daemon-managed monitor already running for project '{}'.",
                    id
                );
            } else if outcome.snapshot_taken {
                println!(
                    "Started daemon-managed monitor for project '{}' after taking an initial snapshot.",
                    id
                );
            } else {
                println!("Started daemon-managed monitor for project '{}'.", id);
            }
            return Ok(());
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return Err(e),
    }

    let info = pm
        .get(id)?
        .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
    let effective_config = pm.resolve_project_config(&info)?;
    let db = pm.open_project_db(id)?;
    drop(db);

    println!("Starting monitor for project '{}'...", id);
    let handle = monitor::start_monitor(&info.db_path, info.root_path.clone(), effective_config)?;
    let stop_requested = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_requested_handler = stop_requested.clone();
    ctrlc::set_handler(move || {
        stop_requested_handler.store(true, std::sync::atomic::Ordering::Relaxed);
    })
    .map_err(|e| OpenDogError::Io(std::io::Error::other(e.to_string())))?;

    println!("Monitor running. Press Ctrl+C to stop.");
    while handle.is_running() && !stop_requested.load(std::sync::atomic::Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    handle.stop();
    println!("Monitor stopped.");
    Ok(())
}

pub(in crate::cli) fn cmd_stop(id: &str) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    match daemon.stop_monitor(id)? {
        true => println!("Stopped daemon-managed monitor for project '{}'.", id),
        false => println!("No daemon-managed monitor running for project '{}'.", id),
    }
    Ok(())
}

pub(in crate::cli) fn cmd_list(pm: &ProjectManager) -> Result<(), OpenDogError> {
    let svc = project_lifecycle(pm);
    let projects = svc.list_projects()?;
    output::print_project_list(&projects);
    Ok(())
}

pub(in crate::cli) fn cmd_delete(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let svc = project_lifecycle(pm);
    match svc.delete_project(id)? {
        true => println!("Project '{}' deleted.", id),
        false => {
            eprintln!("Project '{}' not found.", id);
            std::process::exit(1);
        }
    }
    Ok(())
}
