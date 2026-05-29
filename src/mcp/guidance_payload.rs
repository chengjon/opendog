use serde_json::{json, Value};

use crate::contracts::MCP_GUIDANCE_V1;
use crate::core::verification;
use crate::storage::database::Database;
use crate::storage::queries::{StatsEntry, VerificationRun};

use super::{
    agent_guidance_recommended_flow, base_guidance_layers, build_constraints_boundaries_layer,
    default_shell_verification_commands, external_truth_boundary_for_top_project,
    guidance_types::{
        ExecutionEvidencePriority, ExecutionStrategyLayer, ExecutionStrategyLayerStatus,
        RepoRiskPreferredTool, RepoRiskStrategyMode, WorkspaceObservationAnalysisState,
        WorkspaceObservationLayer, WorkspaceObservationLayerStatus,
    },
    review_focus_projection_for_top_project,
    serialization::to_value_or_error,
    sort_project_recommendations, storage_maintenance_layer, workspace_portfolio_layer,
    workspace_strategy_profile, workspace_toolchain_layer, workspace_verification_evidence_layer,
    WorkspaceCounts,
};

mod execution_strategy;

use execution_strategy::{
    execution_strategy_data_risk_focus_summary, execution_strategy_observation_summary,
    execution_strategy_repo_risk_coupling, execution_strategy_repo_truth_summary,
    execution_strategy_stabilization_summary, execution_strategy_verification_summary,
};
#[derive(Debug, Clone, Default)]
pub(crate) struct ProjectGuidanceState {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) root_path: std::path::PathBuf,
    pub(crate) total_files: i64,
    pub(crate) accessed_files: i64,
    pub(crate) unused_files: i64,
    pub(crate) latest_snapshot_captured_at: Option<String>,
    pub(crate) latest_activity_at: Option<String>,
    pub(crate) latest_verification_at: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ProjectGuidanceData {
    pub(crate) total_files: i64,
    pub(crate) accessed_files: i64,
    pub(crate) unused_files: i64,
    pub(crate) latest_snapshot_captured_at: Option<String>,
    pub(crate) verification_runs: Vec<VerificationRun>,
    pub(crate) stats_entries: Vec<StatsEntry>,
}

pub(crate) fn latest_verification_runs_for_project(db: &Database) -> Vec<VerificationRun> {
    verification::get_latest_verification_runs(db).unwrap_or_default()
}

pub(crate) fn now_unix_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn string_field(value: &Value, fallback: &str) -> String {
    value.as_str().unwrap_or(fallback).to_string()
}

