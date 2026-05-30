use crate::config::{ConfigPatch, ProjectConfigPatch, RetentionPolicy};
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

use super::super::output;

pub(super) fn parse_retention_policy_json(
    value: Option<String>,
) -> Result<Option<RetentionPolicy>, OpenDogError> {
    value
        .map(|raw| {
            serde_json::from_str::<RetentionPolicy>(&raw).map_err(|err| {
                OpenDogError::InvalidInput(format!(
                    "retention policy JSON must match the RetentionPolicy schema: {}",
                    err
                ))
            })
        })
        .transpose()
}

pub(super) fn cmd_config_show(
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

pub(super) fn cmd_config_set_project(
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

pub(super) fn cmd_config_set_global(
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

pub(super) fn cmd_config_reload(
    pm: &ProjectManager,
    id: &str,
    json_output: bool,
) -> Result<(), OpenDogError> {
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
