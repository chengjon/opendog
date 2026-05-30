use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

use super::*;

#[tool_router(router = config_tool_router, vis = "pub(super)")]
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
}
