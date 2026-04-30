use rmcp::handler::server::wrapper::Json;
use serde_json::Value;

use crate::control::DaemonClient;
use crate::core::verification;
use crate::core::verification::{ExecuteVerificationInput, RecordVerificationInput};
use crate::error::OpenDogError;

use super::{
    error_json_for, record_verification_payload, run_verification_payload,
    verification_status_payload, OpenDogServer, MCP_RECORD_VERIFICATION_V1,
    MCP_RUN_VERIFICATION_V1, MCP_VERIFICATION_STATUS_V1,
};

pub(super) fn handle_get_verification_status(server: &OpenDogServer, id: &str) -> Json<Value> {
    match DaemonClient::new().get_verification_status(id) {
        Ok(runs) => return Json(verification_status_payload(id, &runs)),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_VERIFICATION_STATUS_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, _) = server.get_project(id)?;
        verification::get_latest_verification_runs(&db)
    })();
    match result {
        Ok(runs) => Json(verification_status_payload(id, &runs)),
        Err(e) => error_json_for(MCP_VERIFICATION_STATUS_V1, Some(id), &e),
    }
}

pub(super) fn handle_record_verification_result(
    server: &OpenDogServer,
    id: &str,
    input: RecordVerificationInput,
) -> Json<Value> {
    match DaemonClient::new().record_verification_result(id, input.clone()) {
        Ok(run) => {
            return Json(record_verification_payload(id, &run));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_RECORD_VERIFICATION_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, _) = server.get_project(id)?;
        verification::record_verification_result(&db, input)
    })();
    match result {
        Ok(run) => Json(record_verification_payload(id, &run)),
        Err(e) => error_json_for(MCP_RECORD_VERIFICATION_V1, Some(id), &e),
    }
}

pub(super) fn handle_run_verification_command(
    server: &OpenDogServer,
    id: &str,
    input: ExecuteVerificationInput,
) -> Json<Value> {
    match DaemonClient::new().execute_verification(id, input.clone()) {
        Ok(result) => {
            return Json(run_verification_payload(id, &result));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_RUN_VERIFICATION_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, info) = server.get_project(id)?;
        verification::execute_verification_command(&db, &info.root_path, input)
    })();
    match result {
        Ok(result) => Json(run_verification_payload(id, &result)),
        Err(e) => error_json_for(MCP_RUN_VERIFICATION_V1, Some(id), &e),
    }
}
