mod config_output;
mod guidance_output;
mod project_output;
mod report_output;
mod verification_output;

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
use serde_json::Value;

pub fn print_created(info: &ProjectInfo) {
    project_output::print_created(info);
}

pub fn print_snapshot_result(id: &str, result: &SnapshotResult) {
    project_output::print_snapshot_result(id, result);
}

pub fn print_stats(id: &str, summary: &ProjectSummary, entries: &[StatsEntry]) {
    project_output::print_stats(id, summary, entries);
}

pub fn print_unused(id: &str, unused: &[StatsEntry]) {
    project_output::print_unused(id, unused);
}

pub fn print_time_window_report(id: &str, report: &TimeWindowReport) {
    report_output::print_time_window_report(id, report);
}

pub fn print_snapshot_comparison(id: &str, comparison: &SnapshotComparison) {
    report_output::print_snapshot_comparison(id, comparison);
}

pub fn print_usage_trends(id: &str, report: &UsageTrendReport) {
    report_output::print_usage_trends(id, report);
}

pub fn print_cleanup_data_result(id: &str, result: &ProjectDataCleanupResult) {
    project_output::print_cleanup_data_result(id, result);
}

pub fn print_project_list(projects: &[ProjectInfo]) {
    project_output::print_project_list(projects);
}

pub fn print_agent_guidance(guidance: &Value) {
    guidance_output::print_agent_guidance(guidance);
}

pub fn print_decision_brief(payload: &Value) {
    guidance_output::print_decision_brief(payload);
}

pub fn print_verification_recorded(id: &str, run: &VerificationRun) {
    verification_output::print_verification_recorded(id, run);
}

pub fn print_verification_status(id: &str, runs: &[VerificationRun]) {
    verification_output::print_verification_status(id, runs);
}

pub fn print_verification_executed(id: &str, result: &ExecutedVerificationResult) {
    verification_output::print_verification_executed(id, result);
}

pub fn print_data_risk(
    id: &str,
    candidate_type: &str,
    min_review_priority: &str,
    rendered: &Value,
    guidance: &Value,
) {
    guidance_output::print_data_risk(id, candidate_type, min_review_priority, rendered, guidance);
}

pub fn print_workspace_data_risk(
    candidate_type: &str,
    min_review_priority: &str,
    project_limit: usize,
    total_registered_projects: usize,
    matched_projects: usize,
    guidance: &Value,
) {
    guidance_output::print_workspace_data_risk(
        candidate_type,
        min_review_priority,
        project_limit,
        total_registered_projects,
        matched_projects,
        guidance,
    );
}

pub fn print_global_config(config: &ProjectConfig) {
    config_output::print_global_config(config);
}

pub fn print_project_config(view: &ProjectConfigView) {
    config_output::print_project_config(view);
}

pub fn print_project_config_update(result: &ProjectConfigUpdateResult) {
    config_output::print_project_config_update(result);
}

pub fn print_global_config_update(result: &GlobalConfigUpdateResult) {
    config_output::print_global_config_update(result);
}

pub fn print_project_config_reload(
    id: &str,
    reload: &ProjectConfigReload,
    effective: &ProjectConfig,
) {
    config_output::print_project_config_reload(id, reload, effective);
}

pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("...{}", &s[s.len() - max + 3..])
    }
}
