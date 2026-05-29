use super::{DecisionEntrypointsPlan, EntrypointReasonKind};

#[test]
fn entrypoints_plan_injects_project_id_into_cleanup_commands() {
    let plan = DecisionEntrypointsPlan::from_action("review_unused_files", Some("alpha"));

    assert_eq!(plan.next_mcp_tools()[0], "get_unused_files");
    assert_eq!(plan.next_cli_commands()[0], "opendog unused --id alpha");
    assert_eq!(
        plan.selection_reasons()[0].kind(),
        EntrypointReasonKind::McpTool
    );
}

#[test]
fn entrypoints_plan_keeps_hot_file_review_shell_context() {
    let plan = DecisionEntrypointsPlan::from_action("inspect_hot_files", None);

    assert_eq!(plan.next_mcp_tools()[0], "get_stats");
    assert!(plan.next_cli_commands().contains(&"git diff".to_string()));
    assert!(plan
        .selection_reasons()
        .iter()
        .any(|reason| reason.target() == "git diff"));
}

#[test]
fn entrypoints_plan_renders_tool_policy_contract() {
    let plan = DecisionEntrypointsPlan::from_action("unknown_action", None);
    let json = plan.to_json("mcp", "cli");

    assert_eq!(json["next_mcp_tools"][0], "get_guidance");
    assert_eq!(
        json["tool_selection_policy"]["preferred_primary_tool"],
        "mcp"
    );
    assert_eq!(
        json["tool_selection_policy"]["preferred_secondary_tool"],
        "cli"
    );
    assert_eq!(json["tool_selection_policy"]["fallback_order"][2], "shell");
}
