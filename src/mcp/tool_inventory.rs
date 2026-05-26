use crate::contracts::{
    MCP_BUILD_INFO_V1, MCP_CLOSE_GOVERNANCE_LANE_V1, MCP_CREATE_GOVERNANCE_LANE_V1,
    MCP_DATA_RISK_V1, MCP_DELETE_PROJECT_V1, MCP_GET_GOVERNANCE_STATE_V1, MCP_GLOBAL_CONFIG_V1,
    MCP_GUIDANCE_V1, MCP_LIST_PROJECTS_V1, MCP_ORPHAN_DELETION_PLAN_V1, MCP_ORPHAN_SCAN_V1,
    MCP_PROJECT_CONFIG_V1, MCP_RECORD_VERIFICATION_V1, MCP_REGISTER_PROJECT_V1,
    MCP_RUN_VERIFICATION_V1, MCP_SNAPSHOT_COMPARE_V1, MCP_SNAPSHOT_V1, MCP_START_MONITOR_V1,
    MCP_STATS_V1, MCP_STOP_MONITOR_V1, MCP_TIME_WINDOW_REPORT_V1, MCP_UNUSED_FILES_V1,
    MCP_UPSERT_GOVERNANCE_NODE_V1, MCP_USAGE_TRENDS_V1, MCP_VERIFICATION_STATUS_V1,
    MCP_WORKSPACE_DATA_RISK_V1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct McpToolSpec {
    pub(crate) name: &'static str,
    pub(crate) contract: &'static str,
    pub(crate) params_type: Option<&'static str>,
    pub(crate) payload_builder: &'static str,
    pub(crate) handler_module: &'static str,
    pub(crate) handler: &'static str,
    pub(crate) test_owner: &'static str,
}

pub(crate) const MCP_TOOL_INVENTORY: &[McpToolSpec] = &[
    McpToolSpec {
        name: "get_guidance",
        contract: MCP_GUIDANCE_V1,
        params_type: Some("GuidanceParams"),
        payload_builder: "agent_guidance_payload",
        handler_module: "guidance_handlers",
        handler: "handle_get_guidance",
        test_owner: "mcp::tests::guidance_basics",
    },
    McpToolSpec {
        name: "get_global_config",
        contract: MCP_GLOBAL_CONFIG_V1,
        params_type: None,
        payload_builder: "global_config_payload",
        handler_module: "config_handlers",
        handler: "handle_get_global_config",
        test_owner: "mcp::tests::payload_contracts::config_payloads",
    },
    McpToolSpec {
        name: "get_build_info",
        contract: MCP_BUILD_INFO_V1,
        params_type: None,
        payload_builder: "build_info_payload",
        handler_module: "config_handlers",
        handler: "handle_get_build_info",
        test_owner: "mcp::tests::payload_contracts::config_payloads",
    },
    McpToolSpec {
        name: "get_project_config",
        contract: MCP_PROJECT_CONFIG_V1,
        params_type: Some("ProjectIdParams"),
        payload_builder: "project_config_payload",
        handler_module: "config_handlers",
        handler: "handle_get_project_config",
        test_owner: "mcp::tests::payload_contracts::config_payloads",
    },
    McpToolSpec {
        name: "register_project",
        contract: MCP_REGISTER_PROJECT_V1,
        params_type: Some("RegisterProjectParams"),
        payload_builder: "register_project_payload",
        handler_module: "project_handlers",
        handler: "handle_register_project",
        test_owner: "mcp::tests::payload_contracts::project_payloads",
    },
    McpToolSpec {
        name: "take_snapshot",
        contract: MCP_SNAPSHOT_V1,
        params_type: Some("ProjectIdParams"),
        payload_builder: "snapshot_payload",
        handler_module: "project_handlers",
        handler: "handle_take_snapshot",
        test_owner: "mcp::tests::payload_contracts::analysis_payloads",
    },
    McpToolSpec {
        name: "start_monitor",
        contract: MCP_START_MONITOR_V1,
        params_type: Some("ProjectIdParams"),
        payload_builder: "start_monitor_payload",
        handler_module: "project_handlers",
        handler: "handle_start_monitor",
        test_owner: "mcp::tests::payload_contracts::project_payloads",
    },
    McpToolSpec {
        name: "stop_monitor",
        contract: MCP_STOP_MONITOR_V1,
        params_type: Some("ProjectIdParams"),
        payload_builder: "stop_monitor_payload",
        handler_module: "project_handlers",
        handler: "handle_stop_monitor",
        test_owner: "mcp::tests::payload_contracts::project_payloads",
    },
    McpToolSpec {
        name: "get_stats",
        contract: MCP_STATS_V1,
        params_type: Some("ObservationRowsParams"),
        payload_builder: "stats_payload",
        handler_module: "analysis_handlers",
        handler: "handle_get_stats",
        test_owner: "mcp::tests::payload_contracts::analysis_payloads",
    },
    McpToolSpec {
        name: "get_unused_files",
        contract: MCP_UNUSED_FILES_V1,
        params_type: Some("ObservationRowsParams"),
        payload_builder: "unused_files_payload",
        handler_module: "analysis_handlers",
        handler: "handle_get_unused_files",
        test_owner: "mcp::tests::payload_contracts::analysis_payloads",
    },
    McpToolSpec {
        name: "get_time_window_report",
        contract: MCP_TIME_WINDOW_REPORT_V1,
        params_type: Some("TimeWindowReportParams"),
        payload_builder: "time_window_report_payload",
        handler_module: "analysis_handlers",
        handler: "handle_get_time_window_report",
        test_owner: "mcp::tests::payload_contracts::analysis_payloads",
    },
    McpToolSpec {
        name: "compare_snapshots",
        contract: MCP_SNAPSHOT_COMPARE_V1,
        params_type: Some("CompareSnapshotsParams"),
        payload_builder: "snapshot_compare_payload",
        handler_module: "analysis_handlers",
        handler: "handle_compare_snapshots",
        test_owner: "mcp::tests::payload_contracts::analysis_payloads",
    },
    McpToolSpec {
        name: "get_usage_trends",
        contract: MCP_USAGE_TRENDS_V1,
        params_type: Some("UsageTrendParams"),
        payload_builder: "usage_trends_payload",
        handler_module: "analysis_handlers",
        handler: "handle_get_usage_trends",
        test_owner: "mcp::tests::payload_contracts::analysis_payloads",
    },
    McpToolSpec {
        name: "get_verification_status",
        contract: MCP_VERIFICATION_STATUS_V1,
        params_type: Some("ProjectIdParams"),
        payload_builder: "verification_status_payload",
        handler_module: "verification_handlers",
        handler: "handle_get_verification_status",
        test_owner: "mcp::tests::payload_contracts::verification_payloads",
    },
    McpToolSpec {
        name: "record_verification_result",
        contract: MCP_RECORD_VERIFICATION_V1,
        params_type: Some("RecordVerificationParams"),
        payload_builder: "record_verification_payload",
        handler_module: "verification_handlers",
        handler: "handle_record_verification_result",
        test_owner: "mcp::tests::payload_contracts::verification_payloads",
    },
    McpToolSpec {
        name: "run_verification_command",
        contract: MCP_RUN_VERIFICATION_V1,
        params_type: Some("ExecuteVerificationParams"),
        payload_builder: "run_verification_payload",
        handler_module: "verification_handlers",
        handler: "handle_run_verification_command",
        test_owner: "mcp::tests::payload_contracts::verification_payloads",
    },
    McpToolSpec {
        name: "scan_orphans",
        contract: MCP_ORPHAN_SCAN_V1,
        params_type: Some("ScanOrphansParams"),
        payload_builder: "orphan_scan_payload",
        handler_module: "orphan_handlers",
        handler: "handle_scan_orphans",
        test_owner: "mcp::tests::payload_contracts::orphan_payloads",
    },
    McpToolSpec {
        name: "verify_deletion_plan",
        contract: MCP_ORPHAN_DELETION_PLAN_V1,
        params_type: Some("VerifyDeletionPlanParams"),
        payload_builder: "orphan_deletion_plan_payload",
        handler_module: "orphan_handlers",
        handler: "handle_verify_deletion_plan",
        test_owner: "mcp::tests::payload_contracts::orphan_payloads",
    },
    McpToolSpec {
        name: "get_data_risk_candidates",
        contract: MCP_DATA_RISK_V1,
        params_type: Some("DataRiskParams"),
        payload_builder: "project_data_risk_payload",
        handler_module: "risk_handlers",
        handler: "handle_get_data_risk_candidates",
        test_owner: "mcp::tests::data_risk_cases",
    },
    McpToolSpec {
        name: "get_workspace_data_risk_overview",
        contract: MCP_WORKSPACE_DATA_RISK_V1,
        params_type: Some("WorkspaceDataRiskParams"),
        payload_builder: "workspace_data_risk_payload",
        handler_module: "risk_handlers",
        handler: "handle_get_workspace_data_risk_overview",
        test_owner: "mcp::tests::data_risk_cases",
    },
    McpToolSpec {
        name: "list_projects",
        contract: MCP_LIST_PROJECTS_V1,
        params_type: None,
        payload_builder: "list_projects_payload",
        handler_module: "project_handlers",
        handler: "handle_list_projects",
        test_owner: "mcp::tests::payload_contracts::project_payloads",
    },
    McpToolSpec {
        name: "delete_project",
        contract: MCP_DELETE_PROJECT_V1,
        params_type: Some("ProjectIdParams"),
        payload_builder: "delete_project_payload",
        handler_module: "project_handlers",
        handler: "handle_delete_project",
        test_owner: "mcp::tests::payload_contracts::project_payloads",
    },
    McpToolSpec {
        name: "create_governance_lane",
        contract: MCP_CREATE_GOVERNANCE_LANE_V1,
        params_type: Some("CreateGovernanceLaneParams"),
        payload_builder: "create_governance_lane_payload",
        handler_module: "governance_handlers",
        handler: "handle_create_governance_lane",
        test_owner: "mcp::tests::payload_contracts::governance_payloads",
    },
    McpToolSpec {
        name: "upsert_governance_node",
        contract: MCP_UPSERT_GOVERNANCE_NODE_V1,
        params_type: Some("UpsertGovernanceNodeParams"),
        payload_builder: "upsert_governance_node_payload",
        handler_module: "governance_handlers",
        handler: "handle_upsert_governance_node",
        test_owner: "mcp::tests::payload_contracts::governance_payloads",
    },
    McpToolSpec {
        name: "get_governance_state",
        contract: MCP_GET_GOVERNANCE_STATE_V1,
        params_type: Some("GetGovernanceStateParams"),
        payload_builder: "get_governance_state_payload",
        handler_module: "governance_handlers",
        handler: "handle_get_governance_state",
        test_owner: "mcp::tests::payload_contracts::governance_payloads",
    },
    McpToolSpec {
        name: "close_governance_lane",
        contract: MCP_CLOSE_GOVERNANCE_LANE_V1,
        params_type: Some("CloseGovernanceLaneParams"),
        payload_builder: "close_governance_lane_payload",
        handler_module: "governance_handlers",
        handler: "handle_close_governance_lane",
        test_owner: "mcp::tests::payload_contracts::governance_payloads",
    },
];

pub(crate) fn mcp_tool_inventory() -> &'static [McpToolSpec] {
    MCP_TOOL_INVENTORY
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- mcp_tool_inventory ---

    #[test]
    fn mcp_tool_inventory_returns_non_empty_slice() {
        let inventory = mcp_tool_inventory();
        assert!(!inventory.is_empty());
    }

    #[test]
    fn mcp_tool_inventory_has_26_tools() {
        let inventory = mcp_tool_inventory();
        assert_eq!(inventory.len(), 26);
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
}
