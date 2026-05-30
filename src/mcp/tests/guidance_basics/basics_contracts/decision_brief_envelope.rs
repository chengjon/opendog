use super::*;
use serde_json::Value;

#[path = "decision_brief_envelope/fixtures.rs"]
mod fixtures;

fn assert_selected_execution_sequence(
    recommendation: serde_json::Value,
    monitoring_count: usize,
    monitored_projects: &[String],
    expected_sequence: serde_json::Value,
) {
    let project_overview = fixtures::demo_project_overview();
    let agent_guidance = agent_guidance_payload(
        1,
        monitoring_count,
        monitored_projects,
        &[],
        std::slice::from_ref(&recommendation),
        std::slice::from_ref(&project_overview),
        default_governance_layer(),
    );

    let brief = decision_brief_payload(
        MCP_DECISION_BRIEF_V1,
        "project",
        Some("demo"),
        1,
        &agent_guidance,
        None,
    );

    assert_eq!(brief["decision"]["execution_sequence"], expected_sequence);
}

#[path = "decision_brief_envelope/entry_envelope.rs"]
mod entry_envelope;
#[path = "decision_brief_envelope/external_truth_boundary.rs"]
mod external_truth_boundary;
#[path = "decision_brief_envelope/review_focus_projection.rs"]
mod review_focus_projection;
#[path = "decision_brief_envelope/selected_sequences.rs"]
mod selected_sequences;
