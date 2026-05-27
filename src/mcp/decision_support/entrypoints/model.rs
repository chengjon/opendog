use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EntrypointReasonKind {
    McpTool,
    ShellCommand,
    CliOrShell,
}

impl EntrypointReasonKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::McpTool => "mcp_tool",
            Self::ShellCommand => "shell_command",
            Self::CliOrShell => "cli_or_shell",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntrypointSelectionReason {
    kind: EntrypointReasonKind,
    target: String,
    why: String,
}

impl EntrypointSelectionReason {
    fn new(kind: EntrypointReasonKind, target: &str, why: &str) -> Self {
        Self {
            kind,
            target: target.to_string(),
            why: why.to_string(),
        }
    }

    #[cfg(test)]
    fn kind(&self) -> EntrypointReasonKind {
        self.kind
    }

    #[cfg(test)]
    fn target(&self) -> &str {
        &self.target
    }

    fn to_json(&self) -> Value {
        json!({
            "kind": self.kind.as_str(),
            "target": self.target,
            "why": self.why,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DecisionEntrypointsPlan {
    next_mcp_tools: Vec<String>,
    next_cli_commands: Vec<String>,
    selection_reasons: Vec<EntrypointSelectionReason>,
}

impl DecisionEntrypointsPlan {
    pub(super) fn from_action(action: &str, project_id: Option<&str>) -> Self {
        let project_flag = project_id
            .map(|id| format!(" --id {id}"))
            .unwrap_or_default();

        match action {
            "review_failing_verification" => Self::new(
                &[
                    "get_verification_status",
                    "run_verification_command",
                    "get_data_risk_candidates",
                ],
                vec![
                    format!("opendog verification{project_flag}"),
                    format!("opendog run-verification{project_flag} --kind test --command '<cmd>'"),
                    format!("opendog data-risk{project_flag} --json"),
                ],
                vec![
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "get_verification_status",
                        "inspect persisted evidence before new edits",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "run_verification_command",
                        "refresh failing or missing verification evidence",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::CliOrShell,
                        "git diff",
                        "compare current repository state against the failing evidence",
                    ),
                ],
            ),
            "stabilize_repository_state" => Self::new(
                &["get_guidance", "get_verification_status"],
                vec![
                    "git status".to_string(),
                    "git diff".to_string(),
                    format!("opendog verification{project_flag}"),
                ],
                vec![
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::ShellCommand,
                        "git status",
                        "repository truth is needed before activity-based guidance",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::ShellCommand,
                        "git diff",
                        "inspect the unstable working state directly",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "get_verification_status",
                        "avoid cleanup or refactor while evidence is stale or failing",
                    ),
                ],
            ),
            "start_monitor" => Self::new(
                &["start_monitor", "take_snapshot", "get_stats"],
                vec![
                    format!("opendog start{project_flag}"),
                    format!("opendog snapshot{project_flag}"),
                    format!("opendog stats{project_flag}"),
                ],
                vec![
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "start_monitor",
                        "fresh activity evidence does not exist yet",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "take_snapshot",
                        "monitoring is more useful when a file baseline exists",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "get_stats",
                        "only meaningful after monitoring and baseline data exist",
                    ),
                ],
            ),
            "take_snapshot" => Self::new(
                &["take_snapshot", "get_stats", "get_unused_files"],
                vec![
                    format!("opendog snapshot{project_flag}"),
                    format!("opendog stats{project_flag}"),
                    format!("opendog unused{project_flag}"),
                ],
                vec![
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "take_snapshot",
                        "file inventory is still missing",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "get_stats",
                        "activity rankings depend on an established snapshot",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "get_unused_files",
                        "cleanup review depends on the snapshot baseline",
                    ),
                ],
            ),
            "generate_activity_then_stats" => Self::new(
                &["get_stats", "get_guidance"],
                vec![
                    "git status".to_string(),
                    format!("opendog stats{project_flag}"),
                ],
                vec![
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::ShellCommand,
                        "git status",
                        "real workflow activity usually comes from edits, builds, or tests",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "get_stats",
                        "wait until enough activity exists before trusting hotspot signals",
                    ),
                ],
            ),
            "run_verification_before_high_risk_changes" => Self::new(
                &[
                    "get_verification_status",
                    "run_verification_command",
                    "get_data_risk_candidates",
                ],
                vec![
                    format!("opendog verification{project_flag}"),
                    format!("opendog run-verification{project_flag} --kind test --command '<cmd>'"),
                    format!("opendog data-risk{project_flag} --json"),
                ],
                vec![
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "get_verification_status",
                        "determine exactly which evidence is still missing",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "run_verification_command",
                        "record fresh build/test/lint evidence before risky changes",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "get_data_risk_candidates",
                        "check whether suspicious data files raise cleanup or refactor risk",
                    ),
                ],
            ),
            "review_unused_files" => Self::new(
                &[
                    "get_unused_files",
                    "get_verification_status",
                    "get_data_risk_candidates",
                ],
                vec![
                    format!("opendog unused{project_flag}"),
                    format!("opendog verification{project_flag}"),
                    "rg \"<pattern>\" .".to_string(),
                ],
                vec![
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "get_unused_files",
                        "OPENDOG has already ranked cleanup candidates",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "get_verification_status",
                        "cleanup decisions still need evidence gates",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::ShellCommand,
                        "rg \"<pattern>\" .",
                        "confirm whether unused candidates are referenced indirectly",
                    ),
                ],
            ),
            "inspect_hot_files" => Self::new(
                &[
                    "get_stats",
                    "get_verification_status",
                    "get_data_risk_candidates",
                ],
                vec![
                    format!("opendog stats{project_flag}"),
                    "git diff".to_string(),
                    "rg \"<pattern>\" .".to_string(),
                ],
                vec![
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "get_stats",
                        "activity-based ranking should narrow the first inspection target",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::ShellCommand,
                        "git diff",
                        "hot files still need source-level repository context",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::ShellCommand,
                        "rg \"<pattern>\" .",
                        "follow activity signals with code search before edits",
                    ),
                ],
            ),
            _ => Self::new(
                &[
                    "get_guidance",
                    "list_projects",
                    "get_workspace_data_risk_overview",
                ],
                vec![
                    "opendog agent-guidance".to_string(),
                    "opendog list".to_string(),
                    "opendog workspace-data-risk --json".to_string(),
                ],
                vec![
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "get_guidance",
                        "refresh project-level next-action advice",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "list_projects",
                        "reconfirm project availability and monitoring state",
                    ),
                    EntrypointSelectionReason::new(
                        EntrypointReasonKind::McpTool,
                        "get_workspace_data_risk_overview",
                        "cross-project prioritization may still be unresolved",
                    ),
                ],
            ),
        }
    }

    fn new(
        next_mcp_tools: &[&str],
        next_cli_commands: Vec<String>,
        selection_reasons: Vec<EntrypointSelectionReason>,
    ) -> Self {
        Self {
            next_mcp_tools: next_mcp_tools
                .iter()
                .map(|tool| (*tool).to_string())
                .collect(),
            next_cli_commands,
            selection_reasons,
        }
    }

    #[cfg(test)]
    pub(super) fn next_mcp_tools(&self) -> &[String] {
        &self.next_mcp_tools
    }

    #[cfg(test)]
    pub(super) fn next_cli_commands(&self) -> &[String] {
        &self.next_cli_commands
    }

    #[cfg(test)]
    fn selection_reasons(&self) -> &[EntrypointSelectionReason] {
        &self.selection_reasons
    }

    pub(super) fn to_json(
        &self,
        preferred_primary_tool: &str,
        preferred_secondary_tool: &str,
    ) -> Value {
        let selection_reasons: Vec<Value> = self
            .selection_reasons
            .iter()
            .map(EntrypointSelectionReason::to_json)
            .collect();

        json!({
            "next_mcp_tools": self.next_mcp_tools,
            "next_cli_commands": self.next_cli_commands,
            "selection_reasons": selection_reasons,
            "tool_selection_policy": {
                "preferred_primary_tool": preferred_primary_tool,
                "preferred_secondary_tool": preferred_secondary_tool,
                "fallback_order": [preferred_primary_tool, preferred_secondary_tool, "shell"],
            }
        })
    }
}

#[cfg(test)]
mod tests {
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
}
