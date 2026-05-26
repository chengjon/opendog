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

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn recommended_flow_extracts_text_steps() {
        let value = json!(["step 1", "step 2", "step 3"]);
        let steps: Vec<String> = value
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_str().map(|text| format!("  {}. {}", i + 1, text)))
            .collect();
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0], "  1. step 1");
        assert_eq!(steps[2], "  3. step 3");
    }

    #[test]
    fn recommended_flow_skips_non_string_entries() {
        let value = json!(["step 1", 42, null, "step 2"]);
        let steps: Vec<&str> = value
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|s| s.as_str())
            .collect();
        assert_eq!(steps, vec!["step 1", "step 2"]);
    }

    #[test]
    fn recommended_flow_empty_array_returns_none() {
        let value = json!([]);
        let is_empty = value.as_array().map(|a| a.is_empty()).unwrap_or(true);
        assert!(is_empty);
    }

    #[test]
    fn recommended_flow_non_array_returns_none() {
        let value = json!("not an array");
        let is_none = value.as_array().is_none();
        assert!(is_none);
    }

    #[test]
    fn recommended_flow_null_returns_none() {
        let value = json!(null);
        let is_none = value.as_array().is_none();
        assert!(is_none);
    }
}
