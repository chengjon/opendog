use rmcp::schemars;
use serde::Deserialize;

use crate::core::governance::{
    CloseLaneInput, CreateLaneInput, GetGovernanceStateInput, UpsertNodeInput,
};
use crate::core::orphan::{
    DeletionPlanInput, ExternalScannerReport, OrphanSubject, ScanOrphansInput,
};
use crate::core::verification::{ExecuteVerificationInput, RecordVerificationInput};

#[derive(Deserialize, schemars::JsonSchema)]
pub struct RegisterProjectParams {
    /// Unique project identifier (alphanumeric, dash, underscore, max 64 chars)
    pub id: String,
    /// Absolute path to the project root directory
    pub path: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct ProjectIdParams {
    /// Project identifier
    pub id: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct ObservationRowsParams {
    /// Project identifier
    pub id: String,
    /// Optional file row limit, defaults to 50 for MCP payload safety.
    pub limit: Option<usize>,
    /// Optional row classification filter: "all" (default), "source", "infrastructure", "backup", or "project".
    pub path_classification: Option<String>,
}

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
    pub(super) fn into_parts(self) -> (String, RecordVerificationInput) {
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
    pub(super) fn into_parts(self) -> (String, ExecuteVerificationInput) {
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
    pub(super) fn into_parts(self) -> (String, ScanOrphansInput) {
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
    pub(super) fn into_parts(self) -> (String, DeletionPlanInput) {
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

#[derive(Deserialize, schemars::JsonSchema)]
pub struct TimeWindowReportParams {
    pub id: String,
    /// Optional window: "24h", "7d", or "30d". Defaults to "24h".
    pub window: Option<String>,
    /// Optional row limit, defaults to 10.
    pub limit: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct CompareSnapshotsParams {
    pub id: String,
    /// Optional base snapshot run id. Must be paired with head_run_id when supplied.
    pub base_run_id: Option<i64>,
    /// Optional head snapshot run id. Must be paired with base_run_id when supplied.
    pub head_run_id: Option<i64>,
    /// Optional change row limit, defaults to 20.
    pub limit: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct UsageTrendParams {
    pub id: String,
    /// Optional window: "24h", "7d", or "30d". Defaults to "7d".
    pub window: Option<String>,
    /// Optional file limit, defaults to 10.
    pub limit: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct DataRiskParams {
    pub id: String,
    /// Optional filter: "all" (default), "mock", or "hardcoded"
    pub candidate_type: Option<String>,
    /// Optional minimum priority: "low", "medium", or "high"
    pub min_review_priority: Option<String>,
    /// Optional per-list result limit, defaults to 20
    pub limit: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct WorkspaceDataRiskParams {
    /// Optional filter: "all" (default), "mock", or "hardcoded"
    pub candidate_type: Option<String>,
    /// Optional minimum priority: "low", "medium", or "high"
    pub min_review_priority: Option<String>,
    /// Optional maximum number of matching projects to return, defaults to 20
    pub project_limit: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct AgentGuidanceParams {
    /// Optional single-project scope
    pub project_id: Option<String>,
    /// Optional priority list limit, defaults to 5
    pub top: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct DecisionBriefParams {
    /// Optional single-project scope
    pub project_id: Option<String>,
    /// Optional priority list limit, defaults to 5
    pub top: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct GuidanceParams {
    /// Optional single-project scope
    pub project_id: Option<String>,
    /// Optional priority list limit, defaults to 5
    pub top: Option<usize>,
    /// Optional merged response mode: "summary" (default) or "decision"
    pub detail: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct CreateGovernanceLaneParams {
    /// Project identifier
    pub id: String,
    /// Unique lane identifier
    pub lane_id: String,
    /// Lane title
    pub title: String,
    /// Optional lane description
    pub description: Option<String>,
}

impl CreateGovernanceLaneParams {
    pub(super) fn into_parts(self) -> (String, CreateLaneInput) {
        (self.id, CreateLaneInput { lane_id: self.lane_id, title: self.title, description: self.description })
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct UpsertGovernanceNodeParams {
    /// Project identifier
    pub id: String,
    /// Lane identifier this node belongs to
    pub lane_id: String,
    /// Unique node identifier (e.g. "G2.46")
    pub node_id: String,
    /// Node state — required on create, optional on update
    pub state: Option<String>,
    /// One-line factual summary
    pub summary: Option<String>,
    /// JSON array of report/document paths
    pub evidence_refs: Option<Vec<String>>,
    /// JSON array of generated artifact paths
    pub artifact_refs: Option<Vec<String>>,
    /// Caller-reported HEAD anchor
    pub reported_git_head: Option<String>,
    /// Recommended next step
    pub suggested_next: Option<String>,
    /// JSON array of semantic scope descriptions
    pub forbidden_scope: Option<Vec<String>>,
    /// JSON object with external references
    pub external_anchors: Option<serde_json::Value>,
}

impl UpsertGovernanceNodeParams {
    pub(super) fn into_parts(self) -> (String, UpsertNodeInput) {
        (self.id, UpsertNodeInput {
            node_id: self.node_id, lane_id: self.lane_id, state: self.state,
            summary: self.summary, evidence_refs: self.evidence_refs,
            artifact_refs: self.artifact_refs, reported_git_head: self.reported_git_head,
            suggested_next: self.suggested_next, forbidden_scope: self.forbidden_scope,
            external_anchors: self.external_anchors,
        })
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct GetGovernanceStateParams {
    /// Project identifier
    pub id: String,
    /// Optional lane filter
    pub lane_id: Option<String>,
    /// Optional specific node filter
    pub node_id: Option<String>,
    /// When true, filter out closed/completed lanes and closed nodes
    pub active_only: Option<bool>,
}

impl GetGovernanceStateParams {
    pub(super) fn into_parts(self) -> (String, GetGovernanceStateInput) {
        (self.id, GetGovernanceStateInput { lane_id: self.lane_id, node_id: self.node_id, active_only: self.active_only })
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct CloseGovernanceLaneParams {
    /// Project identifier
    pub id: String,
    /// Lane identifier
    pub lane_id: String,
    /// Action: "complete", "defer", or "delete"
    pub action: String,
}

impl CloseGovernanceLaneParams {
    pub(super) fn into_parts(self) -> (String, CloseLaneInput) {
        (self.id, CloseLaneInput { lane_id: self.lane_id, action: self.action })
    }
}
