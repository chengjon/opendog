#[test]
fn mcp_tool_surface_excludes_operator_only_mutation_tools() {
    let mcp_router_source = include_str!("../mod.rs");

    assert!(
        mcp_router_source.contains("name = \"get_guidance\""),
        "expected merged MCP guidance entrypoint to be exposed"
    );
    for removed in ["get_agent_guidance", "get_decision_brief", "create_project"] {
        assert!(
            !mcp_router_source.contains(&format!("name = \"{removed}\"")),
            "unexpected legacy MCP guidance alias still exposed: {removed}"
        );
    }
    assert!(
        mcp_router_source.contains("name = \"register_project\""),
        "expected project registration entrypoint to be exposed"
    );

    for removed in [
        "update_global_config",
        "update_project_config",
        "reload_project_config",
        "export_project_evidence",
        "cleanup_project_data",
    ] {
        assert!(
            !mcp_router_source.contains(&format!("name = \"{removed}\"")),
            "unexpected MCP tool still exposed: {removed}"
        );
    }
}
