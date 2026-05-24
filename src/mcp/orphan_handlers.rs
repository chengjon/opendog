use rmcp::handler::server::wrapper::Json;
use serde_json::Value;

use crate::control::DaemonClient;
use crate::core::orphan;
use crate::error::OpenDogError;

use super::{
    error_json_for, orphan_deletion_plan_payload, orphan_scan_payload, OpenDogServer,
    ScanOrphansParams, VerifyDeletionPlanParams, MCP_ORPHAN_DELETION_PLAN_V1, MCP_ORPHAN_SCAN_V1,
};

pub(super) fn handle_scan_orphans(
    server: &OpenDogServer,
    params: ScanOrphansParams,
) -> Json<Value> {
    let (id, input) = params.into_parts();

    match DaemonClient::new().scan_orphans(&id, input.clone()) {
        Ok(result) => return Json(orphan_scan_payload(&id, &result)),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_ORPHAN_SCAN_V1, Some(&id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (info, config) = server.get_project_with_config(&id)?;
        orphan::scan_project_orphans(&info.root_path, &config, input)
    })();
    match result {
        Ok(result) => Json(orphan_scan_payload(&id, &result)),
        Err(e) => error_json_for(MCP_ORPHAN_SCAN_V1, Some(&id), &e),
    }
}

pub(super) fn handle_verify_deletion_plan(
    server: &OpenDogServer,
    params: VerifyDeletionPlanParams,
) -> Json<Value> {
    let (id, input) = params.into_parts();

    match DaemonClient::new().verify_deletion_plan(&id, input.clone()) {
        Ok(result) => return Json(orphan_deletion_plan_payload(&id, &result)),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_ORPHAN_DELETION_PLAN_V1, Some(&id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (info, config) = server.get_project_with_config(&id)?;
        orphan::verify_deletion_plan(&info.root_path, &config, input)
    })();
    match result {
        Ok(result) => Json(orphan_deletion_plan_payload(&id, &result)),
        Err(e) => error_json_for(MCP_ORPHAN_DELETION_PLAN_V1, Some(&id), &e),
    }
}
