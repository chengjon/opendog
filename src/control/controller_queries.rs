use crate::config::ProjectInfo;
use crate::core::governance::{
    self, CloseLaneInput, CreateLaneInput, GetGovernanceStateInput, GovernanceState,
    UpsertNodeInput, UpsertNodeResult,
};
use crate::core::orphan::{
    self, DeletionPlanInput, DeletionPlanVerification, ScanOrphansInput, ScanOrphansResult,
};
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
    workspace_data_risk_payload, ProjectDataRiskPayloadInput,
};
use crate::storage::queries::{GovernanceLane, StatsEntry, VerificationRun};
use serde_json::Value;

use super::MonitorController;

impl MonitorController {
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
}

#[path = "controller_queries/data_risk.rs"]
mod data_risk;
#[path = "controller_queries/governance_orphans.rs"]
mod governance_orphans;
#[path = "controller_queries/guidance.rs"]
mod guidance;
#[path = "controller_queries/reports.rs"]
mod reports;
#[path = "controller_queries/verification_queries.rs"]
mod verification_queries;
