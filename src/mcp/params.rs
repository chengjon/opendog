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
        (
            self.id,
            CreateLaneInput {
                lane_id: self.lane_id,
                title: self.title,
                description: self.description,
            },
        )
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
        (
            self.id,
            UpsertNodeInput {
                node_id: self.node_id,
                lane_id: self.lane_id,
                state: self.state,
                summary: self.summary,
                evidence_refs: self.evidence_refs,
                artifact_refs: self.artifact_refs,
                reported_git_head: self.reported_git_head,
                suggested_next: self.suggested_next,
                forbidden_scope: self.forbidden_scope,
                external_anchors: self.external_anchors,
            },
        )
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
        (
            self.id,
            GetGovernanceStateInput {
                lane_id: self.lane_id,
                node_id: self.node_id,
                active_only: self.active_only,
            },
        )
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
        (
            self.id,
            CloseLaneInput {
                lane_id: self.lane_id,
                action: self.action,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::orphan::OrphanSubjectKind;
    use serde_json::json;

    // ---- RecordVerificationParams::into_parts ----

    #[test]
    fn record_verification_into_parts_maps_required_fields() {
        let params = RecordVerificationParams {
            id: "proj-1".to_string(),
            kind: "test".to_string(),
            status: "passed".to_string(),
            command: "cargo test".to_string(),
            exit_code: None,
            summary: None,
            source: None,
            started_at: None,
        };
        let (id, input) = params.into_parts();
        assert_eq!(id, "proj-1");
        assert_eq!(input.kind, "test");
        assert_eq!(input.status, "passed");
        assert_eq!(input.command, "cargo test");
    }

    #[test]
    fn record_verification_into_parts_source_defaults_to_mcp() {
        let params = RecordVerificationParams {
            id: "proj-2".to_string(),
            kind: "lint".to_string(),
            status: "failed".to_string(),
            command: "clippy".to_string(),
            exit_code: None,
            summary: None,
            source: None,
            started_at: None,
        };
        let (_, input) = params.into_parts();
        assert_eq!(input.source, "mcp");
    }

    #[test]
    fn record_verification_into_parts_preserves_all_optionals() {
        let params = RecordVerificationParams {
            id: "proj-3".to_string(),
            kind: "build".to_string(),
            status: "passed".to_string(),
            command: "cargo build".to_string(),
            exit_code: Some(0),
            summary: Some("all good".to_string()),
            source: Some("cli".to_string()),
            started_at: Some("2026-01-01T00:00:00Z".to_string()),
        };
        let (_, input) = params.into_parts();
        assert_eq!(input.exit_code, Some(0));
        assert_eq!(input.summary, Some("all good".to_string()));
        assert_eq!(input.source, "cli");
        assert_eq!(input.started_at, Some("2026-01-01T00:00:00Z".to_string()));
    }

    // ---- ExecuteVerificationParams::into_parts ----

    #[test]
    fn execute_verification_into_parts_maps_required_fields() {
        let params = ExecuteVerificationParams {
            id: "proj-4".to_string(),
            kind: "test".to_string(),
            command: "cargo test".to_string(),
            source: None,
        };
        let (id, input) = params.into_parts();
        assert_eq!(id, "proj-4");
        assert_eq!(input.kind, "test");
        assert_eq!(input.command, "cargo test");
    }

    #[test]
    fn execute_verification_into_parts_source_defaults_to_mcp() {
        let params = ExecuteVerificationParams {
            id: "proj-5".to_string(),
            kind: "lint".to_string(),
            command: "clippy".to_string(),
            source: None,
        };
        let (_, input) = params.into_parts();
        assert_eq!(input.source, "mcp");
    }

    #[test]
    fn execute_verification_into_parts_preserves_explicit_source() {
        let params = ExecuteVerificationParams {
            id: "proj-6".to_string(),
            kind: "build".to_string(),
            command: "cargo build".to_string(),
            source: Some("ci".to_string()),
        };
        let (_, input) = params.into_parts();
        assert_eq!(input.source, "ci");
    }

    // ---- ScanOrphansParams::into_parts ----

    #[test]
    fn scan_orphans_into_parts_defaults() {
        let params = ScanOrphansParams {
            id: "proj-7".to_string(),
            subjects: None,
            external_reports: None,
            include_internal_scanners: None,
            required_scanners: None,
            max_age_secs: None,
            limit: None,
            include_evidence: None,
        };
        let (id, input) = params.into_parts();
        assert_eq!(id, "proj-7");
        assert!(input.subjects.is_none());
        assert!(input.external_reports.is_empty());
        assert!(input.include_internal_scanners);
        assert!(input.required_scanners.is_none());
        assert!(input.max_age_secs.is_none());
        assert!(input.limit.is_none());
        assert!(input.include_evidence);
    }

    #[test]
    fn scan_orphans_into_parts_preserves_explicit_values() {
        let params = ScanOrphansParams {
            id: "proj-8".to_string(),
            subjects: None,
            external_reports: None,
            include_internal_scanners: Some(false),
            required_scanners: Some(vec!["scanner-a".to_string()]),
            max_age_secs: Some(3600),
            limit: Some(10),
            include_evidence: Some(false),
        };
        let (_, input) = params.into_parts();
        assert!(!input.include_internal_scanners);
        assert_eq!(input.required_scanners, Some(vec!["scanner-a".to_string()]));
        assert_eq!(input.max_age_secs, Some(3600));
        assert_eq!(input.limit, Some(10));
        assert!(!input.include_evidence);
    }

    // ---- VerifyDeletionPlanParams::into_parts ----

    #[test]
    fn verify_deletion_plan_into_parts_defaults() {
        let params = VerifyDeletionPlanParams {
            id: "proj-9".to_string(),
            targets: vec![],
            external_reports: None,
            required_project_verification_commands: None,
            max_age_secs: None,
        };
        let (id, input) = params.into_parts();
        assert_eq!(id, "proj-9");
        assert!(input.targets.is_empty());
        assert!(input.external_reports.is_empty());
        assert!(input.required_project_verification_commands.is_empty());
        assert!(input.max_age_secs.is_none());
    }

    #[test]
    fn verify_deletion_plan_into_parts_with_targets() {
        let target = OrphanSubject {
            subject_kind: OrphanSubjectKind::File,
            subject: "dead.rs".to_string(),
            path: Some("src/dead.rs".to_string()),
            display_name: None,
        };
        let params = VerifyDeletionPlanParams {
            id: "proj-10".to_string(),
            targets: vec![target.clone()],
            external_reports: None,
            required_project_verification_commands: Some(vec!["cargo test".to_string()]),
            max_age_secs: Some(7200),
        };
        let (id, input) = params.into_parts();
        assert_eq!(id, "proj-10");
        assert_eq!(input.targets.len(), 1);
        assert_eq!(input.targets[0].subject, "dead.rs");
        assert_eq!(
            input.required_project_verification_commands,
            vec!["cargo test".to_string()]
        );
        assert_eq!(input.max_age_secs, Some(7200));
    }

    // ---- CreateGovernanceLaneParams::into_parts ----

    #[test]
    fn create_governance_lane_into_parts_without_description() {
        let params = CreateGovernanceLaneParams {
            id: "proj-11".to_string(),
            lane_id: "lane-1".to_string(),
            title: "My Lane".to_string(),
            description: None,
        };
        let (id, input) = params.into_parts();
        assert_eq!(id, "proj-11");
        assert_eq!(input.lane_id, "lane-1");
        assert_eq!(input.title, "My Lane");
        assert!(input.description.is_none());
    }

    #[test]
    fn create_governance_lane_into_parts_with_description() {
        let params = CreateGovernanceLaneParams {
            id: "proj-12".to_string(),
            lane_id: "lane-2".to_string(),
            title: "Lane Two".to_string(),
            description: Some("A description".to_string()),
        };
        let (_, input) = params.into_parts();
        assert_eq!(input.description, Some("A description".to_string()));
    }

    // ---- UpsertGovernanceNodeParams::into_parts ----

    #[test]
    fn upsert_governance_node_into_parts_minimal() {
        let params = UpsertGovernanceNodeParams {
            id: "proj-13".to_string(),
            lane_id: "lane-3".to_string(),
            node_id: "G2.46".to_string(),
            state: None,
            summary: None,
            evidence_refs: None,
            artifact_refs: None,
            reported_git_head: None,
            suggested_next: None,
            forbidden_scope: None,
            external_anchors: None,
        };
        let (id, input) = params.into_parts();
        assert_eq!(id, "proj-13");
        assert_eq!(input.node_id, "G2.46");
        assert_eq!(input.lane_id, "lane-3");
        assert!(input.state.is_none());
        assert!(input.summary.is_none());
        assert!(input.external_anchors.is_none());
    }

    #[test]
    fn upsert_governance_node_into_parts_all_fields() {
        let params = UpsertGovernanceNodeParams {
            id: "proj-14".to_string(),
            lane_id: "lane-4".to_string(),
            node_id: "N1".to_string(),
            state: Some("in_progress".to_string()),
            summary: Some("doing work".to_string()),
            evidence_refs: Some(vec!["doc.md".to_string()]),
            artifact_refs: Some(vec!["output.txt".to_string()]),
            reported_git_head: Some("abc123".to_string()),
            suggested_next: Some("continue".to_string()),
            forbidden_scope: Some(vec!["scope-a".to_string()]),
            external_anchors: Some(json!({"key": "value"})),
        };
        let (_, input) = params.into_parts();
        assert_eq!(input.state, Some("in_progress".to_string()));
        assert_eq!(input.summary, Some("doing work".to_string()));
        assert_eq!(input.evidence_refs, Some(vec!["doc.md".to_string()]));
        assert_eq!(input.artifact_refs, Some(vec!["output.txt".to_string()]));
        assert_eq!(input.reported_git_head, Some("abc123".to_string()));
        assert_eq!(input.suggested_next, Some("continue".to_string()));
        assert_eq!(input.forbidden_scope, Some(vec!["scope-a".to_string()]));
        assert_eq!(input.external_anchors, Some(json!({"key": "value"})));
    }

    // ---- GetGovernanceStateParams::into_parts ----

    #[test]
    fn get_governance_state_into_parts_minimal() {
        let params = GetGovernanceStateParams {
            id: "proj-15".to_string(),
            lane_id: None,
            node_id: None,
            active_only: None,
        };
        let (id, input) = params.into_parts();
        assert_eq!(id, "proj-15");
        assert!(input.lane_id.is_none());
        assert!(input.node_id.is_none());
        assert!(input.active_only.is_none());
    }

    #[test]
    fn get_governance_state_into_parts_with_filters() {
        let params = GetGovernanceStateParams {
            id: "proj-16".to_string(),
            lane_id: Some("lane-5".to_string()),
            node_id: Some("N2".to_string()),
            active_only: Some(true),
        };
        let (_, input) = params.into_parts();
        assert_eq!(input.lane_id, Some("lane-5".to_string()));
        assert_eq!(input.node_id, Some("N2".to_string()));
        assert_eq!(input.active_only, Some(true));
    }

    // ---- CloseGovernanceLaneParams::into_parts ----

    #[test]
    fn close_governance_lane_into_parts() {
        let params = CloseGovernanceLaneParams {
            id: "proj-17".to_string(),
            lane_id: "lane-6".to_string(),
            action: "complete".to_string(),
        };
        let (id, input) = params.into_parts();
        assert_eq!(id, "proj-17");
        assert_eq!(input.lane_id, "lane-6");
        assert_eq!(input.action, "complete");
    }
}
