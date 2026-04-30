use rmcp::schemars;
use serde::Deserialize;

use crate::config::{ConfigPatch, ProjectConfigPatch};
use crate::core::verification::{ExecuteVerificationInput, RecordVerificationInput};

#[derive(Deserialize, schemars::JsonSchema)]
pub struct CreateProjectParams {
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
pub struct GlobalConfigParams {}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct UpdateGlobalConfigParams {
    pub ignore_patterns: Option<Vec<String>>,
    pub process_whitelist: Option<Vec<String>>,
}

impl UpdateGlobalConfigParams {
    pub(super) fn into_patch(self) -> ConfigPatch {
        ConfigPatch {
            ignore_patterns: self.ignore_patterns,
            process_whitelist: self.process_whitelist,
        }
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct UpdateProjectConfigParams {
    pub id: String,
    pub ignore_patterns: Option<Vec<String>>,
    pub process_whitelist: Option<Vec<String>>,
    pub inherit_ignore_patterns: Option<bool>,
    pub inherit_process_whitelist: Option<bool>,
}

impl UpdateProjectConfigParams {
    pub(super) fn into_parts(self) -> (String, ProjectConfigPatch) {
        (
            self.id,
            ProjectConfigPatch {
                ignore_patterns: self.ignore_patterns,
                process_whitelist: self.process_whitelist,
                inherit_ignore_patterns: self.inherit_ignore_patterns.unwrap_or(false),
                inherit_process_whitelist: self.inherit_process_whitelist.unwrap_or(false),
            },
        )
    }
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
pub struct ExportProjectEvidenceParams {
    pub id: String,
    pub format: String,
    pub view: Option<String>,
    pub output_path: String,
    pub min_access_count: Option<i64>,
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
pub struct CleanupProjectDataParams {
    pub id: String,
    /// Required cleanup scope: "activity", "snapshots", "verification", or "all"
    pub scope: String,
    /// Optional retention cutoff in days for activity/verification cleanup. `0` means everything older than now.
    pub older_than_days: Option<i64>,
    /// Optional snapshot-run count to retain for snapshot-history cleanup.
    pub keep_snapshot_runs: Option<usize>,
    /// Optional heavy compaction step. When true, rewrites the SQLite file with `VACUUM`; cannot be combined with `dry_run=true`.
    pub vacuum: Option<bool>,
    /// Optional dry-run mode, defaults to true when omitted.
    pub dry_run: Option<bool>,
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
