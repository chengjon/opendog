use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::{tool, tool_router};
use serde_json::Value;

use crate::contracts::{
    versioned_error_payload, versioned_project_error_payload, versioned_project_payload,
    MCP_CLEANUP_PROJECT_DATA_V1, MCP_CREATE_PROJECT_V1, MCP_DATA_RISK_V1, MCP_DECISION_BRIEF_V1,
    MCP_DELETE_PROJECT_V1, MCP_EXPORT_PROJECT_EVIDENCE_V1, MCP_GLOBAL_CONFIG_V1, MCP_GUIDANCE_V1,
    MCP_LIST_PROJECTS_V1, MCP_PROJECT_CONFIG_V1, MCP_RECORD_VERIFICATION_V1,
    MCP_RELOAD_PROJECT_CONFIG_V1, MCP_RUN_VERIFICATION_V1, MCP_SNAPSHOT_COMPARE_V1,
    MCP_SNAPSHOT_V1, MCP_START_MONITOR_V1, MCP_STATS_V1, MCP_STOP_MONITOR_V1,
    MCP_TIME_WINDOW_REPORT_V1, MCP_UNUSED_FILES_V1, MCP_UPDATE_GLOBAL_CONFIG_V1,
    MCP_UPDATE_PROJECT_CONFIG_V1, MCP_USAGE_TRENDS_V1, MCP_VERIFICATION_STATUS_V1,
    MCP_WORKSPACE_DATA_RISK_V1,
};

mod analysis_handlers;
mod attention;
mod config_handlers;
mod constraints;
mod data_risk;
mod decision_support;
mod guidance_handlers;
mod guidance_payload;
mod guidance_scaffold;
mod maintenance_handlers;
mod mock_detection;
mod observation;
mod params;
mod payloads;
mod project_guidance;
mod project_handlers;
mod project_recommendation;
mod repo_risk;
pub(crate) mod review_candidates;
mod risk_handlers;
mod server_core;
mod storage_maintenance;
mod strategy;
mod tool_helpers;
mod toolchain;
mod verification_evidence;
mod verification_handlers;
mod workspace_decision;

