use rmcp::schemars;
use serde::Deserialize;

use crate::core::orphan::{
    DeletionPlanInput, ExternalScannerReport, OrphanSubject, ScanOrphansInput,
};
use crate::core::verification::{ExecuteVerificationInput, RecordVerificationInput};

#[derive(Deserialize, schemars::JsonSchema)]
pub struct RecordVerificationParams {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub command: String,
    pub exit_code: Option<i64>,
    pub summary: Option<String>,
    pub source: Option<String>,
    pub started_at: Option<String>,
}

impl RecordVerificationParams {
    pub(crate) fn into_parts(self) -> (String, RecordVerificationInput) {
        (
            self.id,
            RecordVerificationInput {
                kind: self.kind,
                status: self.status,
                command: self.command,
                exit_code: self.exit_code,
                summary: self.summary,
                source: self.source.unwrap_or_else(|| "mcp".to_string()),
                started_at: self.started_at,
            },
        )
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct ExecuteVerificationParams {
    pub id: String,
    pub kind: String,
    pub command: String,
    pub source: Option<String>,
}

impl ExecuteVerificationParams {
    pub(crate) fn into_parts(self) -> (String, ExecuteVerificationInput) {
        (
            self.id,
            ExecuteVerificationInput {
                kind: self.kind,
                command: self.command,
                source: self.source.unwrap_or_else(|| "mcp".to_string()),
            },
        )
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct ScanOrphansParams {
    pub id: String,
    pub subjects: Option<Vec<OrphanSubject>>,
    pub external_reports: Option<Vec<ExternalScannerReport>>,
    pub include_internal_scanners: Option<bool>,
    pub required_scanners: Option<Vec<String>>,
    pub max_age_secs: Option<u64>,
    pub limit: Option<usize>,
    pub include_evidence: Option<bool>,
}

impl ScanOrphansParams {
    pub(crate) fn into_parts(self) -> (String, ScanOrphansInput) {
        (
            self.id,
            ScanOrphansInput {
                subjects: self.subjects,
                external_reports: self.external_reports.unwrap_or_default(),
                include_internal_scanners: self.include_internal_scanners.unwrap_or(true),
                required_scanners: self.required_scanners,
                max_age_secs: self.max_age_secs,
                limit: self.limit,
                include_evidence: self.include_evidence.unwrap_or(true),
            },
        )
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct VerifyDeletionPlanParams {
    pub id: String,
    pub targets: Vec<OrphanSubject>,
    pub external_reports: Option<Vec<ExternalScannerReport>>,
    pub required_project_verification_commands: Option<Vec<String>>,
    pub max_age_secs: Option<u64>,
}

impl VerifyDeletionPlanParams {
    pub(crate) fn into_parts(self) -> (String, DeletionPlanInput) {
        (
            self.id,
            DeletionPlanInput {
                targets: self.targets,
                external_reports: self.external_reports.unwrap_or_default(),
                required_project_verification_commands: self
                    .required_project_verification_commands
                    .unwrap_or_default(),
                max_age_secs: self.max_age_secs,
            },
        )
    }
}
