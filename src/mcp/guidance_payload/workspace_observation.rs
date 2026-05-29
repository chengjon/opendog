use serde_json::{json, Value};

use super::super::{
    guidance_types::{
        DataRiskFocusSummary, WorkspaceObservationAnalysisState, WorkspaceObservationLayer,
        WorkspaceObservationLayerStatus,
    },
    serialization::to_value_or_error,
    WorkspaceCounts,
};

pub(super) struct WorkspaceObservationFacts {
    pub(super) has_failing_verification: bool,
    pub(super) has_mid_operation_repo: bool,
    pub(super) missing_verification_projects: usize,
    pub(super) projects_not_ready_for_cleanup: usize,
    pub(super) projects_not_ready_for_refactor: usize,
    pub(super) projects_with_hardcoded_data_candidates: usize,
    pub(super) projects_missing_snapshot: usize,
    pub(super) projects_with_stale_snapshot: usize,
    pub(super) projects_missing_activity: usize,
    pub(super) projects_with_stale_activity: usize,
    pub(super) projects_missing_verification: usize,
    pub(super) projects_with_stale_verification: usize,
    pub(super) projects_with_storage_maintenance_candidates: u64,
    pub(super) projects_with_vacuum_candidates: u64,
}

impl WorkspaceObservationFacts {
    pub(super) fn from_projects(project_overviews: &[Value], storage_maintenance: &Value) -> Self {
        Self {
            has_failing_verification: project_overviews.iter().any(has_failing_verification),
            has_mid_operation_repo: project_overviews.iter().any(has_mid_operation_repo),
            missing_verification_projects: project_overviews
                .iter()
                .filter(|project| project["verification_evidence"]["status"] == "not_recorded")
                .count(),
            projects_not_ready_for_cleanup: project_overviews
                .iter()
                .filter(|project| !project["safe_for_cleanup"].as_bool().unwrap_or(false))
                .count(),
            projects_not_ready_for_refactor: project_overviews
                .iter()
                .filter(|project| !project["safe_for_refactor"].as_bool().unwrap_or(false))
                .count(),
            projects_with_hardcoded_data_candidates: project_overviews
                .iter()
                .filter(|project| {
                    project["mock_data_summary"]["hardcoded_candidate_count"]
                        .as_u64()
                        .unwrap_or(0)
                        > 0
                })
                .count(),
            projects_missing_snapshot: count_freshness_status(
                project_overviews,
                "snapshot",
                "missing",
            ),
            projects_with_stale_snapshot: count_stale_or_unknown(project_overviews, "snapshot"),
            projects_missing_activity: count_freshness_status(
                project_overviews,
                "activity",
                "missing",
            ),
            projects_with_stale_activity: count_stale_or_unknown(project_overviews, "activity"),
            projects_missing_verification: count_freshness_status(
                project_overviews,
                "verification",
                "missing",
            ),
            projects_with_stale_verification: count_stale_or_unknown(
                project_overviews,
                "verification",
            ),
            projects_with_storage_maintenance_candidates: storage_maintenance
                ["projects_with_candidates"]
                .as_u64()
                .unwrap_or(0),
            projects_with_vacuum_candidates: storage_maintenance["projects_with_vacuum_candidates"]
                .as_u64()
                .unwrap_or(0),
        }
    }

    pub(super) fn analysis_state(
        &self,
        project_count: usize,
        monitoring_count: usize,
    ) -> WorkspaceObservationAnalysisState {
        if project_count == 0 {
            WorkspaceObservationAnalysisState::Empty
        } else if monitoring_count == 0 {
            WorkspaceObservationAnalysisState::InsufficientActivity
        } else if self.projects_with_stale_snapshot > 0
            || self.projects_with_stale_activity > 0
            || self.projects_with_stale_verification > 0
        {
            WorkspaceObservationAnalysisState::Stale
        } else {
            WorkspaceObservationAnalysisState::Ready
        }
    }

    pub(super) fn layer_value(
        &self,
        project_count: usize,
        monitoring_count: usize,
        storage_maintenance: &Value,
        data_risk_focus_summary: &DataRiskFocusSummary,
        notes: &[String],
    ) -> Value {
        to_value_or_error(
            "WorkspaceObservationLayer",
            WorkspaceObservationLayer {
                status: WorkspaceObservationLayerStatus::Available,
                project_count,
                monitoring_count,
                analysis_state: self.analysis_state(project_count, monitoring_count),
                projects_missing_snapshot: self.projects_missing_snapshot,
                projects_with_stale_snapshot: self.projects_with_stale_snapshot,
                projects_missing_activity: self.projects_missing_activity,
                projects_with_stale_activity: self.projects_with_stale_activity,
                projects_missing_verification: self.projects_missing_verification,
                projects_with_stale_verification: self.projects_with_stale_verification,
                projects_with_storage_maintenance_candidates: self
                    .projects_with_storage_maintenance_candidates,
                projects_with_vacuum_candidates: self.projects_with_vacuum_candidates,
                total_storage_reclaimable_bytes: storage_maintenance
                    ["total_approx_reclaimable_bytes"]
                    .clone(),
                data_risk_focus_distribution: data_risk_focus_summary
                    .data_risk_focus_distribution
                    .to_value(),
                projects_requiring_hardcoded_review: json!(
                    data_risk_focus_summary.projects_requiring_hardcoded_review
                ),
                projects_requiring_mock_review: json!(
                    data_risk_focus_summary.projects_requiring_mock_review
                ),
                projects_requiring_mixed_file_review: json!(
                    data_risk_focus_summary.projects_requiring_mixed_file_review
                ),
                notes: notes.to_vec(),
            },
        )
    }

    pub(super) fn workspace_counts(&self) -> WorkspaceCounts {
        WorkspaceCounts {
            projects_not_ready_for_cleanup: self.projects_not_ready_for_cleanup,
            projects_not_ready_for_refactor: self.projects_not_ready_for_refactor,
            projects_with_hardcoded_data_candidates: self.projects_with_hardcoded_data_candidates,
            projects_missing_snapshot: self.projects_missing_snapshot,
            projects_with_stale_snapshot: self.projects_with_stale_snapshot,
            projects_missing_activity: self.projects_missing_activity,
            projects_with_stale_activity: self.projects_with_stale_activity,
            projects_missing_verification: self.projects_missing_verification,
            projects_with_stale_verification: self.projects_with_stale_verification,
            projects_with_storage_maintenance_candidates: self
                .projects_with_storage_maintenance_candidates,
        }
    }
}

fn has_failing_verification(project: &Value) -> bool {
    project["verification_evidence"]["failing_runs"]
        .as_array()
        .map(|runs| !runs.is_empty())
        .unwrap_or(false)
}

fn has_mid_operation_repo(project: &Value) -> bool {
    project["repo_status_risk"]["operation_states"]
        .as_array()
        .map(|states| !states.is_empty())
        .unwrap_or(false)
}

fn count_freshness_status(
    project_overviews: &[Value],
    evidence_name: &str,
    expected_status: &str,
) -> usize {
    project_overviews
        .iter()
        .filter(|project| freshness_status(project, evidence_name) == expected_status)
        .count()
}

fn count_stale_or_unknown(project_overviews: &[Value], evidence_name: &str) -> usize {
    project_overviews
        .iter()
        .filter(|project| {
            matches!(
                freshness_status(project, evidence_name),
                "stale" | "unknown"
            )
        })
        .count()
}

fn freshness_status<'a>(project: &'a Value, evidence_name: &str) -> &'a str {
    project["observation"]["freshness"][evidence_name]["status"]
        .as_str()
        .unwrap_or("")
}
