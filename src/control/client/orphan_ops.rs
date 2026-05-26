use crate::core::orphan::{
    DeletionPlanInput, DeletionPlanVerification, ScanOrphansInput, ScanOrphansResult,
};
use crate::error::{OpenDogError, Result};

use super::{ControlRequest, ControlResponse, DaemonClient};

impl DaemonClient {
    pub fn scan_orphans(&self, id: &str, input: ScanOrphansInput) -> Result<ScanOrphansResult> {
        match self.send(ControlRequest::ScanOrphans {
            id: id.to_string(),
            input,
        })? {
            ControlResponse::OrphansScanned { result, .. } => Ok(result),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon scan-orphans response: {:?}",
                response
            ))),
        }
    }

    pub fn verify_deletion_plan(
        &self,
        id: &str,
        input: DeletionPlanInput,
    ) -> Result<DeletionPlanVerification> {
        match self.send(ControlRequest::VerifyDeletionPlan {
            id: id.to_string(),
            input,
        })? {
            ControlResponse::DeletionPlanVerified { result, .. } => Ok(result),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon verify-deletion-plan response: {:?}",
                response
            ))),
        }
    }
}
