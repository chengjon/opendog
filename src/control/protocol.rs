use crate::config::{
    ConfigPatch, GlobalConfigUpdateResult, ProjectConfig, ProjectConfigPatch,
    ProjectConfigReload, ProjectConfigUpdateResult, ProjectConfigView, ProjectInfo,
};
use crate::core::report::{SnapshotComparison, TimeWindowReport, UsageTrendReport};
use crate::core::retention::{ProjectDataCleanupRequest, ProjectDataCleanupResult};
use crate::core::snapshot::SnapshotResult;
use crate::core::stats::ProjectSummary;
use crate::core::verification::{ExecuteVerificationInput, ExecutedVerificationResult, RecordVerificationInput};
use crate::storage::queries::{StatsEntry, VerificationRun};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProjectConfigFields {
    pub id: String,
    #[serde(flatten)]
    pub patch: ProjectConfigPatch,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordVerificationFields {
    pub id: String,
    #[serde(flatten)]
    pub input: RecordVerificationInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteVerificationFields {
    pub id: String,
    #[serde(flatten)]
    pub input: ExecuteVerificationInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupProjectDataFields {
    pub id: String,
    #[serde(flatten)]
    pub request: ProjectDataCleanupRequest,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ControlRequest {
    Ping,
    CreateProject {
        id: String,
        path: String,
    },
    DeleteProject {
        id: String,
    },
    ListProjects,
    ListMonitors,
    GetStats {
        id: String,
    },
    GetGlobalConfig,
    GetProjectConfig {
        id: String,
    },
    UpdateGlobalConfig(ConfigPatch),
    UpdateProjectConfig(UpdateProjectConfigFields),
    ReloadProjectConfig {
        id: String,
    },
    GetUnusedFiles {
        id: String,
    },
    GetTimeWindowReport {
        id: String,
        window: String,
        limit: usize,
    },
    CompareSnapshots {
        id: String,
        base_run_id: Option<i64>,
        head_run_id: Option<i64>,
        limit: usize,
    },
    GetUsageTrends {
        id: String,
        window: String,
        limit: usize,
    },
    GetDataRiskCandidates {
        id: String,
        candidate_type: String,
        min_review_priority: String,
        limit: usize,
        schema_version: String,
    },
    GetWorkspaceDataRiskOverview {
        candidate_type: String,
        min_review_priority: String,
        project_limit: usize,
        schema_version: String,
    },
    GetAgentGuidance {
        project: Option<String>,
        top: usize,
    },
    GetDecisionBrief {
        project: Option<String>,
        top: usize,
        schema_version: String,
    },
    GetVerificationStatus {
        id: String,
    },
    CleanupProjectData(CleanupProjectDataFields),
    ExecuteVerification(ExecuteVerificationFields),
    RecordVerificationResult(RecordVerificationFields),
    StartMonitor {
        id: String,
    },
    StopMonitor {
        id: String,
    },
    TakeSnapshot {
        id: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ControlResponse {
    Pong,
    ProjectCreated {
        info: ProjectInfo,
    },
    ProjectDeleted {
        id: String,
        deleted: bool,
    },
    Projects {
        projects: Vec<ProjectInfo>,
    },
    Monitors {
        ids: Vec<String>,
    },
    Stats {
        id: String,
        summary: ProjectSummary,
        entries: Vec<StatsEntry>,
    },
    GlobalConfig {
        config: ProjectConfig,
    },
    ProjectConfig {
        view: ProjectConfigView,
    },
    GlobalConfigUpdated {
        result: GlobalConfigUpdateResult,
    },
    ProjectConfigUpdated {
        result: ProjectConfigUpdateResult,
    },
    ProjectConfigReloaded {
        id: String,
        reload: ProjectConfigReload,
        effective: ProjectConfig,
    },
    UnusedFiles {
        id: String,
        entries: Vec<StatsEntry>,
    },
    TimeWindowReport {
        id: String,
        report: TimeWindowReport,
    },
    SnapshotComparison {
        id: String,
        comparison: SnapshotComparison,
    },
    UsageTrends {
        id: String,
        report: UsageTrendReport,
    },
    DataRisk {
        payload: Value,
    },
    WorkspaceDataRisk {
        payload: Value,
    },
    AgentGuidance {
        payload: Value,
    },
    DecisionBrief {
        payload: Value,
    },
    VerificationStatus {
        id: String,
        runs: Vec<VerificationRun>,
    },
    CleanupProjectData {
        id: String,
        result: ProjectDataCleanupResult,
    },
    VerificationRecorded {
        id: String,
        run: VerificationRun,
    },
    VerificationExecuted {
        id: String,
        result: ExecutedVerificationResult,
    },
    Started {
        id: String,
        already_running: bool,
        snapshot_taken: bool,
    },
    Stopped {
        id: String,
        was_running: bool,
    },
    Snapshot {
        id: String,
        result: SnapshotResult,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct StartMonitorOutcome {
    pub already_running: bool,
    pub snapshot_taken: bool,
}
