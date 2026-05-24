use crate::contracts::{
    versioned_project_payload, CLI_CLOSE_GOVERNANCE_LANE_V1, CLI_CREATE_GOVERNANCE_LANE_V1,
    CLI_GET_GOVERNANCE_STATE_V1, CLI_UPSERT_GOVERNANCE_NODE_V1,
};
use crate::core::governance::{
    self, CloseLaneInput, CreateLaneInput, GetGovernanceStateInput, UpsertNodeInput,
};
use crate::core::project::ProjectManager;
use crate::error::OpenDogError;
use super::output;

pub(super) fn cmd_create_lane(pm: &ProjectManager, id: &str, input: CreateLaneInput, json_output: bool) -> Result<(), OpenDogError> {
    let db = pm.open_project_db(id)?;
    let lane = governance::create_lane(&db, input)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&versioned_project_payload(
            CLI_CREATE_GOVERNANCE_LANE_V1, id, [("lane", serde_json::json!(lane))],
        ))?);
    } else {
        output::print_lane_created(id, &lane);
    }
    Ok(())
}

pub(super) fn cmd_upsert_node(pm: &ProjectManager, id: &str, input: UpsertNodeInput, json_output: bool) -> Result<(), OpenDogError> {
    let db = pm.open_project_db(id)?;
    let result = governance::upsert_node(&db, input)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&versioned_project_payload(
            CLI_UPSERT_GOVERNANCE_NODE_V1, id, [("result", serde_json::json!(result))],
        ))?);
    } else {
        output::print_node_upserted(id, &result);
    }
    Ok(())
}

pub(super) fn cmd_show(pm: &ProjectManager, id: &str, input: GetGovernanceStateInput, json_output: bool) -> Result<(), OpenDogError> {
    let db = pm.open_project_db(id)?;
    let state = governance::get_governance_state(&db, input)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&versioned_project_payload(
            CLI_GET_GOVERNANCE_STATE_V1, id, [("governance", serde_json::json!(state))],
        ))?);
    } else {
        output::print_governance_state(id, &state);
    }
    Ok(())
}

pub(super) fn cmd_close_lane(pm: &ProjectManager, id: &str, input: CloseLaneInput, json_output: bool) -> Result<(), OpenDogError> {
    let lane_id = input.lane_id.clone();
    let action = input.action.clone();
    let db = pm.open_project_db(id)?;
    let (status, nodes) = governance::close_lane(&db, input)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&versioned_project_payload(
            CLI_CLOSE_GOVERNANCE_LANE_V1, id, [
                ("lane_id", serde_json::json!(lane_id)),
                ("action_taken", serde_json::json!(action)),
                ("status", serde_json::json!(status)),
                ("nodes_affected", serde_json::json!(nodes)),
            ],
        ))?);
    } else {
        output::print_lane_closed(id, &lane_id, &action, &status, nodes);
    }
    Ok(())
}
