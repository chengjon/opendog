use clap::Subcommand;

use crate::config::{ConfigPatch, ProjectConfigPatch};
use crate::contracts::{
    CLI_GLOBAL_CONFIG_V1, CLI_PROJECT_CONFIG_V1, CLI_RELOAD_PROJECT_CONFIG_V1,
    CLI_UPDATE_GLOBAL_CONFIG_V1, CLI_UPDATE_PROJECT_CONFIG_V1,
};
use crate::control::{DaemonClient, MonitorController};
use crate::core::project::ProjectManager;
use crate::error::OpenDogError;
use crate::mcp::{
    global_config_payload, project_config_payload, project_config_reload_payload,
    project_config_update_payload, update_global_config_payload,
};

use super::output;

#[derive(Subcommand)]
pub(super) enum ConfigCommand {
    /// Show effective configuration for one project, or global defaults when no project is supplied
    Show {
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Update per-project override fields
    SetProject {
        #[arg(short, long)]
        id: String,
        #[arg(long = "ignore-pattern")]
        ignore_patterns: Vec<String>,
        #[arg(long = "process")]
        process_whitelist: Vec<String>,
        #[arg(long)]
        inherit_ignore_patterns: bool,
        #[arg(long)]
        inherit_process_whitelist: bool,
        #[arg(long)]
        json: bool,
    },
    /// Update global default configuration
    SetGlobal {
        #[arg(long = "ignore-pattern")]
        ignore_patterns: Vec<String>,
        #[arg(long = "process")]
        process_whitelist: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Re-apply persisted configuration to a running daemon-managed monitor
    Reload {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        json: bool,
    },
}

pub(super) fn cmd_config(pm: &ProjectManager, command: ConfigCommand) -> Result<(), OpenDogError> {
    match command {
        ConfigCommand::Show { id, json } => cmd_config_show(pm, id, json),
        ConfigCommand::SetProject {
            id,
            ignore_patterns,
            process_whitelist,
            inherit_ignore_patterns,
            inherit_process_whitelist,
            json,
        } => cmd_config_set_project(
            pm,
            &id,
            ProjectConfigPatch {
                ignore_patterns: if ignore_patterns.is_empty() {
                    None
                } else {
                    Some(ignore_patterns)
                },
                process_whitelist: if process_whitelist.is_empty() {
                    None
                } else {
                    Some(process_whitelist)
                },
                inherit_ignore_patterns,
                inherit_process_whitelist,
            },
            json,
        ),
        ConfigCommand::SetGlobal {
            ignore_patterns,
            process_whitelist,
            json,
        } => cmd_config_set_global(
            pm,
            ConfigPatch {
                ignore_patterns: if ignore_patterns.is_empty() {
                    None
                } else {
                    Some(ignore_patterns)
                },
                process_whitelist: if process_whitelist.is_empty() {
                    None
                } else {
                    Some(process_whitelist)
                },
            },
            json,
        ),
        ConfigCommand::Reload { id, json } => cmd_config_reload(pm, &id, json),
    }
}

fn cmd_config_show(
    pm: &ProjectManager,
    id: Option<String>,
    json_output: bool,
) -> Result<(), OpenDogError> {
    if let Some(id) = id {
        let daemon = DaemonClient::new();
        let view = match daemon.get_project_config(&id) {
            Ok(view) => view,
            Err(OpenDogError::DaemonUnavailable) => pm.project_config_view(&id)?,
            Err(e) => return Err(e),
        };
        let payload = project_config_payload(CLI_PROJECT_CONFIG_V1, &view);
        if json_output {
            println!("{}", serde_json::to_string_pretty(&payload)?);
        } else {
            output::print_project_config(&view);
        }
        return Ok(());
    }

    let daemon = DaemonClient::new();
    let config = match daemon.global_config() {
        Ok(config) => config,
        Err(OpenDogError::DaemonUnavailable) => pm.global_config()?,
        Err(e) => return Err(e),
    };
    let payload = global_config_payload(CLI_GLOBAL_CONFIG_V1, &config);
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        output::print_global_config(&config);
    }
    Ok(())
}

fn cmd_config_set_project(
    pm: &ProjectManager,
    id: &str,
    patch: ProjectConfigPatch,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    let result = match daemon.update_project_config(id, patch.clone()) {
        Ok(result) => result,
        Err(OpenDogError::DaemonUnavailable) => {
            let mut controller = MonitorController::with_project_manager(
                ProjectManager::with_data_dir(&crate::config::data_dir())?,
            );
            controller.update_project_config(id, patch)?
        }
        Err(e) => return Err(e),
    };
    let payload = project_config_update_payload(CLI_UPDATE_PROJECT_CONFIG_V1, &result);
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        output::print_project_config_update(&result);
    }
    let _ = pm;
    Ok(())
}

fn cmd_config_set_global(
    pm: &ProjectManager,
    patch: ConfigPatch,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    let result = match daemon.update_global_config(patch.clone()) {
        Ok(result) => result,
        Err(OpenDogError::DaemonUnavailable) => {
            let mut controller = MonitorController::with_project_manager(
                ProjectManager::with_data_dir(&crate::config::data_dir())?,
            );
            controller.update_global_config(patch)?
        }
        Err(e) => return Err(e),
    };
    let payload = update_global_config_payload(CLI_UPDATE_GLOBAL_CONFIG_V1, &result);
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        output::print_global_config_update(&result);
    }
    let _ = pm;
    Ok(())
}

fn cmd_config_reload(pm: &ProjectManager, id: &str, json_output: bool) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    let (reload, effective) = match daemon.reload_project_config(id) {
        Ok(result) => result,
        Err(OpenDogError::DaemonUnavailable) => {
            let mut controller = MonitorController::with_project_manager(
                ProjectManager::with_data_dir(&crate::config::data_dir())?,
            );
            let reload = controller.reload_project_config(id)?;
            let effective = controller.project_manager().effective_project_config(id)?;
            (reload, effective)
        }
        Err(e) => return Err(e),
    };
    let payload =
        project_config_reload_payload(CLI_RELOAD_PROJECT_CONFIG_V1, id, &reload, &effective);
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        output::print_project_config_reload(id, &reload, &effective);
    }
    let _ = pm;
    Ok(())
}
