use super::*;

#[test]
fn tool_guidance_wraps_payload_with_next_steps() {
    let value = tool_guidance(
        "Start monitoring succeeded.",
        &["opendog stats --id demo", "opendog unused --id demo"],
        &["get_stats", "get_unused_files"],
        Some("Use shell commands when you need repository-wide search or tests."),
    );

    assert_eq!(value["summary"], "Start monitoring succeeded.");
    assert_eq!(value["suggested_commands"][0], "opendog stats --id demo");
    assert_eq!(value["next_tools"][1], "get_unused_files");
    assert_eq!(
        value["when_to_use_shell"],
        json!("Use shell commands when you need repository-wide search or tests.")
    );
    assert_eq!(value["schema_version"], MCP_GUIDANCE_V1);
    assert_eq!(
        value["layers"]["repo_status_risk"]["status"],
        json!("not_collected")
    );
    assert_eq!(
        value["layers"]["constraints_boundaries"]["status"],
        json!("available")
    );
    assert!(value["layers"]["constraints_boundaries"]["guardrails"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("activity-derived signals")));
}

#[test]
fn create_project_guidance_uses_multi_step_flow() {
    let value = create_project_guidance();
    assert_eq!(
        value["recommended_flow"][0],
        json!("Project is now registered in OPENDOG.")
    );
    assert!(value["recommended_flow"]
        .as_array()
        .unwrap()
        .iter()
        .any(|step| step.as_str().unwrap().contains("Start monitoring")));
}

#[test]
fn start_monitor_guidance_adapts_when_already_running() {
    let value = start_monitor_guidance(true, false);
    assert!(value["summary"]
        .as_str()
        .unwrap()
        .contains("already active"));
}
