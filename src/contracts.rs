use serde_json::{Map, Value};

pub const PORTABLE_PROJECT_EXPORT_V1: &str = "opendog.export.project-evidence.v1";

pub const MCP_GUIDANCE_V1: &str = "opendog.mcp.guidance.v1";
pub const MCP_DECISION_BRIEF_V1: &str = "opendog.mcp.decision-brief.v1";
pub const MCP_CREATE_PROJECT_V1: &str = "opendog.mcp.create-project.v1";
pub const MCP_START_MONITOR_V1: &str = "opendog.mcp.start-monitor.v1";
pub const MCP_STOP_MONITOR_V1: &str = "opendog.mcp.stop-monitor.v1";
pub const MCP_LIST_PROJECTS_V1: &str = "opendog.mcp.list-projects.v1";
pub const MCP_DELETE_PROJECT_V1: &str = "opendog.mcp.delete-project.v1";
pub const MCP_SNAPSHOT_V1: &str = "opendog.mcp.snapshot.v1";
pub const MCP_STATS_V1: &str = "opendog.mcp.stats.v1";
pub const MCP_UNUSED_FILES_V1: &str = "opendog.mcp.unused-files.v1";
pub const MCP_TIME_WINDOW_REPORT_V1: &str = "opendog.mcp.time-window-report.v1";
pub const MCP_SNAPSHOT_COMPARE_V1: &str = "opendog.mcp.snapshot-compare.v1";
pub const MCP_USAGE_TRENDS_V1: &str = "opendog.mcp.usage-trends.v1";
pub const MCP_EXPORT_PROJECT_EVIDENCE_V1: &str = "opendog.mcp.export-project-evidence.v1";
pub const MCP_GLOBAL_CONFIG_V1: &str = "opendog.mcp.global-config.v1";
pub const MCP_PROJECT_CONFIG_V1: &str = "opendog.mcp.project-config.v1";
pub const MCP_UPDATE_GLOBAL_CONFIG_V1: &str = "opendog.mcp.update-global-config.v1";
pub const MCP_UPDATE_PROJECT_CONFIG_V1: &str = "opendog.mcp.update-project-config.v1";
pub const MCP_RELOAD_PROJECT_CONFIG_V1: &str = "opendog.mcp.reload-project-config.v1";
pub const MCP_DATA_RISK_V1: &str = "opendog.mcp.data-risk.v1";
pub const MCP_WORKSPACE_DATA_RISK_V1: &str = "opendog.mcp.workspace-data-risk.v1";
pub const MCP_VERIFICATION_STATUS_V1: &str = "opendog.mcp.verification-status.v1";
pub const MCP_RECORD_VERIFICATION_V1: &str = "opendog.mcp.record-verification.v1";
pub const MCP_RUN_VERIFICATION_V1: &str = "opendog.mcp.run-verification.v1";
pub const MCP_CLEANUP_PROJECT_DATA_V1: &str = "opendog.mcp.cleanup-project-data.v1";

pub const CLI_GLOBAL_CONFIG_V1: &str = "opendog.cli.global-config.v1";
pub const CLI_PROJECT_CONFIG_V1: &str = "opendog.cli.project-config.v1";
pub const CLI_UPDATE_GLOBAL_CONFIG_V1: &str = "opendog.cli.update-global-config.v1";
pub const CLI_UPDATE_PROJECT_CONFIG_V1: &str = "opendog.cli.update-project-config.v1";
pub const CLI_RELOAD_PROJECT_CONFIG_V1: &str = "opendog.cli.reload-project-config.v1";
pub const CLI_DATA_RISK_V1: &str = "opendog.cli.data-risk.v1";
pub const CLI_EXPORT_PROJECT_EVIDENCE_V1: &str = "opendog.cli.export-project-evidence.v1";
pub const CLI_TIME_WINDOW_REPORT_V1: &str = "opendog.cli.time-window-report.v1";
pub const CLI_SNAPSHOT_COMPARE_V1: &str = "opendog.cli.snapshot-compare.v1";
pub const CLI_USAGE_TRENDS_V1: &str = "opendog.cli.usage-trends.v1";
pub const CLI_DECISION_BRIEF_V1: &str = "opendog.cli.decision-brief.v1";
pub const CLI_WORKSPACE_DATA_RISK_V1: &str = "opendog.cli.workspace-data-risk.v1";
pub const CLI_RECORD_VERIFICATION_V1: &str = "opendog.cli.record-verification.v1";
pub const CLI_VERIFICATION_STATUS_V1: &str = "opendog.cli.verification-status.v1";
pub const CLI_RUN_VERIFICATION_V1: &str = "opendog.cli.run-verification.v1";
pub const CLI_CLEANUP_PROJECT_DATA_V1: &str = "opendog.cli.cleanup-project-data.v1";

