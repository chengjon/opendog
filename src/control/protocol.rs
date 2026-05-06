use crate::config::{
    GlobalConfigUpdateResult, ProjectConfig, ProjectConfigReload, ProjectConfigUpdateResult,
    ProjectConfigView, ProjectInfo,
};
use crate::core::report::{SnapshotComparison, TimeWindowReport, UsageTrendReport};
use crate::core::retention::ProjectDataCleanupResult;
use crate::core::snapshot::SnapshotResult;
use crate::core::stats::ProjectSummary;
use crate::core::verification::ExecutedVerificationResult;
use crate::storage::queries::{StatsEntry, VerificationRun};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    UpdateGlobalConfig {
        ignore_patterns: Option<Vec<String>>,
        process_whitelist: Option<Vec<String>>,
        #[serde(default)]
        add_ignore_patterns: Vec<String>,
        #[serde(default)]
        remove_ignore_patterns: Vec<String>,
        #[serde(default)]
        add_process_whitelist: Vec<String>,
        #[serde(default)]
        remove_process_whitelist: Vec<String>,
    },
    UpdateProjectConfig {
        id: String,
        ignore_patterns: Option<Vec<String>>,
        process_whitelist: Option<Vec<String>>,
        #[serde(default)]
        add_ignore_patterns: Vec<String>,
        #[serde(default)]
        remove_ignore_patterns: Vec<String>,
        #[serde(default)]
        add_process_whitelist: Vec<String>,
        #[serde(default)]
        remove_process_whitelist: Vec<String>,
        inherit_ignore_patterns: bool,
        inherit_process_whitelist: bool,
    },
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
    CleanupProjectData {
        id: String,
        scope: String,
        older_than_days: Option<i64>,
        keep_snapshot_runs: Option<usize>,
        vacuum: bool,
        dry_run: bool,
    },
    ExecuteVerification {
        id: String,
        kind: String,
        command: String,
        source: String,
    },
    RecordVerificationResult {
        id: String,
        kind: String,
        status: String,
        command: String,
        exit_code: Option<i64>,
        summary: Option<String>,
        source: String,
        started_at: Option<String>,
    },
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
