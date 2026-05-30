use serde_json::{json, Value};

mod action_catalog;

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
        action_catalog::plan_for_action(action, project_id)
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
mod tests;
