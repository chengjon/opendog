use super::*;

#[test]
fn agent_guidance_params_deserialize_project_scope_and_top() {
    let params: AgentGuidanceParams = serde_json::from_value(json!({
        "project_id": "demo",
        "top": 2
    }))
    .unwrap();

    assert_eq!(params.project_id.as_deref(), Some("demo"));
    assert_eq!(params.top, Some(2));
}

#[test]
fn merged_guidance_params_deserialize_project_scope_top_and_detail() {
    let params: GuidanceParams = serde_json::from_value(json!({
        "project_id": "demo",
        "top": 1,
        "detail": "decision"
    }))
    .unwrap();

    assert_eq!(params.project_id.as_deref(), Some("demo"));
    assert_eq!(params.top, Some(1));
    assert_eq!(params.detail.as_deref(), Some("decision"));
}

#[test]
fn decision_entrypoints_prefer_merged_guidance_tool_for_refresh_paths() {
    let value =
        decision_entrypoints_payload("stabilize_repository_state", Some("demo"), "mcp", "shell");

    assert_eq!(
        value["next_mcp_tools"],
        json!(["get_guidance", "get_verification_status"])
    );
}
