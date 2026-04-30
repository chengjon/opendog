use super::*;

#[test]
fn verification_status_layer_reports_missing_kinds_and_safety_gates() {
    let value = verification_status_layer(&[VerificationRun {
        id: 1,
        kind: "test".to_string(),
        status: "passed".to_string(),
        command: "cargo test".to_string(),
        exit_code: Some(0),
        summary: None,
        source: "cli".to_string(),
        started_at: None,
        finished_at: fresh_ts(),
    }]);

    assert_eq!(
        value["gate_assessment"]["cleanup"]["level"],
        json!("caution")
    );
    assert_eq!(value["gate_assessment"]["cleanup"]["allowed"], json!(true));
    assert_eq!(
        value["gate_assessment"]["refactor"]["level"],
        json!("blocked")
    );
    assert_eq!(value["safe_for_cleanup"], json!(true));
    assert_eq!(value["safe_for_refactor"], json!(false));
    assert_eq!(value["cleanup_blockers"], json!([]));
    assert!(value["gate_assessment"]["cleanup"]["reasons"].is_array());
    assert!(value["gate_assessment"]["cleanup"]["next_steps"].is_array());
    assert!(value["safe_for_cleanup_reason"]
        .as_str()
        .unwrap()
        .contains("supports cleanup review"));
    assert!(value["refactor_blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v.as_str().unwrap().contains("build evidence")));
    assert!(value["missing_kinds"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "build"));
    assert_eq!(value["freshness"]["status"], json!("fresh"));
}

#[test]
fn verification_status_layer_flags_stale_evidence() {
    let value = verification_status_layer(&[VerificationRun {
        id: 1,
        kind: "test".to_string(),
        status: "passed".to_string(),
        command: "cargo test".to_string(),
        exit_code: Some(0),
        summary: None,
        source: "cli".to_string(),
        started_at: None,
        finished_at: stale_ts(),
    }]);

    assert_eq!(value["freshness"]["status"], json!("stale"));
    assert_eq!(
        value["gate_assessment"]["cleanup"]["level"],
        json!("blocked")
    );
    assert!(value["gate_assessment"]["cleanup"]["stale_kinds"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "test"));
    assert!(value["cleanup_blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("stale")));
}
