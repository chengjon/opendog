use crate::config::{ConfigPatch, ProjectConfigPatch};
use crate::core::report::ReportWindow;
use crate::core::retention::{CleanupScope, ProjectDataCleanupRequest};
use crate::core::verification::{ExecuteVerificationInput, RecordVerificationInput};

use super::{ControlRequest, ControlResponse, MonitorController};

impl MonitorController {
    pub fn handle_request(&mut self, request: ControlRequest) -> ControlResponse {
        match request {
            ControlRequest::Ping => ControlResponse::Pong,
            ControlRequest::CreateProject { id, path } => match self.create_project(&id, &path) {
                Ok(info) => ControlResponse::ProjectCreated { info },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::DeleteProject { id } => match self.delete_project(&id) {
                Ok(deleted) => ControlResponse::ProjectDeleted { id, deleted },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::ListProjects => match self.list_projects() {
                Ok(projects) => ControlResponse::Projects { projects },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::GetGlobalConfig => match self.global_config() {
                Ok(config) => ControlResponse::GlobalConfig { config },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::GetProjectConfig { id } => match self.project_config_view(&id) {
                Ok(view) => ControlResponse::ProjectConfig { view },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::UpdateGlobalConfig {
                ignore_patterns,
                process_whitelist,
            } => match self.update_global_config(ConfigPatch {
                ignore_patterns,
                process_whitelist,
            }) {
                Ok(result) => ControlResponse::GlobalConfigUpdated { result },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::UpdateProjectConfig {
                id,
                ignore_patterns,
                process_whitelist,
                inherit_ignore_patterns,
                inherit_process_whitelist,
            } => match self.update_project_config(
                &id,
                ProjectConfigPatch {
                    ignore_patterns,
                    process_whitelist,
                    inherit_ignore_patterns,
                    inherit_process_whitelist,
                },
            ) {
                Ok(result) => ControlResponse::ProjectConfigUpdated { result },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::ReloadProjectConfig { id } => match self.reload_project_config(&id) {
                Ok(reload) => match self.pm.effective_project_config(&id) {
                    Ok(effective) => ControlResponse::ProjectConfigReloaded {
                        id,
                        reload,
                        effective,
                    },
                    Err(e) => ControlResponse::Error {
                        message: e.to_string(),
                    },
                },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::ListMonitors => ControlResponse::Monitors {
                ids: self.monitor_ids(),
            },
            ControlRequest::GetStats { id } => match self.get_stats(&id) {
                Ok((summary, entries)) => ControlResponse::Stats {
                    id,
                    summary,
                    entries,
                },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::GetUnusedFiles { id } => match self.get_unused_files(&id) {
                Ok(entries) => ControlResponse::UnusedFiles { id, entries },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::GetTimeWindowReport { id, window, limit } => {
                let window = match ReportWindow::parse(&window) {
                    Ok(window) => window,
                    Err(e) => {
                        return ControlResponse::Error {
                            message: e.to_string(),
                        };
                    }
                };
                match self.get_time_window_report(&id, window, limit) {
                    Ok(report) => ControlResponse::TimeWindowReport { id, report },
                    Err(e) => ControlResponse::Error {
                        message: e.to_string(),
                    },
                }
            }
            ControlRequest::CompareSnapshots {
                id,
                base_run_id,
                head_run_id,
                limit,
            } => match self.compare_snapshots(&id, base_run_id, head_run_id, limit) {
                Ok(comparison) => ControlResponse::SnapshotComparison { id, comparison },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::GetUsageTrends { id, window, limit } => {
                let window = match ReportWindow::parse(&window) {
                    Ok(window) => window,
                    Err(e) => {
                        return ControlResponse::Error {
                            message: e.to_string(),
                        };
                    }
                };
                match self.get_usage_trends(&id, window, limit) {
                    Ok(report) => ControlResponse::UsageTrends { id, report },
                    Err(e) => ControlResponse::Error {
                        message: e.to_string(),
                    },
                }
            }
            ControlRequest::GetDataRiskCandidates {
                id,
                candidate_type,
                min_review_priority,
                limit,
                schema_version,
            } => match self.get_data_risk_candidates(
                &schema_version,
                &id,
                &candidate_type,
                &min_review_priority,
                limit,
            ) {
                Ok(payload) => ControlResponse::DataRisk { payload },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::GetWorkspaceDataRiskOverview {
                candidate_type,
                min_review_priority,
                project_limit,
                schema_version,
            } => match self.get_workspace_data_risk_overview(
                &schema_version,
                &candidate_type,
                &min_review_priority,
                project_limit,
            ) {
                Ok(payload) => ControlResponse::WorkspaceDataRisk { payload },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::GetAgentGuidance { project, top } => {
                match self.get_agent_guidance(project.as_deref(), top) {
                    Ok(payload) => ControlResponse::AgentGuidance { payload },
                    Err(e) => ControlResponse::Error {
                        message: e.to_string(),
                    },
                }
            }
            ControlRequest::GetDecisionBrief {
                project,
                top,
                schema_version,
            } => match self.get_decision_brief(&schema_version, project.as_deref(), top) {
                Ok(payload) => ControlResponse::DecisionBrief { payload },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::GetVerificationStatus { id } => match self.get_verification_status(&id)
            {
                Ok(runs) => ControlResponse::VerificationStatus { id, runs },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::CleanupProjectData {
                id,
                scope,
                older_than_days,
                keep_snapshot_runs,
                vacuum,
                dry_run,
            } => {
                let scope = match CleanupScope::parse(&scope) {
                    Ok(scope) => scope,
                    Err(e) => {
                        return ControlResponse::Error {
                            message: e.to_string(),
                        };
                    }
                };
                match self.cleanup_project_data(
                    &id,
                    ProjectDataCleanupRequest {
                        scope,
                        older_than_days,
                        keep_snapshot_runs,
                        vacuum,
                        dry_run,
                    },
                ) {
                    Ok(result) => ControlResponse::CleanupProjectData { id, result },
                    Err(e) => ControlResponse::Error {
                        message: e.to_string(),
                    },
                }
            }
            ControlRequest::RecordVerificationResult {
                id,
                kind,
                status,
                command,
                exit_code,
                summary,
                source,
                started_at,
            } => match self.record_verification_result(
                &id,
                RecordVerificationInput {
                    kind,
                    status,
                    command,
                    exit_code,
                    summary,
                    source,
                    started_at,
                },
            ) {
                Ok(run) => ControlResponse::VerificationRecorded { id, run },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::ExecuteVerification {
                id,
                kind,
                command,
                source,
            } => match self.execute_verification(
                &id,
                ExecuteVerificationInput {
                    kind,
                    command,
                    source,
                },
            ) {
                Ok(result) => ControlResponse::VerificationExecuted { id, result },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::StartMonitor { id } => match self.start_monitor(&id) {
                Ok(outcome) => ControlResponse::Started {
                    id,
                    already_running: outcome.already_running,
                    snapshot_taken: outcome.snapshot_taken,
                },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
            ControlRequest::StopMonitor { id } => ControlResponse::Stopped {
                was_running: self.stop_monitor(&id),
                id,
            },
            ControlRequest::TakeSnapshot { id } => match self.take_snapshot(&id) {
                Ok(result) => ControlResponse::Snapshot { id, result },
                Err(e) => ControlResponse::Error {
                    message: e.to_string(),
                },
            },
        }
    }
}
