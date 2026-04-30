use crate::config::ProjectInfo;
use crate::core::report::{
    self, ReportWindow, SnapshotComparison, TimeWindowReport, UsageTrendReport,
};
use crate::core::retention::{self, ProjectDataCleanupRequest, ProjectDataCleanupResult};
use crate::core::stats::{self, ProjectSummary};
use crate::core::verification::{
    self, ExecuteVerificationInput, ExecutedVerificationResult, RecordVerificationInput,
};
use crate::error::{OpenDogError, Result};
use crate::guidance::{
    build_agent_guidance_for_projects, build_decision_brief_for_projects,
    load_project_guidance_data,
};
use crate::mcp::{
    normalize_candidate_type, normalize_min_review_priority, project_data_risk_payload,
    workspace_data_risk_payload,
};
use crate::storage::queries::{StatsEntry, VerificationRun};
use serde_json::Value;

use super::MonitorController;

impl MonitorController {
    pub fn get_stats(&self, id: &str) -> Result<(ProjectSummary, Vec<StatsEntry>)> {
        self.pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = self.pm.open_project_db(id)?;
        let summary = stats::get_summary(&db)?;
        let entries = stats::get_stats(&db)?;
        Ok((summary, entries))
    }

    pub fn get_unused_files(&self, id: &str) -> Result<Vec<StatsEntry>> {
        self.pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = self.pm.open_project_db(id)?;
        stats::get_unused_files(&db)
    }

    pub fn get_time_window_report(
        &self,
        id: &str,
        window: ReportWindow,
        limit: usize,
    ) -> Result<TimeWindowReport> {
        self.pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = self.pm.open_project_db(id)?;
        report::get_time_window_report(&db, window, limit)
    }

    pub fn compare_snapshots(
        &self,
        id: &str,
        base_run_id: Option<i64>,
        head_run_id: Option<i64>,
        limit: usize,
    ) -> Result<SnapshotComparison> {
        self.pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = self.pm.open_project_db(id)?;
        match (base_run_id, head_run_id) {
            (None, None) => report::compare_latest_snapshots(&db, limit),
            (Some(base_run_id), Some(head_run_id)) => {
                report::compare_snapshot_runs(&db, base_run_id, head_run_id, limit)
            }
            _ => Err(OpenDogError::InvalidInput(
                "base_run_id and head_run_id must be provided together".to_string(),
            )),
        }
    }

    pub fn get_usage_trends(
        &self,
        id: &str,
        window: ReportWindow,
        limit: usize,
    ) -> Result<UsageTrendReport> {
        self.pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = self.pm.open_project_db(id)?;
        report::get_usage_trend_report(&db, window, limit)
    }

    pub fn cleanup_project_data(
        &self,
        id: &str,
        request: ProjectDataCleanupRequest,
    ) -> Result<ProjectDataCleanupResult> {
        self.pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = self.pm.open_project_db(id)?;
        retention::cleanup_project_data(&db, &request)
    }

    pub fn get_data_risk_candidates(
        &self,
        schema_version: &str,
        id: &str,
        candidate_type: &str,
        min_review_priority: &str,
        limit: usize,
    ) -> Result<Value> {
        let candidate_type =
            normalize_candidate_type(Some(candidate_type.to_string())).map_err(|error| {
                OpenDogError::InvalidInput(error["error"].as_str().unwrap_or("").to_string())
            })?;
        let min_review_priority = normalize_min_review_priority(Some(
            min_review_priority.to_string(),
        ))
        .map_err(|error| {
            OpenDogError::InvalidInput(error["error"].as_str().unwrap_or("").to_string())
        })?;

        let info = self
            .pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = self.pm.open_project_db(id)?;
        let entries = stats::get_stats(&db)?;
        Ok(project_data_risk_payload(
            schema_version,
            id,
            &candidate_type,
            &min_review_priority,
            limit.max(1),
            &info.root_path,
            &entries,
        ))
    }

    pub fn get_workspace_data_risk_overview(
        &self,
        schema_version: &str,
        candidate_type: &str,
        min_review_priority: &str,
        project_limit: usize,
    ) -> Result<Value> {
        let candidate_type =
            normalize_candidate_type(Some(candidate_type.to_string())).map_err(|error| {
                OpenDogError::InvalidInput(error["error"].as_str().unwrap_or("").to_string())
            })?;
        let min_review_priority = normalize_min_review_priority(Some(
            min_review_priority.to_string(),
        ))
        .map_err(|error| {
            OpenDogError::InvalidInput(error["error"].as_str().unwrap_or("").to_string())
        })?;

        let projects = self.list_projects()?;
        Ok(workspace_data_risk_payload(
            schema_version,
            &projects,
            &candidate_type,
            &min_review_priority,
            project_limit.max(1),
            |item| {
                self.pm
                    .open_project_db(&item.id)
                    .ok()
                    .and_then(|db| stats::get_stats(&db).ok())
                    .unwrap_or_default()
            },
        ))
    }

    fn guidance_projects(&self, project: Option<&str>) -> Result<Vec<ProjectInfo>> {
        let mut projects = self.list_projects()?;
        if let Some(project_id) = project {
            projects.retain(|item| item.id == project_id);
            if projects.is_empty() {
                return Err(OpenDogError::ProjectNotFound(project_id.to_string()));
            }
        }
        Ok(projects)
    }

    fn guidance_project_state(&self, project: &ProjectInfo) -> crate::mcp::ProjectGuidanceData {
        load_project_guidance_data(&self.pm, project)
    }

    pub fn get_agent_guidance(&self, project: Option<&str>, top: usize) -> Result<Value> {
        let projects = self.guidance_projects(project)?;
        Ok(build_agent_guidance_for_projects(
            &projects,
            top.max(1),
            |item| self.guidance_project_state(item),
        ))
    }

    pub fn get_decision_brief(
        &self,
        schema_version: &str,
        project: Option<&str>,
        top: usize,
    ) -> Result<Value> {
        let projects = self.guidance_projects(project)?;
        Ok(build_decision_brief_for_projects(
            schema_version,
            if project.is_some() {
                "project"
            } else {
                "workspace"
            },
            project,
            &projects,
            top.max(1),
            |item| self.guidance_project_state(item),
            |item| {
                self.pm
                    .open_project_db(&item.id)
                    .ok()
                    .and_then(|db| stats::get_stats(&db).ok())
                    .unwrap_or_default()
            },
        ))
    }

    pub fn get_verification_status(&self, id: &str) -> Result<Vec<VerificationRun>> {
        self.pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = self.pm.open_project_db(id)?;
        verification::get_latest_verification_runs(&db)
    }

    pub fn record_verification_result(
        &self,
        id: &str,
        input: RecordVerificationInput,
    ) -> Result<VerificationRun> {
        self.pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = self.pm.open_project_db(id)?;
        verification::record_verification_result(&db, input)
    }

    pub fn execute_verification(
        &self,
        id: &str,
        input: ExecuteVerificationInput,
    ) -> Result<ExecutedVerificationResult> {
        let info = self
            .pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = self.pm.open_project_db(id)?;
        verification::execute_verification_command(&db, &info.root_path, input)
    }
}
