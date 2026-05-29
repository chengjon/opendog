use serde_json::Value;

mod model;

use model::DecisionEntrypointsPlan;

pub(in crate::mcp) fn decision_entrypoints_payload(
    action: &str,
    project_id: Option<&str>,
    preferred_primary_tool: &str,
    preferred_secondary_tool: &str,
) -> Value {
    DecisionEntrypointsPlan::from_action(action, project_id)
        .to_json(preferred_primary_tool, preferred_secondary_tool)
}

#[cfg(test)]
mod tests;
