mod agent_guidance_output;
mod data_risk_output;
mod decision_brief_output;

use serde_json::Value;

pub(super) fn print_agent_guidance(guidance: &Value) {
    agent_guidance_output::print_agent_guidance(guidance);
}

pub(super) fn print_decision_brief(payload: &Value) {
    decision_brief_output::print_decision_brief(payload);
}

pub(super) fn print_data_risk(
    id: &str,
    candidate_type: &str,
    min_review_priority: &str,
    rendered: &Value,
    guidance: &Value,
) {
    data_risk_output::print_data_risk(id, candidate_type, min_review_priority, rendered, guidance);
}

pub(super) fn print_workspace_data_risk(
    candidate_type: &str,
    min_review_priority: &str,
    project_limit: usize,
    total_registered_projects: usize,
    matched_projects: usize,
    guidance: &Value,
) {
    data_risk_output::print_workspace_data_risk(
        candidate_type,
        min_review_priority,
        project_limit,
        total_registered_projects,
        matched_projects,
        guidance,
    );
}

pub(super) fn print_recommended_flow(value: &Value) {
    let Some(steps) = value.as_array() else {
        return;
    };
    if steps.is_empty() {
        return;
    }

    println!();
    println!("Recommended flow:");
    for (index, step) in steps.iter().enumerate() {
        if let Some(text) = step.as_str() {
            println!("  {}. {}", index + 1, text);
        }
    }
}
