use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

use super::*;

#[tool_router(router = verification_tool_router, vis = "pub(super)")]
impl OpenDogServer {
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
}