use self::analysis_handlers::{
    handle_compare_snapshots, handle_get_stats, handle_get_time_window_report,
    handle_get_unused_files, handle_get_usage_trends,
};
use self::attention::{
    enrich_project_overview_with_attention, sort_project_recommendations, workspace_portfolio_layer,
};
use self::config_handlers::{
    handle_get_global_config, handle_get_project_config, handle_reload_project_config,
    handle_update_global_config, handle_update_project_config,
};
use self::constraints::{
    build_constraints_boundaries_layer, common_boundary_hints, project_readiness_snapshot,
};
#[cfg(test)]
use self::data_risk::data_risk_guidance;
use self::data_risk::path_kind_score;
pub(crate) use self::data_risk::{
    normalize_candidate_type, normalize_min_review_priority, project_data_risk_payload,
    review_priority_score, workspace_data_risk_overview_payload, DataCandidate, MockDataReport,
};
use self::decision_support::{
    decision_action_profile, decision_entrypoints_payload, decision_execution_templates,
    decision_risk_profile,
};
use self::guidance_handlers::{handle_get_agent_guidance, handle_get_decision_brief};
pub(crate) use self::guidance_payload::{
    agent_guidance_payload, latest_verification_runs_for_project, now_unix_secs,
    ProjectGuidanceData, ProjectGuidanceState,
};
use self::guidance_scaffold::{
    base_guidance_layers, default_shell_verification_commands, set_recommended_flow, tool_guidance,
};
use self::maintenance_handlers::{handle_cleanup_project_data, handle_export_project_evidence};
pub(crate) use self::mock_detection::detect_mock_data_report;
use self::observation::{
    activity_is_stale, latest_activity_timestamp, latest_verification_timestamp,
    project_observation_layer, snapshot_is_stale, verification_is_stale,
};
pub use self::params::{
    AgentGuidanceParams, CleanupProjectDataParams, CompareSnapshotsParams, CreateProjectParams,
    DataRiskParams, DecisionBriefParams, ExecuteVerificationParams, ExportProjectEvidenceParams,
    GlobalConfigParams, ProjectIdParams, RecordVerificationParams, TimeWindowReportParams,
    UpdateGlobalConfigParams, UpdateProjectConfigParams, UsageTrendParams, WorkspaceDataRiskParams,
};
pub(crate) use self::payloads::{
    cleanup_project_data_payload, create_project_payload, delete_project_payload,
    export_project_evidence_payload, global_config_payload, list_projects_payload,
    project_config_payload, project_config_reload_payload, project_config_update_payload,
    snapshot_comparison_payload, snapshot_payload, start_monitor_payload, stats_payload,
    stop_monitor_payload, time_window_report_payload, unused_files_payload,
    update_global_config_payload, usage_trends_payload,
};
use self::project_guidance::{
    create_project_guidance, snapshot_guidance, start_monitor_guidance, stats_guidance,
    unused_guidance,
};
use self::project_handlers::{
    handle_create_project, handle_delete_project, handle_list_projects, handle_start_monitor,
    handle_stop_monitor, handle_take_snapshot,
};
pub(crate) use self::project_recommendation::collect_project_guidance_context;
#[cfg(test)]
use self::project_recommendation::{project_overview, recommend_project_action};
use self::repo_risk::repo_status_risk_layer;
use self::risk_handlers::{
    handle_get_data_risk_candidates, handle_get_workspace_data_risk_overview,
};
pub use self::server_core::{run_stdio, OpenDogServer};
use self::storage_maintenance::{
    augment_entrypoints_for_storage_maintenance, project_storage_maintenance,
    storage_maintenance_layer,
};
use self::strategy::{
    agent_guidance_recommended_flow, strategy_profile, workspace_strategy_profile,
};
use self::tool_helpers::{
    error_json_for, open_dog_error_code, scoped_projects_or_error, validation_error_json,
};
use self::toolchain::{
    detect_project_commands, project_toolchain_layer, workspace_toolchain_layer,
};
pub(crate) use self::verification_evidence::verification_status_layer;
use self::verification_evidence::{
    record_verification_payload, run_verification_payload, verification_has_failures,
    verification_is_missing, verification_status_payload, workspace_verification_evidence_layer,
};
use self::verification_handlers::{
    handle_get_verification_status, handle_record_verification_result,
    handle_run_verification_command,
};
pub(crate) use self::workspace_decision::{
    collect_workspace_data_risk_summaries, decision_brief_payload, workspace_data_risk_payload,
};

