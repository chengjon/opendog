use super::*;

#[test]
fn agent_guidance_aggregates_workspace_toolchain_signals() {
    let value = agent_guidance_payload(
        3,
        2,
        &["rust-app".to_string(), "go-service".to_string()],
        &[],
        &[
            json!({
                "project_id": "rust-app",
                "recommended_next_action": "inspect_hot_files",
                "reason": "Activity exists.",
                "confidence": "medium",
                "recommended_flow": ["Inspect the hottest files first."]
            }),
            json!({
                "project_id": "go-service",
                "recommended_next_action": "inspect_hot_files",
                "reason": "Activity exists.",
                "confidence": "medium",
                "recommended_flow": ["Inspect the hottest files first."]
            }),
            json!({
                "project_id": "mystery",
                "recommended_next_action": "take_snapshot",
                "reason": "Needs baseline.",
                "confidence": "low",
                "recommended_flow": ["Take a snapshot first."]
            }),
        ],
        &[
            workspace_toolchain_overview(
                "rust-app",
                "rust",
                "high",
                &["cargo test"],
                &["cargo clippy --all-targets --all-features -- -D warnings"],
                &["cargo check"],
            ),
            workspace_toolchain_overview(
                "go-service",
                "go",
                "high",
                &["go test ./..."],
                &["go vet ./..."],
                &["go build ./..."],
            ),
            workspace_toolchain_overview("mystery", "unknown", "low", &[], &[], &[]),
        ],
    );

    let layer = &value["guidance"]["layers"]["project_toolchain"];
    assert_eq!(layer["status"], json!("available"));
    assert_eq!(layer["known_project_types"], json!(2));
    assert_eq!(layer["projects_without_detected_toolchain"], json!(1));
    assert_eq!(layer["projects_with_test_commands"], json!(2));
    assert_eq!(layer["project_type_counts"]["rust"], json!(1));
    assert_eq!(layer["project_type_counts"]["go"], json!(1));
    assert_eq!(layer["project_type_counts"]["unknown"], json!(1));
    assert!(layer["low_confidence_projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["project_id"] == "mystery"));
}

#[test]
fn workspace_toolchain_aggregation_treats_medium_high_as_trusted() {
    let value = agent_guidance_payload(
        2,
        1,
        &["hybrid".to_string()],
        &[],
        &[
            json!({
                "project_id": "hybrid",
                "recommended_next_action": "inspect_hot_files",
                "reason": "Activity exists.",
                "confidence": "medium",
                "recommended_flow": ["Inspect the hottest files first."]
            }),
            json!({
                "project_id": "mystery",
                "recommended_next_action": "take_snapshot",
                "reason": "Needs baseline.",
                "confidence": "low",
                "recommended_flow": ["Take a snapshot first."]
            }),
        ],
        &[
            workspace_toolchain_overview(
                "hybrid",
                "mixed_workspace",
                "medium-high",
                &["cargo test", "npm test"],
                &[],
                &[],
            ),
            workspace_toolchain_overview("mystery", "unknown", "low", &[], &[], &[]),
        ],
    );

    let layer = &value["guidance"]["layers"]["project_toolchain"];
    assert!(!layer["low_confidence_projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["project_id"] == "hybrid"));
    assert!(layer["low_confidence_projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["project_id"] == "mystery"));
}

#[test]
fn workspace_toolchain_aggregation_treats_docs_only_as_trusted() {
    let value = agent_guidance_payload(
        2,
        1,
        &["docs".to_string()],
        &[],
        &[
            json!({
                "project_id": "docs",
                "recommended_next_action": "inspect_hot_files",
                "reason": "Activity exists.",
                "confidence": "medium",
                "recommended_flow": ["Inspect the hottest files first."]
            }),
            json!({
                "project_id": "mystery",
                "recommended_next_action": "take_snapshot",
                "reason": "Needs baseline.",
                "confidence": "low",
                "recommended_flow": ["Take a snapshot first."]
            }),
        ],
        &[
            workspace_toolchain_overview("docs", "docs_only", "medium-high", &[], &[], &[]),
            workspace_toolchain_overview("mystery", "unknown", "low", &[], &[], &[]),
        ],
    );

    let layer = &value["guidance"]["layers"]["project_toolchain"];
    assert!(!layer["low_confidence_projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["project_id"] == "docs"));
    assert!(layer["low_confidence_projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["project_id"] == "mystery"));
}
