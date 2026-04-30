use crate::core::verification::{
    ExecuteVerificationInput, ExecutedVerificationResult, RecordVerificationInput,
};
use crate::error::{OpenDogError, Result};
use crate::storage::queries::VerificationRun;

use super::{ControlRequest, ControlResponse, DaemonClient};

impl DaemonClient {
    pub fn get_verification_status(&self, id: &str) -> Result<Vec<VerificationRun>> {
        match self.send(ControlRequest::GetVerificationStatus { id: id.to_string() })? {
            ControlResponse::VerificationStatus { runs, .. } => Ok(runs),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon verification status response: {:?}",
                response
            ))),
        }
    }

    pub fn record_verification_result(
        &self,
        id: &str,
        input: RecordVerificationInput,
    ) -> Result<VerificationRun> {
        match self.send(ControlRequest::RecordVerificationResult {
            id: id.to_string(),
            kind: input.kind,
            status: input.status,
            command: input.command,
            exit_code: input.exit_code,
            summary: input.summary,
            source: input.source,
            started_at: input.started_at,
        })? {
            ControlResponse::VerificationRecorded { run, .. } => Ok(run),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon verification record response: {:?}",
                response
            ))),
        }
    }

    pub fn execute_verification(
        &self,
        id: &str,
        input: ExecuteVerificationInput,
    ) -> Result<ExecutedVerificationResult> {
        match self.send(ControlRequest::ExecuteVerification {
            id: id.to_string(),
            kind: input.kind,
            command: input.command,
            source: input.source,
        })? {
            ControlResponse::VerificationExecuted { result, .. } => Ok(result),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon verification execute response: {:?}",
                response
            ))),
        }
    }
}
