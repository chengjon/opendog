use super::*;

#[test]
fn agent_guidance_aggregates_workspace_verification_evidence() {
    let value = agent_guidance_payload(
        3,
        2,
        &["demo".to_string(), "clean".to_string()],
        &[],
        &[
            json!({
                "project_id": "demo",
                "recommended_next_action": "review_failing_verification",
                "reason": "Test evidence is failing.",
                "confidence": "high",
                "recommended_flow": ["Inspect verification state first."]
            }),
            json!({
                "project_id": "clean",
                "recommended_next_action": "inspect_hot_files",
                "reason": "Activity exists.",
                "confidence": "medium",
                "recommended_flow": ["Inspect the hottest files first."]
            }),
        ],
        &[
            workspace_verification_overview(
                "demo",
                "available",
                "fresh",
                &[json!({"kind":"test","status":"failed"})],
                false,
                false,
            ),
            workspace_verification_overview("clean", "available", "fresh", &[], true, true),
            workspace_verification_overview(
                "missing",
                "not_recorded",
                "missing",
                &[],
                false,
                false,
            ),
        ],
    );

    let layer = &value["guidance"]["layers"]["verification_evidence"];
    assert_eq!(layer["status"], json!("available"));
    assert_eq!(layer["projects_with_recorded_verification"], json!(2));
    assert_eq!(layer["projects_missing_verification"], json!(1));
    assert_eq!(layer["projects_with_failing_verification"], json!(1));
    assert_eq!(layer["projects_safe_for_cleanup"], json!(1));
    assert_eq!(layer["projects_safe_for_refactor"], json!(1));
    assert_eq!(layer["cleanup_gate_distribution"]["allow"], json!(1));
    assert_eq!(layer["cleanup_gate_distribution"]["blocked"], json!(2));
    assert_eq!(layer["refactor_gate_distribution"]["allow"], json!(1));
    assert_eq!(layer["refactor_gate_distribution"]["blocked"], json!(2));
    assert_eq!(layer["confidence"], json!("medium"));
    assert!(layer["blocking_projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["project_id"] == "demo" && item["cleanup_gate_level"] == "blocked"));
}
