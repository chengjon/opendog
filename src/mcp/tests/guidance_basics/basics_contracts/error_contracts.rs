use super::*;

#[test]
fn daemon_control_unavailable_error_includes_remediation() {
    let Json(value) = error_json_for(
        MCP_GUIDANCE_V1,
        None,
        &OpenDogError::DaemonControlUnavailable,
    );

    assert_eq!(value["schema_version"], MCP_GUIDANCE_V1);
    assert_eq!(value["status"], "error");
    assert_eq!(value["error_code"], "daemon_control_unavailable");
    assert!(value["error"].as_str().unwrap().contains("control socket"));
    assert!(value["remediation"]["socket_path"]
        .as_str()
        .unwrap()
        .contains(".opendog/data/daemon.sock"));
    assert!(value["remediation"]["pid_path"]
        .as_str()
        .unwrap()
        .contains(".opendog/data/daemon.pid"));
}

#[test]
fn project_scoped_error_uses_tool_contract() {
    let Json(value) = error_json_for(
        MCP_STATS_V1,
        Some("demo"),
        &OpenDogError::ProjectNotFound("demo".to_string()),
    );

    assert_eq!(value["schema_version"], MCP_STATS_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["status"], "error");
    assert_eq!(value["error_code"], "project_not_found");
}

#[test]
fn validation_error_uses_tool_contract() {
    let Json(value) = validation_error_json(
        MCP_DATA_RISK_V1,
        Some("demo"),
        "invalid_candidate_type",
        "candidate_type must be one of: all, mock, hardcoded",
    );

    assert_eq!(value["schema_version"], MCP_DATA_RISK_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["status"], "error");
    assert_eq!(value["error_code"], "invalid_candidate_type");
}
