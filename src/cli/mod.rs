pub mod output;

use clap::Parser;
use std::path::Path;

use crate::core::monitor;
use crate::core::project::ProjectManager;
use crate::core::{snapshot, stats};
use crate::error::OpenDogError;

#[derive(Parser)]
#[command(name = "opendog", version, about = "Multi-project file monitor for AI workflows")]
enum Cli {
    /// Register a new project
    Create {
        /// Unique project identifier
        #[arg(short, long)]
        id: String,
        /// Absolute path to project root directory
        #[arg(short, long)]
        path: String,
    },
    /// Trigger a file scan for a project
    Snapshot {
        /// Project identifier
        #[arg(short, long)]
        id: String,
    },
    /// Start monitoring a project (blocks until Ctrl+C)
    Start {
        /// Project identifier
        #[arg(short, long)]
        id: String,
    },
    /// Run as stdio MCP server (for AI clients)
    Mcp,
    /// Show usage statistics for a project
    Stats {
        /// Project identifier
        #[arg(short, long)]
        id: String,
    },
    /// List never-accessed files (unused candidates)
    Unused {
        /// Project identifier
        #[arg(short, long)]
        id: String,
    },
    /// List all registered projects
    List,
    /// Delete a project and all its data
    Delete {
        /// Project identifier
        #[arg(short, long)]
        id: String,
    },
    /// Run as background daemon (for systemd)
    Daemon,
}

pub fn run() {
    let cli = Cli::parse();
    let pm = ProjectManager::new().unwrap_or_else(|e| {
        eprintln!("Error: failed to initialize — {}", e);
        std::process::exit(1);
    });

    let result = match cli {
        Cli::Create { id, path } => cmd_create(&pm, &id, &path),
        Cli::Snapshot { id } => cmd_snapshot(&pm, &id),
        Cli::Start { id } => cmd_start(&pm, &id),
        Cli::Mcp => {
            crate::mcp::run_stdio();
            return;
        }
        Cli::Stats { id } => cmd_stats(&pm, &id),
        Cli::Unused { id } => cmd_unused(&pm, &id),
        Cli::List => cmd_list(&pm),
        Cli::Delete { id } => cmd_delete(&pm, &id),
        Cli::Daemon => {
            crate::daemon::run();
            return;
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn cmd_create(pm: &ProjectManager, id: &str, path: &str) -> Result<(), OpenDogError> {
    let info = pm.create(id, Path::new(path))?;
    output::print_created(&info);
    Ok(())
}

fn cmd_snapshot(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let db = pm.open_project_db(id)?;
    let info = pm.get(id)?.ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
    let result = snapshot::take_snapshot(&db, &info.root_path, &info.config)?;
    output::print_snapshot_result(id, &result);
    Ok(())
}

fn cmd_start(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let info = pm.get(id)?.ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
    let db = pm.open_project_db(id)?;
    drop(db);

    println!("Starting monitor for project '{}'...", id);
    let handle = monitor::start_monitor(&info.db_path, info.root_path.clone(), info.config.clone())?;
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

fn cmd_stats(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let db = pm.open_project_db(id)?;
    let summary = stats::get_summary(&db)?;
    let entries = stats::get_stats(&db)?;
    output::print_stats(id, &summary, &entries);
    Ok(())
}

fn cmd_unused(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let db = pm.open_project_db(id)?;
    let unused = stats::get_unused_files(&db)?;
    output::print_unused(id, &unused);
    Ok(())
}

fn cmd_list(pm: &ProjectManager) -> Result<(), OpenDogError> {
    let projects = pm.list()?;
    output::print_project_list(&projects);
    Ok(())
}

fn cmd_delete(pm: &ProjectManager, id: &str) -> Result<(), OpenDogError> {
    let deleted = pm.delete(id)?;
    if deleted {
        println!("Project '{}' deleted.", id);
    } else {
        eprintln!("Project '{}' not found.", id);
        std::process::exit(1);
    }
    Ok(())
}
