use rmcp::handler::server::wrapper::Json;
use serde_json::Value;

use crate::control::DaemonClient;
use crate::core::governance::{
    self, CloseLaneInput, CreateLaneInput, GetGovernanceStateInput, UpsertNodeInput,
};
use crate::error::OpenDogError;

use super::{
    close_governance_lane_payload, create_governance_lane_payload, error_json_for,
    get_governance_state_payload, upsert_governance_node_payload, OpenDogServer,
    MCP_CLOSE_GOVERNANCE_LANE_V1, MCP_CREATE_GOVERNANCE_LANE_V1, MCP_GET_GOVERNANCE_STATE_V1,
    MCP_UPSERT_GOVERNANCE_NODE_V1,
};

pub(super) fn handle_create_governance_lane(
    server: &OpenDogServer,
    id: &str,
    input: CreateLaneInput,
) -> Json<Value> {
    match DaemonClient::new().create_governance_lane(id, input.clone()) {
        Ok(lane) => return Json(create_governance_lane_payload(id, &lane)),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_CREATE_GOVERNANCE_LANE_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, _) = server.get_project(id)?;
        governance::create_lane(&db, input)
    })();
    match result {
        Ok(lane) => Json(create_governance_lane_payload(id, &lane)),
        Err(e) => error_json_for(MCP_CREATE_GOVERNANCE_LANE_V1, Some(id), &e),
    }
}

pub(super) fn handle_upsert_governance_node(
    server: &OpenDogServer,
    id: &str,
    input: UpsertNodeInput,
) -> Json<Value> {
    match DaemonClient::new().upsert_governance_node(id, input.clone()) {
        Ok(node_result) => return Json(upsert_governance_node_payload(id, &node_result)),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_UPSERT_GOVERNANCE_NODE_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, _) = server.get_project(id)?;
        governance::upsert_node(&db, input)
    })();
    match result {
        Ok(node_result) => Json(upsert_governance_node_payload(id, &node_result)),
        Err(e) => error_json_for(MCP_UPSERT_GOVERNANCE_NODE_V1, Some(id), &e),
    }
}

pub(super) fn handle_get_governance_state(
    server: &OpenDogServer,
    id: &str,
    input: GetGovernanceStateInput,
) -> Json<Value> {
    match DaemonClient::new().get_governance_state(id, input.clone()) {
        Ok(state) => return Json(get_governance_state_payload(id, &state)),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_GET_GOVERNANCE_STATE_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, _) = server.get_project(id)?;
        governance::get_governance_state(&db, input)
    })();
    match result {
        Ok(state) => Json(get_governance_state_payload(id, &state)),
        Err(e) => error_json_for(MCP_GET_GOVERNANCE_STATE_V1, Some(id), &e),
    }
}

pub(super) fn handle_close_governance_lane(
    server: &OpenDogServer,
    id: &str,
    input: CloseLaneInput,
) -> Json<Value> {
    let lane_id = input.lane_id.clone();
    let action = input.action.clone();

    match DaemonClient::new().close_governance_lane(id, input.clone()) {
        Ok((_, status, nodes_affected)) => {
            return Json(close_governance_lane_payload(
                id,
                &lane_id,
                &action,
                &status,
                nodes_affected,
            ))
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_CLOSE_GOVERNANCE_LANE_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, _) = server.get_project(id)?;
        governance::close_lane(&db, input)
    })();
    match result {
        Ok((status, nodes_affected)) => Json(close_governance_lane_payload(
            id,
            &lane_id,
            &action,
            &status,
            nodes_affected,
        )),
        Err(e) => error_json_for(MCP_CLOSE_GOVERNANCE_LANE_V1, Some(id), &e),
    }
}
