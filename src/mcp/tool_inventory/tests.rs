use super::*;

// --- mcp_tool_inventory ---

#[test]
fn mcp_tool_inventory_returns_non_empty_slice() {
    let inventory = mcp_tool_inventory();
    assert!(!inventory.is_empty());
}

#[test]
fn mcp_tool_inventory_has_27_tools() {
    let inventory = mcp_tool_inventory();
    assert_eq!(inventory.len(), 27);
}

#[test]
fn mcp_tool_inventory_all_have_names() {
    let inventory = mcp_tool_inventory();
    for spec in inventory {
        assert!(!spec.name.is_empty(), "tool spec has empty name");
    }
}

#[test]
fn mcp_tool_inventory_all_have_contracts() {
    let inventory = mcp_tool_inventory();
    for spec in inventory {
        assert!(
            !spec.contract.is_empty(),
            "tool '{}' has empty contract",
            spec.name
        );
        assert!(
            spec.contract.starts_with("opendog.mcp."),
            "tool '{}' contract '{}' does not start with opendog.mcp.",
            spec.name,
            spec.contract
        );
    }
}

#[test]
fn mcp_tool_inventory_all_have_handler_info() {
    let inventory = mcp_tool_inventory();
    for spec in inventory {
        assert!(
            !spec.handler_module.is_empty(),
            "tool '{}' has empty handler_module",
            spec.name
        );
        assert!(
            !spec.handler.is_empty(),
            "tool '{}' has empty handler",
            spec.name
        );
    }
}

#[test]
fn mcp_tool_inventory_all_have_payload_builders() {
    let inventory = mcp_tool_inventory();
    for spec in inventory {
        assert!(
            !spec.payload_builder.is_empty(),
            "tool '{}' has empty payload_builder",
            spec.name
        );
    }
}

#[test]
fn mcp_tool_inventory_all_have_test_owners() {
    let inventory = mcp_tool_inventory();
    for spec in inventory {
        assert!(
            !spec.test_owner.is_empty(),
            "tool '{}' has empty test_owner",
            spec.name
        );
    }
}

#[test]
fn mcp_tool_inventory_no_duplicate_names() {
    let inventory = mcp_tool_inventory();
    let mut names = std::collections::HashSet::new();
    for spec in inventory {
        assert!(
            names.insert(spec.name),
            "duplicate tool name: {}",
            spec.name
        );
    }
}

#[test]
fn mcp_tool_inventory_known_tools_present() {
    let inventory = mcp_tool_inventory();
    let names: Vec<&str> = inventory.iter().map(|s| s.name).collect();
    assert!(names.contains(&"get_guidance"));
    assert!(names.contains(&"register_project"));
    assert!(names.contains(&"take_snapshot"));
    assert!(names.contains(&"start_monitor"));
    assert!(names.contains(&"stop_monitor"));
    assert!(names.contains(&"get_stats"));
    assert!(names.contains(&"get_unused_files"));
    assert!(names.contains(&"list_projects"));
    assert!(names.contains(&"delete_project"));
    assert!(names.contains(&"get_verification_status"));
    assert!(names.contains(&"record_verification_result"));
    assert!(names.contains(&"run_verification_command"));
    assert!(names.contains(&"get_data_risk_candidates"));
    assert!(names.contains(&"get_workspace_data_risk_overview"));
    assert!(names.contains(&"get_activity_rollups"));
    assert!(names.contains(&"scan_orphans"));
    assert!(names.contains(&"verify_deletion_plan"));
    assert!(names.contains(&"create_governance_lane"));
    assert!(names.contains(&"upsert_governance_node"));
    assert!(names.contains(&"get_governance_state"));
    assert!(names.contains(&"close_governance_lane"));
}

#[test]
fn mcp_tool_inventory_tools_with_params() {
    let inventory = mcp_tool_inventory();
    let tools_with_params: Vec<&str> = inventory
        .iter()
        .filter(|s| s.params_type.is_some())
        .map(|s| s.name)
        .collect();
    // Most tools take params
    assert!(tools_with_params.contains(&"register_project"));
    assert!(tools_with_params.contains(&"take_snapshot"));
    assert!(tools_with_params.contains(&"get_stats"));
}

#[test]
fn mcp_tool_inventory_tools_without_params() {
    let inventory = mcp_tool_inventory();
    let tools_without: Vec<&str> = inventory
        .iter()
        .filter(|s| s.params_type.is_none())
        .map(|s| s.name)
        .collect();
    assert!(tools_without.contains(&"get_global_config"));
    assert!(tools_without.contains(&"get_build_info"));
    assert!(tools_without.contains(&"list_projects"));
}

// --- McpToolSpec struct construction ---

#[test]
fn mcp_tool_spec_construction() {
    let spec = McpToolSpec {
        name: "test_tool",
        contract: "opendog.mcp.test.v1",
        params_type: Some("TestParams"),
        payload_builder: "test_payload",
        handler_module: "test_handlers",
        handler: "handle_test",
        test_owner: "mcp::tests::test_module",
    };
    assert_eq!(spec.name, "test_tool");
    assert_eq!(spec.contract, "opendog.mcp.test.v1");
    assert_eq!(spec.params_type, Some("TestParams"));
    assert_eq!(spec.payload_builder, "test_payload");
    assert_eq!(spec.handler_module, "test_handlers");
    assert_eq!(spec.handler, "handle_test");
    assert_eq!(spec.test_owner, "mcp::tests::test_module");
}

#[test]
fn mcp_tool_spec_equality() {
    let a = McpToolSpec {
        name: "x",
        contract: "c",
        params_type: None,
        payload_builder: "p",
        handler_module: "m",
        handler: "h",
        test_owner: "t",
    };
    let b = McpToolSpec {
        name: "x",
        contract: "c",
        params_type: None,
        payload_builder: "p",
        handler_module: "m",
        handler: "h",
        test_owner: "t",
    };
    assert_eq!(a, b);
}

#[test]
fn mcp_tool_spec_inequality() {
    let a = McpToolSpec {
        name: "x",
        contract: "c",
        params_type: None,
        payload_builder: "p",
        handler_module: "m",
        handler: "h",
        test_owner: "t",
    };
    let b = McpToolSpec {
        name: "y",
        contract: "c",
        params_type: None,
        payload_builder: "p",
        handler_module: "m",
        handler: "h",
        test_owner: "t",
    };
    assert_ne!(a, b);
}

#[test]
fn mcp_tool_spec_debug_format() {
    let spec = McpToolSpec {
        name: "my_tool",
        contract: "opendog.mcp.my.v1",
        params_type: Some("MyParams"),
        payload_builder: "my_payload",
        handler_module: "my_handlers",
        handler: "handle_my",
        test_owner: "mcp::tests::my",
    };
    let debug_str = format!("{:?}", spec);
    assert!(debug_str.contains("my_tool"));
}
