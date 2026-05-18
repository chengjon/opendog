use super::{mcp_resource_templates, mcp_resources, read_resource_kind, ResourceKind};

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

#[test]
fn observation_tool_params_expose_path_classification_filter() {
    let params_source = include_str!("../params.rs");
    let mcp_router_source = include_str!("../mod.rs");

    assert!(params_source.contains("pub path_classification: Option<String>"));
    assert!(mcp_router_source.contains("path_classification filters rows"));
}

#[test]
fn orphan_detection_tools_are_exposed() {
    let mcp_router_source = include_str!("../mod.rs");
    assert!(mcp_router_source.contains("name = \"scan_orphans\""));
    assert!(mcp_router_source.contains("name = \"verify_deletion_plan\""));
}

#[test]
fn mcp_resource_templates_expose_readonly_project_state_uris() {
    let templates = mcp_resource_templates();
    let uris: Vec<&str> = templates
        .iter()
        .map(|template| template.raw.uri_template.as_str())
        .collect();

    assert!(uris.contains(&"opendog://projects"));
    assert!(uris.contains(&"opendog://project/{id}/verification"));
}

#[test]
fn mcp_resources_expose_static_projects_resource() {
    let resources = mcp_resources();
    let uris: Vec<&str> = resources
        .iter()
        .map(|resource| resource.raw.uri.as_str())
        .collect();

    assert!(uris.contains(&"opendog://projects"));
}

#[test]
fn mcp_resource_uri_parser_accepts_only_readonly_state_resources() {
    assert_eq!(
        read_resource_kind("opendog://projects"),
        Some(ResourceKind::Projects)
    );
    assert_eq!(
        read_resource_kind("opendog://project/demo/verification"),
        Some(ResourceKind::ProjectVerification {
            id: "demo".to_string()
        })
    );
    assert_eq!(read_resource_kind("opendog://project/demo/delete"), None);
}
