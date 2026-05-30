use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

use super::*;

#[tool_router(router = analysis_tool_router, vis = "pub(super)")]
impl OpenDogServer {
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
}
