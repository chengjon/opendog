use serde_json::{json, Value};

pub(crate) fn execution_sequence_for_recommendation(
    forced_action: Option<&str>,
    repo_risk: &Value,
) -> Value {
    let operation_active = repo_risk["operation_states"]
        .as_array()
        .map(|states| !states.is_empty())
        .unwrap_or(false);

    if forced_action != Some("stabilize_repository_state") || !operation_active {
        return Value::Null;
    }

    json!({
        "mode": "shell_stabilize_then_resume",
        "current_phase": "stabilize",
        "resume_with": "refresh_guidance_after_repo_stable",
        "stability_checks": ["git status", "git diff"],
        "resume_conditions": [
            "operation_states_cleared",
            "conflicted_count_zero"
        ]
    })
}