fn string_list_field(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
pub(crate) fn default_governance_layer() -> Value {
    serde_json::json!({
        "has_governance_state": false,
        "project_governance": [],
        "workspace_summary": {
            "total_active_lanes": 0,
            "total_active_nodes": 0,
            "projects_with_governance": 0,
            "projects_without_governance": 0,
        }
    })
}

pub(crate) fn agent_guidance_payload(
    project_count: usize,
    monitoring_count: usize,
    monitored_projects: &[String],
    notes: &[String],
    project_recommendations: &[Value],
    project_overviews: &[Value],
    governance: Value,
) -> Value {
    let has_failing_verification = project_overviews.iter().any(|p| {
        p["verification_evidence"]["failing_runs"]
            .as_array()
            .map(|runs| !runs.is_empty())
            .unwrap_or(false)
    });
    let has_mid_operation_repo = project_overviews.iter().any(|p| {
        p["repo_status_risk"]["operation_states"]
            .as_array()
            .map(|states| !states.is_empty())
            .unwrap_or(false)
    });
    let missing_verification_projects = project_overviews
        .iter()
        .filter(|p| p["verification_evidence"]["status"] == "not_recorded")
        .count();
    let projects_not_ready_for_cleanup = project_overviews
        .iter()
        .filter(|p| !p["safe_for_cleanup"].as_bool().unwrap_or(false))
        .count();
    let projects_not_ready_for_refactor = project_overviews
        .iter()
        .filter(|p| !p["safe_for_refactor"].as_bool().unwrap_or(false))
        .count();
    let projects_with_hardcoded_data = project_overviews
        .iter()
        .filter(|p| {
            p["mock_data_summary"]["hardcoded_candidate_count"]
                .as_u64()
                .unwrap_or(0)
                > 0
        })
        .count();
    let projects_missing_snapshot = project_overviews
        .iter()
        .filter(|p| p["observation"]["freshness"]["snapshot"]["status"] == "missing")
        .count();
    let projects_with_stale_snapshot = project_overviews
        .iter()
        .filter(|p| {
            matches!(
                p["observation"]["freshness"]["snapshot"]["status"]
                    .as_str()
                    .unwrap_or(""),
                "stale" | "unknown"
            )
        })
        .count();
    let projects_missing_activity = project_overviews
        .iter()
        .filter(|p| p["observation"]["freshness"]["activity"]["status"] == "missing")
        .count();
    let projects_with_stale_activity = project_overviews
        .iter()
        .filter(|p| {
            matches!(
                p["observation"]["freshness"]["activity"]["status"]
                    .as_str()
                    .unwrap_or(""),
                "stale" | "unknown"
            )
        })
        .count();
    let projects_missing_verification = project_overviews
        .iter()
        .filter(|p| p["observation"]["freshness"]["verification"]["status"] == "missing")
        .count();
    let projects_with_stale_verification = project_overviews
        .iter()
        .filter(|p| {
            matches!(
                p["observation"]["freshness"]["verification"]["status"]
                    .as_str()
                    .unwrap_or(""),
                "stale" | "unknown"
            )
        })
        .count();
    let storage_maintenance = storage_maintenance_layer(project_overviews);
    let projects_with_storage_maintenance_candidates = storage_maintenance
        ["projects_with_candidates"]
        .as_u64()
        .unwrap_or(0);
    let projects_with_vacuum_candidates = storage_maintenance["projects_with_vacuum_candidates"]
        .as_u64()
        .unwrap_or(0);
    let workspace_strategy = workspace_strategy_profile(
        project_count,
        monitoring_count,
        has_failing_verification,
        has_mid_operation_repo,
        missing_verification_projects,
    );
    let sorted_project_recommendations =
        sort_project_recommendations(project_recommendations, project_overviews);
    let verification_summary =
        execution_strategy_verification_summary(&sorted_project_recommendations);
    let observation_summary =
        execution_strategy_observation_summary(&sorted_project_recommendations);
    let repo_truth_summary = execution_strategy_repo_truth_summary(&sorted_project_recommendations);
    let stabilization_summary =
        execution_strategy_stabilization_summary(&sorted_project_recommendations);
    let data_risk_focus_summary = execution_strategy_data_risk_focus_summary(project_overviews);
    let risk_strategy_coupling = execution_strategy_repo_risk_coupling(
        &sorted_project_recommendations,
        project_overviews,
        &workspace_strategy,
    );
    let risk_strategy_coupling_value = risk_strategy_coupling.to_value();
    let external_truth_boundary =
        external_truth_boundary_for_top_project(sorted_project_recommendations.first());
    let review_focus_projection =
        review_focus_projection_for_top_project(sorted_project_recommendations.first());
    let recommended_flow = agent_guidance_recommended_flow(
        project_count,
        monitoring_count,
        sorted_project_recommendations.first(),
        &workspace_strategy,
        Some(&risk_strategy_coupling_value),
    );

    let mut value = json!({
        "guidance": {
            "schema_version": MCP_GUIDANCE_V1,
            "project_count": project_count,
            "monitoring_count": monitoring_count,
            "monitored_projects": monitored_projects,
            "recommended_flow": recommended_flow,
            "when_to_use_opendog": [
                "Use opendog MCP tools when deciding what files are active, unused, or currently monitored.",
                "Use opendog before cleanup or refactoring decisions that depend on real file activity."
            ],
            "when_to_use_shell": [
                "Use `rg` for repository-wide code search and symbol discovery.",
                "Use `git status`, `git diff`, and `git log` for change inspection.",
                "Use project-native test commands such as `cargo test`, `cargo clippy`, `npm test`, or `pytest` to verify behavior."
            ],
            "example_commands": [
                "opendog list",
                "opendog start --id <project>",
                "opendog stats --id <project>",
                "rg \"<pattern>\" .",
                "cargo test"
            ],
            "notes": notes,
            "project_recommendations": sorted_project_recommendations,
            "layers": base_guidance_layers(),
        }
    });
    let analysis_state = if project_count == 0 {
        WorkspaceObservationAnalysisState::Empty
    } else if monitoring_count == 0 {
        WorkspaceObservationAnalysisState::InsufficientActivity
    } else if projects_with_stale_snapshot > 0
        || projects_with_stale_activity > 0
        || projects_with_stale_verification > 0
    {
        WorkspaceObservationAnalysisState::Stale
    } else {
        WorkspaceObservationAnalysisState::Ready
    };

    value["guidance"]["layers"]["workspace_observation"] = to_value_or_error(
        "WorkspaceObservationLayer",
        WorkspaceObservationLayer {
            status: WorkspaceObservationLayerStatus::Available,
            project_count,
            monitoring_count,
            analysis_state,
            projects_missing_snapshot,
            projects_with_stale_snapshot,
            projects_missing_activity,
            projects_with_stale_activity,
            projects_missing_verification,
            projects_with_stale_verification,
            projects_with_storage_maintenance_candidates,
            projects_with_vacuum_candidates,
            total_storage_reclaimable_bytes: storage_maintenance["total_approx_reclaimable_bytes"]
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
    );
    value["guidance"]["layers"]["execution_strategy"] =
        to_value_or_error("ExecutionStrategyLayer", ExecutionStrategyLayer {
            status: ExecutionStrategyLayerStatus::Available,
            recommended_flow: string_list_field(&recommended_flow),
            project_recommendations: sorted_project_recommendations.clone(),
            global_strategy_mode: RepoRiskStrategyMode::from_mode(&string_field(
                &workspace_strategy["global_strategy_mode"],
                "current_strategy",
            )),
            preferred_primary_tool: RepoRiskPreferredTool::from_tool(&string_field(
                &workspace_strategy["preferred_primary_tool"],
                "current_tool",
            )),
            preferred_secondary_tool: RepoRiskPreferredTool::from_tool(&string_field(
                &workspace_strategy["preferred_secondary_tool"],
                "shell",
            )),
            evidence_priority: string_list_field(&workspace_strategy["evidence_priority"])
                .into_iter()
                .map(|priority| ExecutionEvidencePriority::from_priority(&priority))
                .collect(),
            risk_strategy_coupling,
            external_truth_boundary,
            review_focus_projection,
            when_to_use_opendog: vec![
                "Choose OPENDOG when deciding which files are active, unused, or should be reviewed first.",
            ],
            when_to_use_shell: vec![
                "Choose shell commands for git state, diffs, tests, lint, and builds.",
            ],
            guardrails: vec![
                "Do not recommend broad cleanup or refactor work while recorded verification is failing.",
                "Do not recommend broad changes while a repository is mid-merge, rebase, cherry-pick, or bisect.",
                "When verification is missing, prefer running and recording test/lint/build evidence before high-risk edits.",
                "When snapshot, activity, or verification evidence is stale, refresh it before trusting OPENDOG-driven sequencing.",
            ],
            projects_not_ready_for_cleanup,
            projects_not_ready_for_refactor,
            projects_with_hardcoded_data_candidates: projects_with_hardcoded_data,
            projects_missing_snapshot,
            projects_with_stale_snapshot,
            projects_missing_activity,
            projects_with_stale_activity,
            projects_missing_verification,
            projects_with_stale_verification,
            projects_with_storage_maintenance_candidates,
            projects_with_vacuum_candidates,
            review_opendog_retention_before_large_cleanup:
                projects_with_storage_maintenance_candidates > 0,
            recommend_manual_review_for_hardcoded_data: projects_with_hardcoded_data > 0,
            data_risk_focus_distribution: data_risk_focus_summary
                .data_risk_focus_distribution
                .clone(),
            projects_requiring_hardcoded_review: data_risk_focus_summary
                .projects_requiring_hardcoded_review,
            projects_requiring_mock_review: data_risk_focus_summary.projects_requiring_mock_review,
            projects_requiring_mixed_file_review: data_risk_focus_summary
                .projects_requiring_mixed_file_review,
            projects_requiring_monitor_start: observation_summary.projects_requiring_monitor_start,
            projects_requiring_snapshot_refresh: observation_summary
                .projects_requiring_snapshot_refresh,
            projects_requiring_activity_generation: observation_summary
                .projects_requiring_activity_generation,
            projects_with_repo_truth_gaps: repo_truth_summary.projects_with_repo_truth_gaps,
            repo_truth_gap_distribution: repo_truth_summary.repo_truth_gap_distribution,
            mandatory_shell_check_examples: repo_truth_summary.mandatory_shell_check_examples,
            projects_requiring_verification_run: verification_summary.projects_requiring_verification_run,
            projects_requiring_failing_verification_repair: verification_summary
                .projects_requiring_failing_verification_repair,
            projects_requiring_repo_stabilization: stabilization_summary
                .projects_requiring_repo_stabilization,
            repo_stabilization_priority_projects: stabilization_summary
                .repo_stabilization_priority_projects,
        });
    value["guidance"]["layers"]["multi_project_portfolio"] = to_value_or_error(
        "WorkspacePortfolioLayer",
        workspace_portfolio_layer(
            project_overviews,
            monitoring_count,
            monitored_projects,
            sorted_project_recommendations,
            projects_with_hardcoded_data,
        ),
    );
    value["guidance"]["layers"]["storage_maintenance"] = storage_maintenance;
    value["guidance"]["layers"]["verification_evidence"] =
        workspace_verification_evidence_layer(project_overviews, project_count, monitoring_count);
    value["guidance"]["layers"]["project_toolchain"] = workspace_toolchain_layer(project_overviews);
    value["guidance"]["layers"]["constraints_boundaries"] = build_constraints_boundaries_layer(
        None,
        None,
        vec!["This workspace summary is based on registered projects and OPENDOG monitoring state."
            .to_string()],
        vec![
            "Priority recommendations are advisory and should be combined with repository-specific verification."
                .to_string(),
        ],
        vec![
            "Verification evidence is only as current as the latest recorded test/lint/build results."
                .to_string(),
        ],
        default_shell_verification_commands(),
        Some(WorkspaceCounts {
            projects_not_ready_for_cleanup,
            projects_not_ready_for_refactor,
            projects_with_hardcoded_data_candidates: projects_with_hardcoded_data,
            projects_missing_snapshot,
            projects_with_stale_snapshot,
            projects_missing_activity,
            projects_with_stale_activity,
            projects_missing_verification,
            projects_with_stale_verification,
            projects_with_storage_maintenance_candidates,
        }),
    );
    value["guidance"]["layers"]["governance"] = governance;
    value
}

#[cfg(test)]
mod tests;
