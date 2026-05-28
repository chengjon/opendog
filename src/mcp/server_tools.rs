use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use serde_json::Value;

use super::*;

type ToolResult = Result<CallToolResult, rmcp::ErrorData>;

fn structured_tool_output(payload: Json<Value>) -> ToolResult {
    Ok(CallToolResult::structured(payload.0))
}

#[tool_router(vis = "pub(super)")]
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
        name = "get_build_info",
        description = "Return OPENDOG binary version, git hash, build time, and whether a rebuild is needed."
    )]
    fn get_build_info(&self) -> ToolResult {
        structured_tool_output(handle_get_build_info(self))
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
        name = "get_activity_rollups",
        description = "Return daily activity rollups preserved by retention cleanup for one project. Required param: id. Optional params: window (24h|7d|30d, default 30d), limit (default 30)."
    )]
    fn get_activity_rollups(
        &self,
        Parameters(ActivityRollupParams { id, window, limit }): Parameters<ActivityRollupParams>,
    ) -> ToolResult {
        structured_tool_output(handle_get_activity_rollups(self, &id, window, limit))
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
        name = "scan_orphans",
        description = "Classify orphan cleanup candidates for one project using Rust-internal scanners and optional normalized external scanner reports. Required param: id. Optional params: subjects, external_reports, include_internal_scanners, required_scanners, max_age_secs, limit, include_evidence."
    )]
    fn scan_orphans(&self, Parameters(params): Parameters<ScanOrphansParams>) -> ToolResult {
        structured_tool_output(handle_scan_orphans(self, params))
    }

    #[tool(
        name = "verify_deletion_plan",
        description = "Verify whether proposed deletion targets have enough orphan-detection evidence for a human-reviewed deletion plan. Required params: id, targets. Optional params: external_reports, required_project_verification_commands, max_age_secs."
    )]
    fn verify_deletion_plan(
        &self,
        Parameters(params): Parameters<VerifyDeletionPlanParams>,
    ) -> ToolResult {
        structured_tool_output(handle_verify_deletion_plan(self, params))
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

    #[tool(
        name = "create_governance_lane",
        description = "Create a governance work lane for the project. Required params: id, lane_id, title. Optional: description."
    )]
    fn create_governance_lane(
        &self,
        Parameters(params): Parameters<CreateGovernanceLaneParams>,
    ) -> ToolResult {
        let (id, input) = params.into_parts();
        structured_tool_output(handle_create_governance_lane(self, &id, input))
    }

    #[tool(
        name = "upsert_governance_node",
        description = "Create or update a governance node within a lane. Required params: id, lane_id, node_id. State is required on create, optional on update."
    )]
    fn upsert_governance_node(
        &self,
        Parameters(params): Parameters<UpsertGovernanceNodeParams>,
    ) -> ToolResult {
        let (id, input) = params.into_parts();
        structured_tool_output(handle_upsert_governance_node(self, &id, input))
    }

    #[tool(
        name = "get_governance_state",
        description = "Read governance state for a project. Required param: id. Optional params: lane_id, node_id to filter results."
    )]
    fn get_governance_state(
        &self,
        Parameters(params): Parameters<GetGovernanceStateParams>,
    ) -> ToolResult {
        let (id, input) = params.into_parts();
        structured_tool_output(handle_get_governance_state(self, &id, input))
    }

    #[tool(
        name = "close_governance_lane",
        description = "Close, defer, or hard-delete an entire lane and its nodes. Required params: id, lane_id, action (complete|defer|delete)."
    )]
    fn close_governance_lane(
        &self,
        Parameters(params): Parameters<CloseGovernanceLaneParams>,
    ) -> ToolResult {
        let (id, input) = params.into_parts();
        structured_tool_output(handle_close_governance_lane(self, &id, input))
    }
}
