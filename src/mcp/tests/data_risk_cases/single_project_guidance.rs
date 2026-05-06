use super::*;
use crate::contracts::MCP_DATA_RISK_V1;
use crate::mcp::project_data_risk_payload;

#[test]
fn data_risk_guidance_surfaces_counts_and_candidates() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/customer_seed.rs"),
        r#"const CUSTOMER: &str = "Acme Corp"; const EMAIL: &str = "ops@corp.com"; const ADDRESS: &str = "1 Market Street";"#,
    )
    .unwrap();
    let entries = vec![StatsEntry {
        file_path: "src/customer_seed.rs".to_string(),
        size: 10,
        file_type: "rs".to_string(),
        access_count: 2,
        estimated_duration_ms: 0,
        modification_count: 0,
        last_access_time: None,
        first_seen_time: None,
    }];
    let report = detect_mock_data_report(dir.path(), &entries);

    let guidance = data_risk_guidance(dir.path(), &report);
    let payload = project_data_risk_payload(
        MCP_DATA_RISK_V1,
        "demo",
        "all",
        "low",
        10,
        dir.path(),
        &entries,
    );
    assert_eq!(
        guidance["layers"]["execution_strategy"]["hardcoded_candidate_count"],
        json!(1)
    );
    assert_eq!(
        guidance["data_risk_focus"],
        json!({
            "primary_focus": "hardcoded",
            "priority_order": ["hardcoded", "mixed", "mock"],
            "basis": [
                "hardcoded_candidates_present",
                "runtime_shared_candidates_present",
                "high_severity_content_hits_present"
            ]
        })
    );
    assert_eq!(
        guidance["layers"]["cleanup_refactor_candidates"]["hardcoded_data_candidates"][0]
            ["file_path"],
        json!("src/customer_seed.rs")
    );
    assert_eq!(
        guidance["layers"]["cleanup_refactor_candidates"]["data_risk_focus"],
        guidance["data_risk_focus"]
    );
    assert_eq!(payload["data_risk_focus"], guidance["data_risk_focus"]);
    assert_eq!(
        payload["guidance"]["data_risk_focus"],
        guidance["data_risk_focus"]
    );
    assert_eq!(
        payload["guidance"]["layers"]["cleanup_refactor_candidates"]["data_risk_focus"],
        guidance["data_risk_focus"]
    );
    assert_eq!(
        guidance["recommended_flow"][0],
        json!("Review high-priority hardcoded-data candidates first.")
    );
    assert_eq!(
        guidance["next_tools"],
        json!(["get_stats", "get_unused_files", "get_guidance"])
    );
    assert!(guidance["recommended_flow"]
        .as_array()
        .unwrap()
        .iter()
        .any(|step| step.as_str().unwrap().contains("verification evidence")));
}

#[test]
fn data_risk_guidance_runtime_shared_thin_combo_with_weak_path_token_stays_none() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/customer_sample.rs"),
        r#"const CUSTOMER_EMAIL: &str = "ops@corp.com";"#,
    )
    .unwrap();
    let entries = vec![StatsEntry {
        file_path: "src/customer_sample.rs".to_string(),
        size: 10,
        file_type: "rs".to_string(),
        access_count: 1,
        estimated_duration_ms: 0,
        modification_count: 0,
        last_access_time: None,
        first_seen_time: None,
    }];
    let report = detect_mock_data_report(dir.path(), &entries);

    let guidance = data_risk_guidance(dir.path(), &report);
    let payload = project_data_risk_payload(
        MCP_DATA_RISK_V1,
        "demo",
        "all",
        "low",
        10,
        dir.path(),
        &entries,
    );

    assert_eq!(
        guidance["layers"]["execution_strategy"]["hardcoded_candidate_count"],
        json!(0)
    );
    assert_eq!(
        guidance["data_risk_focus"],
        json!({
            "primary_focus": "none",
            "priority_order": [],
            "basis": ["no_candidates_detected"]
        })
    );
    assert_eq!(payload["data_risk_focus"], guidance["data_risk_focus"]);
    assert_eq!(
        payload["guidance"]["layers"]["cleanup_refactor_candidates"]["hardcoded_data_candidates"],
        json!([])
    );
}

#[test]
fn data_risk_guidance_runtime_shared_weak_literal_pair_stays_none() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/customer_defaults.rs"),
        r#"const LABELS: &[&str] = &["customer", "order", "phone", "city"];"#,
    )
    .unwrap();
    let entries = vec![StatsEntry {
        file_path: "src/customer_defaults.rs".to_string(),
        size: 10,
        file_type: "rs".to_string(),
        access_count: 1,
        estimated_duration_ms: 0,
        modification_count: 0,
        last_access_time: None,
        first_seen_time: None,
    }];
    let report = detect_mock_data_report(dir.path(), &entries);

    let guidance = data_risk_guidance(dir.path(), &report);
    let payload = project_data_risk_payload(
        MCP_DATA_RISK_V1,
        "demo",
        "all",
        "low",
        10,
        dir.path(),
        &entries,
    );

    assert_eq!(
        guidance["layers"]["execution_strategy"]["hardcoded_candidate_count"],
        json!(0)
    );
    assert_eq!(
        guidance["data_risk_focus"],
        json!({
            "primary_focus": "none",
            "priority_order": [],
            "basis": ["no_candidates_detected"]
        })
    );
    assert_eq!(payload["data_risk_focus"], guidance["data_risk_focus"]);
}
