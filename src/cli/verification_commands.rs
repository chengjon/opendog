use crate::contracts::{
    versioned_project_payload, CLI_RECORD_VERIFICATION_V1, CLI_RUN_VERIFICATION_V1,
    CLI_VERIFICATION_STATUS_V1,
};
use crate::control::DaemonClient;
use crate::core::project::ProjectManager;
use crate::core::verification::{self, ExecuteVerificationInput, RecordVerificationInput};
use crate::error::OpenDogError;
use crate::mcp::verification_status_layer;

use super::output;

pub(super) fn cmd_record_verification(
    pm: &ProjectManager,
    id: &str,
    input: RecordVerificationInput,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    let run = match daemon.record_verification_result(id, input.clone()) {
        Ok(run) => run,
        Err(OpenDogError::DaemonUnavailable) => {
            let db = pm.open_project_db(id)?;
            verification::record_verification_result(&db, input)?
        }
        Err(e) => return Err(e),
    };
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&versioned_project_payload(
                CLI_RECORD_VERIFICATION_V1,
                id,
                [("recorded", serde_json::json!(run))],
            ))?
        );
    } else {
        output::print_verification_recorded(id, &run);
    }
    Ok(())
}

pub(super) fn cmd_verification(
    pm: &ProjectManager,
    id: &str,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    let runs = match daemon.get_verification_status(id) {
        Ok(runs) => runs,
        Err(OpenDogError::DaemonUnavailable) => {
            let db = pm.open_project_db(id)?;
            verification::get_latest_verification_runs(&db)?
        }
        Err(e) => return Err(e),
    };
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&versioned_project_payload(
                CLI_VERIFICATION_STATUS_V1,
                id,
                [("verification", verification_status_layer(&runs))],
            ))?
        );
    } else {
        output::print_verification_status(id, &runs);
    }
    Ok(())
}

pub(super) fn cmd_run_verification(
    pm: &ProjectManager,
    id: &str,
    input: ExecuteVerificationInput,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    let result = match daemon.execute_verification(id, input.clone()) {
        Ok(result) => result,
        Err(OpenDogError::DaemonUnavailable) => {
            let info = pm
                .get(id)?
                .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
            let db = pm.open_project_db(id)?;
            verification::execute_verification_command(&db, &info.root_path, input)?
        }
        Err(e) => return Err(e),
    };
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&versioned_project_payload(
                CLI_RUN_VERIFICATION_V1,
                id,
                [("executed", serde_json::json!(result))],
            ))?
        );
    } else {
        output::print_verification_executed(id, &result);
    }
    Ok(())
}
