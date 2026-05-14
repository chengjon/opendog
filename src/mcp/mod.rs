use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_handler, tool_router};
use serde_json::Value;

use crate::contracts::{
    versioned_error_payload, versioned_project_error_payload, versioned_project_payload,
    MCP_DATA_RISK_V1, MCP_DECISION_BRIEF_V1, MCP_DELETE_PROJECT_V1, MCP_GLOBAL_CONFIG_V1,
    MCP_GUIDANCE_V1, MCP_LIST_PROJECTS_V1, MCP_PROJECT_CONFIG_V1, MCP_RECORD_VERIFICATION_V1,
    MCP_REGISTER_PROJECT_V1, MCP_RUN_VERIFICATION_V1, MCP_SNAPSHOT_COMPARE_V1, MCP_SNAPSHOT_V1,
    MCP_START_MONITOR_V1, MCP_STATS_V1, MCP_STOP_MONITOR_V1, MCP_TIME_WINDOW_REPORT_V1,
    MCP_UNUSED_FILES_V1, MCP_USAGE_TRENDS_V1, MCP_VERIFICATION_STATUS_V1,
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
mod mock_detection;
mod observation;
mod params;
mod payloads;
mod project_guidance;
mod project_handlers;
mod project_recommendation;
mod repo_risk;
mod resource_handlers;
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
use self::config_handlers::{handle_get_global_config, handle_get_project_config};
use self::constraints::{
    build_constraints_boundaries_layer, common_boundary_hints,
    external_truth_boundary_for_top_project, project_readiness_snapshot,
    review_focus_projection_for_top_project,
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
use self::guidance_handlers::handle_get_guidance;
pub(crate) use self::guidance_payload::{
    agent_guidance_payload, latest_verification_runs_for_project, now_unix_secs,
    ProjectGuidanceData, ProjectGuidanceState,
};
use self::guidance_scaffold::{
    base_guidance_layers, default_shell_verification_commands, set_recommended_flow, tool_guidance,
};
pub(crate) use self::mock_detection::detect_mock_data_report;
use self::observation::{
    activity_is_stale, latest_activity_timestamp, latest_verification_timestamp,
    project_observation_layer, snapshot_is_stale, verification_is_stale,
};
pub use self::params::{
    AgentGuidanceParams, CompareSnapshotsParams, DataRiskParams, DecisionBriefParams,
    ExecuteVerificationParams, GlobalConfigParams, GuidanceParams, ObservationRowsParams,
    ProjectIdParams, RecordVerificationParams, RegisterProjectParams, TimeWindowReportParams,
    UsageTrendParams, WorkspaceDataRiskParams,
};
pub(crate) use self::payloads::{
    cleanup_project_data_payload, delete_project_payload, export_project_evidence_payload,
    global_config_payload, list_projects_payload, project_config_payload,
    project_config_reload_payload, project_config_update_payload, register_project_payload,
    snapshot_comparison_payload, snapshot_payload, start_monitor_payload, stats_payload_with_limit,
    stop_monitor_payload, time_window_report_payload, unused_files_payload_with_limit,
    update_global_config_payload, usage_trends_payload, DEFAULT_OBSERVATION_PAYLOAD_LIMIT,
};
#[cfg(test)]
pub(crate) use self::payloads::{stats_payload, unused_files_payload};
use self::project_guidance::{
    register_project_guidance, snapshot_guidance, start_monitor_guidance, stats_guidance,
    unused_guidance,
};
use self::project_handlers::{
    handle_delete_project, handle_list_projects, handle_register_project, handle_start_monitor,
    handle_stop_monitor, handle_take_snapshot,
};
pub(crate) use self::project_recommendation::collect_project_guidance_context;
#[cfg(test)]
use self::project_recommendation::{project_overview, recommend_project_action};
use self::repo_risk::repo_status_risk_layer;
use self::resource_handlers::{mcp_resource_templates, mcp_resources, read_mcp_resource};
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
    error_json_for, open_dog_error_code, validation_error_json,
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

type ToolResult = Result<CallToolResult, rmcp::ErrorData>;

fn structured_tool_output(payload: Json<Value>) -> ToolResult {
    Ok(CallToolResult::structured(payload.0))
}

#[tool_router]
impl OpenDogServer {
    #[tool(
        name = "get_guidance",
        description = "Return the preferred MCP guidance surface for the workspace or a single project. Optional params: project_id to scope one project, top to limit priority queue length, detail (summary|decision, default summary). Example intent: {\"project_id\":\"demo\",\"detail\":\"decision\",\"top\":1}."
    )]
    fn get_guidance(&self, Parameters(params): Parameters<GuidanceParams>) -> ToolResult {
        structured_tool_output(handle_get_guidance(self, params))
    }

    #[tool(
        name = "get_global_config",
        description = "Return OPENDOG global default configuration such as ignore patterns and process whitelist."
    )]
    fn get_global_config(&self) -> ToolResult {
        structured_tool_output(handle_get_global_config(self))
    }

    #[tool(
        name = "get_project_config",
        description = "Return resolved configuration for one project, including global defaults, project overrides, and effective runtime values."
    )]
    fn get_project_config(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> ToolResult {
        structured_tool_output(handle_get_project_config(self, &id))
    }

    #[tool(
        name = "register_project",
        description = "Register an existing project root with a unique ID so OPENDOG can snapshot and monitor it"
    )]
    fn register_project(
        &self,
        Parameters(RegisterProjectParams { id, path }): Parameters<RegisterProjectParams>,
    ) -> ToolResult {
        structured_tool_output(handle_register_project(self, &id, &path))
    }

    #[tool(
        name = "take_snapshot",
        description = "Trigger a full recursive file scan for a project, recording file paths, sizes, and metadata"
    )]
    fn take_snapshot(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> ToolResult {
        structured_tool_output(handle_take_snapshot(self, &id))
    }

    #[tool(
        name = "start_monitor",
        description = "Start file monitoring for a project — begins /proc scanning and inotify change detection"
    )]
    fn start_monitor(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> ToolResult {
        structured_tool_output(handle_start_monitor(self, &id))
    }

    #[tool(
        name = "stop_monitor",
        description = "Stop file monitoring for a project"
    )]
    fn stop_monitor(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> ToolResult {
        structured_tool_output(handle_stop_monitor(self, &id))
    }

    #[tool(
        name = "get_stats",
        description = "Query usage statistics for a project. Required param: id. Optional params: limit (default 50) bounds returned file rows; path_classification filters rows by all/source/infrastructure/backup/project; summary still reports full project counts."
    )]
    fn get_stats(
        &self,
        Parameters(ObservationRowsParams {
            id,
            limit,
            path_classification,
        }): Parameters<ObservationRowsParams>,
    ) -> ToolResult {
        structured_tool_output(handle_get_stats(self, &id, limit, path_classification))
    }

    #[tool(
        name = "get_unused_files",
        description = "List never-accessed files for a project. Required param: id. Optional params: limit (default 50) bounds returned file rows; path_classification filters rows by all/source/infrastructure/backup/project; unused_count still reports the full count."
    )]
    fn get_unused_files(
        &self,
        Parameters(ObservationRowsParams {
            id,
            limit,
            path_classification,
        }): Parameters<ObservationRowsParams>,
    ) -> ToolResult {
        structured_tool_output(handle_get_unused_files(
            self,
            &id,
            limit,
            path_classification,
        ))
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
    ) -> ToolResult {
        structured_tool_output(handle_get_time_window_report(self, &id, window, limit))
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
    ) -> ToolResult {
        structured_tool_output(handle_compare_snapshots(
            self,
            &id,
            base_run_id,
            head_run_id,
            limit,
        ))
    }

    #[tool(
        name = "get_usage_trends",
        description = "Return bucketed usage trends for one project. Required param: id. Optional params: window (24h|7d|30d, default 7d), limit (default 10)."
    )]
    fn get_usage_trends(
        &self,
        Parameters(UsageTrendParams { id, window, limit }): Parameters<UsageTrendParams>,
    ) -> ToolResult {
        structured_tool_output(handle_get_usage_trends(self, &id, window, limit))
    }

    #[tool(
        name = "get_verification_status",
        description = "Return the latest recorded test/lint/build verification results for one project. Required param: id. Example intent: {\"id\":\"demo\"}."
    )]
    fn get_verification_status(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> ToolResult {
        structured_tool_output(handle_get_verification_status(self, &id))
    }

    #[tool(
        name = "record_verification_result",
        description = "Record one verification result so OPENDOG can expose it in its evidence layer. Required params: id, kind, status, command. Optional params: exit_code, summary, source, started_at."
    )]
    fn record_verification_result(
        &self,
        Parameters(params): Parameters<RecordVerificationParams>,
    ) -> ToolResult {
        let (id, input) = params.into_parts();
        structured_tool_output(handle_record_verification_result(self, &id, input))
    }

    #[tool(
        name = "run_verification_command",
        description = "Execute a test/lint/build command in the project root and record the result into OPENDOG evidence. Required params: id, kind, command. Optional param: source. Example intent: {\"id\":\"demo\",\"kind\":\"test\",\"command\":\"cargo test\"}."
    )]
    fn run_verification_command(
        &self,
        Parameters(params): Parameters<ExecuteVerificationParams>,
    ) -> ToolResult {
        let (id, input) = params.into_parts();
        structured_tool_output(handle_run_verification_command(self, &id, input))
    }

    #[tool(
        name = "get_data_risk_candidates",
        description = "Detect mock, fixture, demo, sample, and suspicious hardcoded business-like data candidates for one project. Required param: id. Optional params: candidate_type, min_review_priority, limit."
    )]
    fn get_data_risk_candidates(
        &self,
        Parameters(params): Parameters<DataRiskParams>,
    ) -> ToolResult {
        structured_tool_output(handle_get_data_risk_candidates(self, params))
    }

    #[tool(
        name = "get_workspace_data_risk_overview",
        description = "Aggregate mock and hardcoded-data risk signals across all registered projects so AI can prioritize which project to inspect first. Optional params: candidate_type, min_review_priority, project_limit."
    )]
    fn get_workspace_data_risk_overview(
        &self,
        Parameters(params): Parameters<WorkspaceDataRiskParams>,
    ) -> ToolResult {
        structured_tool_output(handle_get_workspace_data_risk_overview(self, params))
    }

    #[tool(
        name = "list_projects",
        description = "List all registered projects with their status, root path, and database location"
    )]
    fn list_projects(&self) -> ToolResult {
        structured_tool_output(handle_list_projects(self))
    }

    #[tool(
        name = "delete_project",
        description = "Delete a project and all its associated data — database, configuration, stops monitor if running"
    )]
    fn delete_project(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> ToolResult {
        structured_tool_output(handle_delete_project(self, &id))
    }
}

#[tool_handler(router = Self::tool_router())]
impl rmcp::ServerHandler for OpenDogServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo::new(
            rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(rmcp::model::Implementation::from_build_env())
    }

    async fn list_resource_templates(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListResourceTemplatesResult, rmcp::ErrorData> {
        Ok(rmcp::model::ListResourceTemplatesResult {
            resource_templates: mcp_resource_templates(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListResourcesResult, rmcp::ErrorData> {
        Ok(rmcp::model::ListResourcesResult {
            resources: mcp_resources(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        request: rmcp::model::ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ReadResourceResult, rmcp::ErrorData> {
        read_mcp_resource(self, &request.uri)
    }
}

#[cfg(test)]
mod tests;
