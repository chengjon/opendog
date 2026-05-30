use crate::contracts::CLI_CLEANUP_PROJECT_DATA_V1;
use crate::control::DaemonClient;
use crate::core::project::ProjectManager;
use crate::core::retention::{self, ProjectDataCleanupRequest};
use crate::error::OpenDogError;

use super::super::output;

pub(in crate::cli) fn cmd_cleanup_data(
    pm: &ProjectManager,
    id: &str,
    request: ProjectDataCleanupRequest,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    let result = match daemon.cleanup_project_data(id, request.clone()) {
        Ok(result) => result,
        Err(OpenDogError::DaemonUnavailable) => {
            let db = pm.open_project_db(id)?;
            retention::cleanup_project_data(&db, &request)?
        }
        Err(error) => return Err(error),
    };

    let payload =
        crate::mcp::cleanup_project_data_payload(CLI_CLEANUP_PROJECT_DATA_V1, id, &result);
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        output::print_cleanup_data_result(id, &result);
    }
    Ok(())
}
