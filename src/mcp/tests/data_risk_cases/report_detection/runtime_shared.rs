use super::*;

#[test]
fn detect_mock_data_report_runtime_shared_weak_token_only_stays_hardcoded_only() {
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

    assert!(report.mock_candidates.is_empty());
    assert_eq!(report.hardcoded_candidates.len(), 1);
    assert!(report.mixed_review_files.is_empty());
    assert_eq!(
        report.data_risk_focus(),
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
}

#[test]
fn detect_mock_data_report_runtime_shared_thin_combo_with_weak_path_token_is_not_hardcoded() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/customer_sample.rs"),
        r#"const CUSTOMER_EMAIL: &str = "ops@corp.com";"#,
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[StatsEntry {
            file_path: "src/customer_sample.rs".to_string(),
            size: 10,
            file_type: "rs".to_string(),
            access_count: 1,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }],
    );

    assert!(report.mock_candidates.is_empty());
    assert!(report.hardcoded_candidates.is_empty());
    assert!(report.mixed_review_files.is_empty());
    assert_eq!(
        report.data_risk_focus(),
        json!({
            "primary_focus": "none",
            "priority_order": [],
            "basis": ["no_candidates_detected"]
        })
    );
}

#[test]
fn detect_mock_data_report_runtime_shared_weak_literal_pair_is_not_hardcoded() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/customer_defaults.rs"),
        r#"const LABELS: &[&str] = &["customer", "order", "phone", "city"];"#,
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[StatsEntry {
            file_path: "src/customer_defaults.rs".to_string(),
            size: 10,
            file_type: "rs".to_string(),
            access_count: 1,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }],
    );

    assert!(report.mock_candidates.is_empty());
    assert!(report.hardcoded_candidates.is_empty());
    assert!(report.mixed_review_files.is_empty());
}

#[test]
fn detect_mock_data_report_runtime_shared_many_weak_literals_still_becomes_hardcoded() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/customer_profile.rs"),
        r#"const PROFILE: &[&str] = &["customer", "order", "price", "phone", "city", "postal", "zip", "usd"];"#,
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[StatsEntry {
            file_path: "src/customer_profile.rs".to_string(),
            size: 10,
            file_type: "rs".to_string(),
            access_count: 2,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }],
    );

    assert!(report.mock_candidates.is_empty());
    assert_eq!(report.hardcoded_candidates.len(), 1);
    assert!(report.hardcoded_candidates[0]
        .rule_hits
        .iter()
        .any(|hit| hit == "content.business_literal_combo"));
    assert!(report.hardcoded_candidates[0]
        .rule_hits
        .iter()
        .any(|hit| hit == "path.runtime_shared"));
}

#[test]
fn detect_mock_data_report_keeps_runtime_source_hardcoded_priority_high() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/customer_defaults.rs"),
        r#"const CUSTOMER: &str = "Acme Corp"; const EMAIL: &str = "ops@corp.com"; const ADDRESS: &str = "1 Market Street"; const PAYMENT: &str = "$100";"#,
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[StatsEntry {
            file_path: "src/customer_defaults.rs".to_string(),
            size: 10,
            file_type: "rs".to_string(),
            access_count: 2,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }],
    );

    assert_eq!(report.hardcoded_candidates.len(), 1);
    assert_eq!(
        report.hardcoded_candidates[0].path_classification,
        "runtime_shared"
    );
    assert_eq!(report.hardcoded_candidates[0].review_priority, "high");
    assert_eq!(report.hardcoded_candidates[0].confidence, "high");
    assert!(!report.hardcoded_candidates[0]
        .rule_hits
        .iter()
        .any(|hit| hit == "content.template_placeholder"));
}

#[test]
fn detect_mock_data_report_runtime_shared_weak_token_with_mock_content_stays_mock_candidate() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/customer_seed.rs"),
        r#"const LABEL: &str = "fixture payload";"#,
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[StatsEntry {
            file_path: "src/customer_seed.rs".to_string(),
            size: 10,
            file_type: "rs".to_string(),
            access_count: 1,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }],
    );

    assert_eq!(report.mock_candidates.len(), 1);
    assert_eq!(
        report.mock_candidates[0].path_classification,
        "runtime_shared"
    );
    assert!(report.mock_candidates[0]
        .rule_hits
        .iter()
        .any(|hit| hit == "path.mock_token"));
    assert!(report.mock_candidates[0]
        .rule_hits
        .iter()
        .any(|hit| hit == "content.mock_token"));
}
