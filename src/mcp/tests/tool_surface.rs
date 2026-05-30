use std::collections::BTreeSet;

use super::{
    mcp_resource_templates, mcp_resources, mcp_tool_inventory, read_resource_kind, ResourceKind,
};

fn registered_tool_names() -> BTreeSet<String> {
    mcp_tool_router_source()
        .lines()
        .filter_map(|line| {
            line.split("name = \"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .map(ToString::to_string)
        })
        .collect()
}

fn mcp_tool_router_source() -> &'static str {
    concat!(
        include_str!("../server_tools.rs"),
        include_str!("../server_tools/analysis.rs"),
        include_str!("../server_tools/config.rs"),
        include_str!("../server_tools/governance.rs"),
        include_str!("../server_tools/lifecycle.rs"),
        include_str!("../server_tools/risk.rs"),
        include_str!("../server_tools/verification.rs"),
    )
}

fn documented_tool_names() -> BTreeSet<String> {
    let tool_reference = include_str!("../../../docs/mcp-tool-reference.md");

    tool_reference
        .lines()
        .filter_map(|line| {
            line.strip_prefix("## `")
                .and_then(|rest| rest.split('`').next())
                .filter(|name| !name.starts_with("opendog://"))
                .map(ToString::to_string)
        })
        .collect()
}

#[test]
fn mcp_tool_surface_matches_inventory() {
    let registered = registered_tool_names();
    let inventory: BTreeSet<String> = mcp_tool_inventory()
        .iter()
        .map(|tool| tool.name.to_string())
        .collect();

    assert_eq!(
        registered, inventory,
        "MCP #[tool] registrations must match the central inventory"
    );

    for tool in mcp_tool_inventory() {
        assert!(!tool.contract.is_empty(), "{} missing contract", tool.name);
        assert!(
            tool.params_type.is_none_or(|params| !params.is_empty()),
            "{} has an empty params_type",
            tool.name
        );
        assert!(
            !tool.payload_builder.is_empty(),
            "{} missing payload builder",
            tool.name
        );
        assert!(
            !tool.handler_module.is_empty(),
            "{} missing handler module",
            tool.name
        );
        assert!(!tool.handler.is_empty(), "{} missing handler", tool.name);
        assert!(
            !tool.test_owner.is_empty(),
            "{} missing test owner",
            tool.name
        );
    }
}

#[test]
fn mcp_tool_reference_documents_inventory() {
    let documented = documented_tool_names();
    let inventory: BTreeSet<String> = mcp_tool_inventory()
        .iter()
        .map(|tool| tool.name.to_string())
        .collect();

    assert_eq!(
        documented, inventory,
        "docs/mcp-tool-reference.md tool headings must match the central MCP inventory"
    );
}

#[test]
fn mcp_tool_surface_excludes_operator_only_mutation_tools() {
    let mcp_router_source = mcp_tool_router_source();
    let inventory_names: BTreeSet<&str> =
        mcp_tool_inventory().iter().map(|tool| tool.name).collect();

    assert!(
        inventory_names.contains("get_guidance"),
        "expected merged MCP guidance entrypoint to be exposed"
    );
    for removed in ["get_agent_guidance", "get_decision_brief", "create_project"] {
        assert!(
            !mcp_router_source.contains(&format!("name = \"{removed}\"")),
            "unexpected legacy MCP guidance alias still exposed: {removed}"
        );
    }
    assert!(
        inventory_names.contains("register_project"),
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
            !inventory_names.contains(removed),
            "unexpected MCP tool in inventory: {removed}"
        );
        assert!(
            !mcp_router_source.contains(&format!("name = \"{removed}\"")),
            "unexpected MCP tool still exposed: {removed}"
        );
    }
}

#[test]
fn observation_tool_params_expose_path_classification_filter() {
    let params_source = include_str!("../params/basic.rs");
    let mcp_router_source = mcp_tool_router_source();

    assert!(params_source.contains("pub path_classification: Option<String>"));
    assert!(mcp_router_source.contains("path_classification filters rows"));
}

#[test]
fn orphan_detection_tools_are_exposed() {
    let inventory_names: BTreeSet<&str> =
        mcp_tool_inventory().iter().map(|tool| tool.name).collect();
    assert!(inventory_names.contains("scan_orphans"));
    assert!(inventory_names.contains("verify_deletion_plan"));
}

#[test]
fn governance_tools_are_exposed() {
    let inventory_names: BTreeSet<&str> =
        mcp_tool_inventory().iter().map(|tool| tool.name).collect();
    assert!(inventory_names.contains("create_governance_lane"));
    assert!(inventory_names.contains("upsert_governance_node"));
    assert!(inventory_names.contains("get_governance_state"));
    assert!(inventory_names.contains("close_governance_lane"));
}

#[test]
fn activity_rollup_tool_is_exposed() {
    let inventory_names: BTreeSet<&str> =
        mcp_tool_inventory().iter().map(|tool| tool.name).collect();
    assert!(inventory_names.contains("get_activity_rollups"));
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