pub fn versioned_payload<I>(schema_version: &str, fields: I) -> Value
where
    I: IntoIterator<Item = (&'static str, Value)>,
{
    let mut map = Map::new();
    map.insert(
        "schema_version".to_string(),
        Value::String(schema_version.to_string()),
    );
    for (key, value) in fields {
        map.insert(key.to_string(), value);
    }
    Value::Object(map)
}

pub fn versioned_project_payload<I>(schema_version: &str, project_id: &str, fields: I) -> Value
where
    I: IntoIterator<Item = (&'static str, Value)>,
{
    let mut map = Map::new();
    map.insert(
        "schema_version".to_string(),
        Value::String(schema_version.to_string()),
    );
    map.insert(
        "project_id".to_string(),
        Value::String(project_id.to_string()),
    );
    for (key, value) in fields {
        map.insert(key.to_string(), value);
    }
    Value::Object(map)
}

pub fn versioned_error_payload<I>(
    schema_version: &str,
    error_code: &str,
    error_message: &str,
    fields: I,
) -> Value
where
    I: IntoIterator<Item = (&'static str, Value)>,
{
    let mut map = Map::new();
    map.insert(
        "schema_version".to_string(),
        Value::String(schema_version.to_string()),
    );
    map.insert("status".to_string(), Value::String("error".to_string()));
    map.insert(
        "error_code".to_string(),
        Value::String(error_code.to_string()),
    );
    map.insert(
        "error".to_string(),
        Value::String(error_message.to_string()),
    );
    for (key, value) in fields {
        map.insert(key.to_string(), value);
    }
    Value::Object(map)
}

pub fn versioned_project_error_payload<I>(
    schema_version: &str,
    project_id: &str,
    error_code: &str,
    error_message: &str,
    fields: I,
) -> Value
where
    I: IntoIterator<Item = (&'static str, Value)>,
{
    let mut map = Map::new();
    map.insert(
        "schema_version".to_string(),
        Value::String(schema_version.to_string()),
    );
    map.insert(
        "project_id".to_string(),
        Value::String(project_id.to_string()),
    );
    map.insert("status".to_string(), Value::String("error".to_string()));
    map.insert(
        "error_code".to_string(),
        Value::String(error_code.to_string()),
    );
    map.insert(
        "error".to_string(),
        Value::String(error_message.to_string()),
    );
    for (key, value) in fields {
        map.insert(key.to_string(), value);
    }
    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::{
        versioned_error_payload, versioned_payload, versioned_project_error_payload,
        versioned_project_payload,
    };
    use serde_json::json;

    #[test]
    fn versioned_payload_builds_top_level_contract() {
        let value = versioned_payload("demo.schema.v1", [("status", json!("ok"))]);
        assert_eq!(value["schema_version"], "demo.schema.v1");
        assert_eq!(value["status"], "ok");
    }

    #[test]
    fn versioned_project_payload_builds_project_contract() {
        let value =
            versioned_project_payload("demo.project.v1", "alpha", [("status", json!("ok"))]);
        assert_eq!(value["schema_version"], "demo.project.v1");
        assert_eq!(value["project_id"], "alpha");
        assert_eq!(value["status"], "ok");
    }

    #[test]
    fn versioned_error_payload_builds_error_contract() {
        let value = versioned_error_payload(
            "demo.error.v1",
            "invalid_input",
            "broken",
            [("field", json!("candidate_type"))],
        );
        assert_eq!(value["schema_version"], "demo.error.v1");
        assert_eq!(value["status"], "error");
        assert_eq!(value["error_code"], "invalid_input");
        assert_eq!(value["error"], "broken");
        assert_eq!(value["field"], "candidate_type");
    }

    #[test]
    fn versioned_project_error_payload_builds_project_error_contract() {
        let value = versioned_project_error_payload(
            "demo.project-error.v1",
            "alpha",
            "not_found",
            "missing",
            [("detail", json!("x"))],
        );
        assert_eq!(value["schema_version"], "demo.project-error.v1");
        assert_eq!(value["project_id"], "alpha");
        assert_eq!(value["status"], "error");
        assert_eq!(value["error_code"], "not_found");
        assert_eq!(value["error"], "missing");
        assert_eq!(value["detail"], "x");
    }
}
