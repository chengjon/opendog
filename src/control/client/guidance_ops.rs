use crate::error::{OpenDogError, Result};
use serde_json::Value;

use super::{ControlRequest, ControlResponse, DaemonClient};

impl DaemonClient {
    pub fn get_data_risk_candidates(
        &self,
        id: &str,
        candidate_type: &str,
        min_review_priority: &str,
        limit: usize,
        schema_version: &str,
    ) -> Result<Value> {
        match self.send(ControlRequest::GetDataRiskCandidates {
            id: id.to_string(),
            candidate_type: candidate_type.to_string(),
            min_review_priority: min_review_priority.to_string(),
            limit,
            schema_version: schema_version.to_string(),
        })? {
            ControlResponse::DataRisk { payload } => Ok(payload),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon data-risk response: {:?}",
                response
            ))),
        }
    }

    pub fn get_workspace_data_risk_overview(
        &self,
        candidate_type: &str,
        min_review_priority: &str,
        project_limit: usize,
        schema_version: &str,
    ) -> Result<Value> {
        match self.send(ControlRequest::GetWorkspaceDataRiskOverview {
            candidate_type: candidate_type.to_string(),
            min_review_priority: min_review_priority.to_string(),
            project_limit,
            schema_version: schema_version.to_string(),
        })? {
            ControlResponse::WorkspaceDataRisk { payload } => Ok(payload),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon workspace data-risk response: {:?}",
                response
            ))),
        }
    }

    pub fn get_agent_guidance(&self, project: Option<&str>, top: usize) -> Result<Value> {
        match self.send(ControlRequest::GetAgentGuidance {
            project: project.map(|value| value.to_string()),
            top,
        })? {
            ControlResponse::AgentGuidance { payload } => Ok(payload),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon agent guidance response: {:?}",
                response
            ))),
        }
    }

    pub fn get_decision_brief(
        &self,
        project: Option<&str>,
        top: usize,
        schema_version: &str,
    ) -> Result<Value> {
        match self.send(ControlRequest::GetDecisionBrief {
            project: project.map(|value| value.to_string()),
            top,
            schema_version: schema_version.to_string(),
        })? {
            ControlResponse::DecisionBrief { payload } => Ok(payload),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon decision brief response: {:?}",
                response
            ))),
        }
    }
}
