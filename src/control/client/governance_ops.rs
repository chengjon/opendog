use crate::core::governance::{
    CloseLaneInput, CreateLaneInput, GetGovernanceStateInput, GovernanceState, UpsertNodeInput,
    UpsertNodeResult,
};
use crate::error::{OpenDogError, Result};
use crate::storage::queries::GovernanceLane;

use super::{ControlRequest, ControlResponse, DaemonClient};

impl DaemonClient {
    pub fn create_governance_lane(
        &self,
        id: &str,
        input: CreateLaneInput,
    ) -> Result<GovernanceLane> {
        match self.send(ControlRequest::CreateGovernanceLane {
            id: id.to_string(),
            input,
        })? {
            ControlResponse::GovernanceLaneCreated { lane, .. } => Ok(lane),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon create-governance-lane response: {:?}",
                response
            ))),
        }
    }

    pub fn upsert_governance_node(
        &self,
        id: &str,
        input: UpsertNodeInput,
    ) -> Result<UpsertNodeResult> {
        match self.send(ControlRequest::UpsertGovernanceNode {
            id: id.to_string(),
            input,
        })? {
            ControlResponse::GovernanceNodeUpserted { result, .. } => Ok(result),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon upsert-governance-node response: {:?}",
                response
            ))),
        }
    }

    pub fn get_governance_state(
        &self,
        id: &str,
        input: GetGovernanceStateInput,
    ) -> Result<GovernanceState> {
        match self.send(ControlRequest::GetGovernanceState {
            id: id.to_string(),
            input,
        })? {
            ControlResponse::GovernanceState { state, .. } => Ok(state),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon get-governance-state response: {:?}",
                response
            ))),
        }
    }

    pub fn close_governance_lane(
        &self,
        id: &str,
        input: CloseLaneInput,
    ) -> Result<(String, String, usize)> {
        let lane_id = input.lane_id.clone();
        match self.send(ControlRequest::CloseGovernanceLane {
            id: id.to_string(),
            input,
        })? {
            ControlResponse::GovernanceLaneClosed {
                status,
                nodes_affected,
                ..
            } => Ok((lane_id, status, nodes_affected)),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon close-governance-lane response: {:?}",
                response
            ))),
        }
    }
}
