use super::*;

#[test]
fn data_risk_guidance_surfaces_counts_and_candidates() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/customer_seed.rs"),
        r#"const CUSTOMER: &str = "Acme Corp"; const EMAIL: &str = "ops@corp.com"; const ADDRESS: &str = "1 Market Street";"#,
    )
    .unwrap();
    let report = detect_mock_data_report(
        dir.path(),
        &[StatsEntry {
            file_path: "src/customer_seed.rs".to_string(),
            size: 10,
            file_type: "rs".to_string(),
            access_count: 2,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }],
    );

    let guidance = data_risk_guidance(dir.path(), &report);
    assert_eq!(
        guidance["layers"]["execution_strategy"]["hardcoded_candidate_count"],
        json!(1)
    );
    assert_eq!(
        guidance["layers"]["cleanup_refactor_candidates"]["hardcoded_data_candidates"][0]
            ["file_path"],
        json!("src/customer_seed.rs")
    );
    assert_eq!(
        guidance["recommended_flow"][0],
        json!("Review high-priority hardcoded-data candidates first.")
    );
    assert!(guidance["recommended_flow"]
        .as_array()
        .unwrap()
        .iter()
        .any(|step| step.as_str().unwrap().contains("verification evidence")));
}
