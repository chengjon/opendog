use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

use super::*;

#[tool_router(router = lifecycle_tool_router, vis = "pub(super)")]
impl OpenDogServer {
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