#[tool_router(server_handler)]
impl OpenDogServer {
    #[tool(
        name = "get_agent_guidance",
        description = "Return AI guidance for the workspace or a single project. Optional params: project_id to scope one project, top to limit priority queue length. Example intent: {\"project_id\":\"demo\",\"top\":3}."
    )]
    fn get_agent_guidance(
        &self,
        Parameters(AgentGuidanceParams { project_id, top }): Parameters<AgentGuidanceParams>,
    ) -> Json<Value> {
        handle_get_agent_guidance(self, AgentGuidanceParams { project_id, top })
    }

    #[tool(
        name = "get_decision_brief",
        description = "Return one AI-facing decision envelope with recommended next action, 8 OPENDOG layers, and suggested MCP/CLI entrypoints. Optional params: project_id to scope one project, top to limit priority queue length. Example intent: {\"project_id\":\"demo\",\"top\":1}."
    )]
    fn get_decision_brief(
        &self,
        Parameters(DecisionBriefParams { project_id, top }): Parameters<DecisionBriefParams>,
    ) -> Json<Value> {
        handle_get_decision_brief(self, DecisionBriefParams { project_id, top })
    }

    #[tool(
        name = "get_global_config",
        description = "Return OPENDOG global default configuration such as ignore patterns and process whitelist."
    )]
    fn get_global_config(&self) -> Json<Value> {
        handle_get_global_config(self)
    }

    #[tool(
        name = "get_project_config",
        description = "Return resolved configuration for one project, including global defaults, project overrides, and effective runtime values."
    )]
    fn get_project_config(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        handle_get_project_config(self, &id)
    }

    #[tool(
        name = "update_global_config",
        description = "Update OPENDOG global default configuration. Optional params: ignore_patterns, process_whitelist."
    )]
    fn update_global_config(
        &self,
        Parameters(params): Parameters<UpdateGlobalConfigParams>,
    ) -> Json<Value> {
        handle_update_global_config(self, params.into_patch())
    }

    #[tool(
        name = "update_project_config",
        description = "Update per-project configuration overrides. Required param: id. Optional params: ignore_patterns, process_whitelist, inherit_ignore_patterns, inherit_process_whitelist."
    )]
    fn update_project_config(
        &self,
        Parameters(params): Parameters<UpdateProjectConfigParams>,
    ) -> Json<Value> {
        let (id, patch) = params.into_parts();
        handle_update_project_config(self, &id, patch)
    }

    #[tool(
        name = "reload_project_config",
        description = "Reload persisted configuration into a running project monitor without restarting the daemon. Required param: id."
    )]
    fn reload_project_config(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        handle_reload_project_config(self, &id)
    }

    #[tool(
        name = "export_project_evidence",
        description = "Export project evidence rows into a portable JSON or CSV artifact. Required params: id, format, output_path. Optional params: view (stats|unused|core, default stats), min_access_count for core view."
    )]
    fn export_project_evidence(
        &self,
        Parameters(ExportProjectEvidenceParams {
            id,
            format,
            view,
            output_path,
            min_access_count,
        }): Parameters<ExportProjectEvidenceParams>,
    ) -> Json<Value> {
        handle_export_project_evidence(self, &id, format, view, output_path, min_access_count)
    }

    #[tool(
        name = "create_project",
        description = "Register a new project with a unique ID and root directory path for file monitoring"
    )]
    fn create_project(
        &self,
        Parameters(CreateProjectParams { id, path }): Parameters<CreateProjectParams>,
    ) -> Json<Value> {
        handle_create_project(self, &id, &path)
    }

    #[tool(
        name = "take_snapshot",
        description = "Trigger a full recursive file scan for a project, recording file paths, sizes, and metadata"
    )]
    fn take_snapshot(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        handle_take_snapshot(self, &id)
    }

    #[tool(
        name = "start_monitor",
        description = "Start file monitoring for a project — begins /proc scanning and inotify change detection"
    )]
    fn start_monitor(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        handle_start_monitor(self, &id)
    }

    #[tool(
        name = "stop_monitor",
        description = "Stop file monitoring for a project"
    )]
    fn stop_monitor(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        handle_stop_monitor(self, &id)
    }

    #[tool(
        name = "get_stats",
        description = "Query usage statistics for a project — per-file access count, estimated duration, modifications, last access"
    )]
    fn get_stats(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        handle_get_stats(self, &id)
    }

    #[tool(
        name = "get_unused_files",
        description = "List never-accessed files for a project — candidates for cleanup or removal review"
    )]
    fn get_unused_files(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        handle_get_unused_files(self, &id)
    }

    #[tool(
        name = "get_time_window_report",
        description = "Return time-windowed activity statistics for one project. Required param: id. Optional params: window (24h|7d|30d, default 24h), limit (default 10)."
    )]
    fn get_time_window_report(
        &self,
        Parameters(TimeWindowReportParams { id, window, limit }): Parameters<
            TimeWindowReportParams,
        >,
    ) -> Json<Value> {
        handle_get_time_window_report(self, &id, window, limit)
    }

    #[tool(
        name = "compare_snapshots",
        description = "Compare two snapshot runs for one project. Required param: id. Optional params: base_run_id and head_run_id together; when omitted, OPENDOG compares the latest two runs. Optional limit defaults to 20."
    )]
    fn compare_snapshots(
        &self,
        Parameters(CompareSnapshotsParams {
            id,
            base_run_id,
            head_run_id,
            limit,
        }): Parameters<CompareSnapshotsParams>,
    ) -> Json<Value> {
        handle_compare_snapshots(self, &id, base_run_id, head_run_id, limit)
    }

    #[tool(
        name = "get_usage_trends",
        description = "Return bucketed usage trends for one project. Required param: id. Optional params: window (24h|7d|30d, default 7d), limit (default 10)."
    )]
    fn get_usage_trends(
        &self,
        Parameters(UsageTrendParams { id, window, limit }): Parameters<UsageTrendParams>,
    ) -> Json<Value> {
        handle_get_usage_trends(self, &id, window, limit)
    }

    #[tool(
        name = "cleanup_project_data",
        description = "Selectively delete retained OPENDOG evidence for one project without touching source files. Required params: id, scope. Optional params: older_than_days, keep_snapshot_runs, vacuum, dry_run (default true)."
    )]
    fn cleanup_project_data(
        &self,
        Parameters(CleanupProjectDataParams {
            id,
            scope,
            older_than_days,
            keep_snapshot_runs,
            vacuum,
            dry_run,
        }): Parameters<CleanupProjectDataParams>,
    ) -> Json<Value> {
        handle_cleanup_project_data(
            self,
            &id,
            scope,
            older_than_days,
            keep_snapshot_runs,
            vacuum,
            dry_run,
        )
    }

    #[tool(
        name = "get_verification_status",
        description = "Return the latest recorded test/lint/build verification results for one project. Required param: id. Example intent: {\"id\":\"demo\"}."
    )]
    fn get_verification_status(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        handle_get_verification_status(self, &id)
    }

    #[tool(
        name = "record_verification_result",
        description = "Record one verification result so OPENDOG can expose it in its evidence layer. Required params: id, kind, status, command. Optional params: exit_code, summary, source, started_at."
    )]
    fn record_verification_result(
        &self,
        Parameters(params): Parameters<RecordVerificationParams>,
    ) -> Json<Value> {
        let (id, input) = params.into_parts();
        handle_record_verification_result(self, &id, input)
    }

    #[tool(
        name = "run_verification_command",
        description = "Execute a test/lint/build command in the project root and record the result into OPENDOG evidence. Required params: id, kind, command. Optional param: source. Example intent: {\"id\":\"demo\",\"kind\":\"test\",\"command\":\"cargo test\"}."
    )]
    fn run_verification_command(
        &self,
        Parameters(params): Parameters<ExecuteVerificationParams>,
    ) -> Json<Value> {
        let (id, input) = params.into_parts();
        handle_run_verification_command(self, &id, input)
    }

    #[tool(
        name = "get_data_risk_candidates",
        description = "Detect mock, fixture, demo, sample, and suspicious hardcoded business-like data candidates for one project. Required param: id. Optional params: candidate_type, min_review_priority, limit."
    )]
    fn get_data_risk_candidates(
        &self,
        Parameters(params): Parameters<DataRiskParams>,
    ) -> Json<Value> {
        handle_get_data_risk_candidates(self, params)
    }

    #[tool(
        name = "get_workspace_data_risk_overview",
        description = "Aggregate mock and hardcoded-data risk signals across all registered projects so AI can prioritize which project to inspect first. Optional params: candidate_type, min_review_priority, project_limit."
    )]
    fn get_workspace_data_risk_overview(
        &self,
        Parameters(params): Parameters<WorkspaceDataRiskParams>,
    ) -> Json<Value> {
        handle_get_workspace_data_risk_overview(self, params)
    }

    #[tool(
        name = "list_projects",
        description = "List all registered projects with their status, root path, and database location"
    )]
    fn list_projects(&self) -> Json<Value> {
        handle_list_projects(self)
    }

    #[tool(
        name = "delete_project",
        description = "Delete a project and all its associated data — database, configuration, stops monitor if running"
    )]
    fn delete_project(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        handle_delete_project(self, &id)
    }
}

#[cfg(test)]
mod tests;
