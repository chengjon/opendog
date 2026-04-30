use super::*;

#[test]
fn verification_status_payload_has_versioned_contract() {
    let value = verification_status_payload(
        "demo",
        &[VerificationRun {
            id: 1,
            kind: "test".to_string(),
            status: "passed".to_string(),
            command: "cargo test".to_string(),
            exit_code: Some(0),
            summary: Some("ok".to_string()),
            source: "cli".to_string(),
            started_at: None,
            finished_at: fresh_ts(),
        }],
    );

    assert_eq!(value["schema_version"], MCP_VERIFICATION_STATUS_V1);
    assert_eq!(value["project_id"], "demo");
    assert!(value["verification"]["latest_runs"].is_array());
    assert_eq!(
        value["verification"]["gate_assessment"]["cleanup"]["level"],
        json!("caution")
    );
    assert!(value["verification"]["gate_assessment"]["cleanup"]["reasons"].is_array());
    assert!(value["verification"]["gate_assessment"]["refactor"]["next_steps"].is_array());
}

#[test]
fn record_verification_payload_has_versioned_contract() {
    let value = record_verification_payload(
        "demo",
        &VerificationRun {
            id: 1,
            kind: "test".to_string(),
            status: "passed".to_string(),
            command: "cargo test".to_string(),
            exit_code: Some(0),
            summary: Some("ok".to_string()),
            source: "mcp".to_string(),
            started_at: None,
            finished_at: "1".to_string(),
        },
    );

    assert_eq!(value["schema_version"], MCP_RECORD_VERIFICATION_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["recorded"]["kind"], "test");
}

#[test]
fn run_verification_payload_has_versioned_contract() {
    let value = run_verification_payload(
        "demo",
        &crate::core::verification::ExecutedVerificationResult {
            run: VerificationRun {
                id: 1,
                kind: "test".to_string(),
                status: "passed".to_string(),
                command: "cargo test".to_string(),
                exit_code: Some(0),
                summary: Some("ok".to_string()),
                source: "mcp".to_string(),
                started_at: None,
                finished_at: "1".to_string(),
            },
            stdout_tail: "done".to_string(),
            stderr_tail: String::new(),
        },
    );

    assert_eq!(value["schema_version"], MCP_RUN_VERIFICATION_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["executed"]["run"]["status"], "passed");
}
