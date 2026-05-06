use super::*;

#[test]
fn detect_mock_data_report_distinguishes_mock_and_hardcoded_candidates() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("tests/fixtures")).unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::create_dir_all(dir.path().join("dist")).unwrap();
    std::fs::write(
        dir.path().join("tests/fixtures/demo.json"),
        r#"{"mock": true, "customer": "Demo"}"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("src/customer_seed.rs"),
        r#"const CUSTOMER: &str = "Acme Corp"; const EMAIL: &str = "ops@corp.com"; const ADDRESS: &str = "1 Market Street";"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("dist/demo_seed.json"),
        r#"{"demo": true, "sample data": "yes"}"#,
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[
            StatsEntry {
                file_path: "tests/fixtures/demo.json".to_string(),
                size: 10,
                file_type: "json".to_string(),
                access_count: 0,
                estimated_duration_ms: 0,
                modification_count: 0,
                last_access_time: None,
                first_seen_time: None,
            },
            StatsEntry {
                file_path: "src/customer_seed.rs".to_string(),
                size: 10,
                file_type: "rs".to_string(),
                access_count: 2,
                estimated_duration_ms: 0,
                modification_count: 0,
                last_access_time: None,
                first_seen_time: None,
            },
            StatsEntry {
                file_path: "dist/demo_seed.json".to_string(),
                size: 10,
                file_type: "json".to_string(),
                access_count: 0,
                estimated_duration_ms: 0,
                modification_count: 0,
                last_access_time: None,
                first_seen_time: None,
            },
        ],
    );

    assert_eq!(report.mock_candidates.len(), 2);
    assert_eq!(report.hardcoded_candidates.len(), 1);
    assert_eq!(report.hardcoded_candidates[0].review_priority, "high");
    assert_eq!(
        report.hardcoded_candidates[0].path_classification,
        "runtime_shared"
    );
    assert!(report.hardcoded_candidates[0]
        .rule_hits
        .iter()
        .any(|hit| hit == "content.business_literal_combo"));
    assert!(report.mock_candidates.iter().any(|candidate| {
        candidate.path_classification == "generated_artifact" && candidate.review_priority == "low"
    }));
    let rendered = report.to_value(5);
    assert!(
        rendered["hardcoded_data_candidates"][0]["suggested_commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cmd| cmd == "git diff")
    );
    assert!(rendered["hardcoded_data_candidates"][0]["rule_hits"]
        .as_array()
        .unwrap()
        .iter()
        .any(|hit| hit["rule"] == "path.runtime_shared" && hit["severity"] == "high"));
    assert!(rendered["rule_groups_summary"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["group"] == "path" && item["severity"] == "low"));
    assert!(rendered["rule_hits_summary"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| {
            item["rule"] == "content.business_literal_combo"
                && item["severity"] == "high"
                && item["description"]
                    .as_str()
                    .unwrap()
                    .contains("business-like keywords")
        }));
    assert!(rendered["hardcoded_data_candidates"][0]["matched_keywords"]
        .as_array()
        .unwrap()
        .iter()
        .any(|kw| kw == "customer"));
    assert!(!report
        .mixed_review_files
        .iter()
        .any(|path| path == "src/customer_seed.rs"));
}

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
fn detect_mock_data_report_test_only_and_example_weak_tokens_remain_mock_candidates() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("tests/fixtures")).unwrap();
    std::fs::create_dir_all(dir.path().join("examples")).unwrap();
    std::fs::write(
        dir.path().join("tests/fixtures/sample.json"),
        r#"{"customer": "Demo"}"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("examples/seed.json"),
        r#"{"customer": "Demo"}"#,
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[
            StatsEntry {
                file_path: "tests/fixtures/sample.json".to_string(),
                size: 10,
                file_type: "json".to_string(),
                access_count: 0,
                estimated_duration_ms: 0,
                modification_count: 0,
                last_access_time: None,
                first_seen_time: None,
            },
            StatsEntry {
                file_path: "examples/seed.json".to_string(),
                size: 10,
                file_type: "json".to_string(),
                access_count: 0,
                estimated_duration_ms: 0,
                modification_count: 0,
                last_access_time: None,
                first_seen_time: None,
            },
        ],
    );

    assert_eq!(report.mock_candidates.len(), 2);
    assert!(report
        .mock_candidates
        .iter()
        .all(|candidate| candidate.path_classification == "test_only"));
    assert!(report.mock_candidates.iter().all(|candidate| {
        candidate
            .rule_hits
            .iter()
            .any(|hit| hit == "path.mock_token")
            && !candidate
                .rule_hits
                .iter()
                .any(|hit| hit == "content.mock_token")
    }));
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

#[test]
fn detect_mock_data_report_unknown_path_weak_token_only_is_not_mock_candidate() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("docs")).unwrap();
    std::fs::write(
        dir.path().join("docs/sample_notes.md"),
        "customer migration notes",
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[StatsEntry {
            file_path: "docs/sample_notes.md".to_string(),
            size: 10,
            file_type: "md".to_string(),
            access_count: 0,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }],
    );

    assert!(report.mock_candidates.is_empty());
}

#[test]
fn detect_mock_data_report_unknown_path_weak_token_with_mock_content_becomes_mock_candidate() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("docs")).unwrap();
    std::fs::write(
        dir.path().join("docs/sample_notes.md"),
        "fixture walkthrough for onboarding",
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[StatsEntry {
            file_path: "docs/sample_notes.md".to_string(),
            size: 10,
            file_type: "md".to_string(),
            access_count: 0,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }],
    );

    assert_eq!(report.mock_candidates.len(), 1);
    assert_eq!(report.mock_candidates[0].path_classification, "unknown");
    assert!(report.mock_candidates[0]
        .rule_hits
        .iter()
        .any(|hit| hit == "path.mock_token"));
    assert!(report.mock_candidates[0]
        .rule_hits
        .iter()
        .any(|hit| hit == "content.mock_token"));
}

#[test]
fn mock_data_report_filtering_respects_type_and_priority() {
    let report = super::MockDataReport {
        mock_candidates: vec![super::DataCandidate {
            file_path: "tests/demo.json".to_string(),
            confidence: "high",
            review_priority: "medium",
            path_classification: "test_only",
            rule_hits: vec!["path.test_only".to_string()],
            matched_keywords: vec!["demo".to_string()],
            reasons: vec!["demo".to_string()],
            evidence: vec!["demo".to_string()],
            access_count: 0,
            file_type: "json".to_string(),
        }],
        hardcoded_candidates: vec![super::DataCandidate {
            file_path: "src/customer_seed.rs".to_string(),
            confidence: "high",
            review_priority: "high",
            path_classification: "runtime_shared",
            rule_hits: vec!["path.runtime_shared".to_string()],
            matched_keywords: vec!["customer".to_string()],
            reasons: vec!["customer".to_string()],
            evidence: vec!["customer".to_string()],
            access_count: 1,
            file_type: "rs".to_string(),
        }],
        mixed_review_files: vec!["src/customer_seed.rs".to_string()],
    };

    assert_eq!(review_priority_score("medium"), 2);
    let filtered = report.filtered("hardcoded", Some("high"));
    assert_eq!(filtered.mock_candidates.len(), 0);
    assert_eq!(filtered.hardcoded_candidates.len(), 1);
    assert_eq!(filtered.mixed_review_files, vec!["src/customer_seed.rs"]);
}

#[test]
fn mock_data_report_derives_hardcoded_focus_from_runtime_shared_high_severity_hits() {
    let report = super::MockDataReport {
        mock_candidates: vec![],
        hardcoded_candidates: vec![super::DataCandidate {
            file_path: "src/customer_seed.rs".to_string(),
            confidence: "high",
            review_priority: "high",
            path_classification: "runtime_shared",
            rule_hits: vec![
                "path.runtime_shared".to_string(),
                "content.business_literal_combo".to_string(),
            ],
            matched_keywords: vec!["customer".to_string(), "email".to_string()],
            reasons: vec!["hardcoded".to_string()],
            evidence: vec!["runtime".to_string()],
            access_count: 1,
            file_type: "rs".to_string(),
        }],
        mixed_review_files: vec!["src/customer_seed.rs".to_string()],
    };

    assert_eq!(
        report.data_risk_focus(),
        json!({
            "primary_focus": "hardcoded",
            "priority_order": ["hardcoded", "mixed", "mock"],
            "basis": [
                "hardcoded_candidates_present",
                "mixed_review_files_present",
                "runtime_shared_candidates_present",
                "high_severity_content_hits_present"
            ]
        })
    );
}

#[test]
fn mock_data_report_derives_mixed_focus_when_mixed_files_exist_after_hardcoded_filtering() {
    let report = super::MockDataReport {
        mock_candidates: vec![super::DataCandidate {
            file_path: "src/demo.rs".to_string(),
            confidence: "medium",
            review_priority: "high",
            path_classification: "unknown",
            rule_hits: vec!["path.mock_token".to_string()],
            matched_keywords: vec!["demo".to_string()],
            reasons: vec!["mock".to_string()],
            evidence: vec!["demo".to_string()],
            access_count: 0,
            file_type: "rs".to_string(),
        }],
        hardcoded_candidates: vec![],
        mixed_review_files: vec!["src/demo.rs".to_string()],
    };

    assert_eq!(
        report.data_risk_focus(),
        json!({
            "primary_focus": "mixed",
            "priority_order": ["mixed", "hardcoded", "mock"],
            "basis": ["mixed_review_files_present"]
        })
    );
}

#[test]
fn mock_data_report_derives_mock_focus_when_only_mock_candidates_exist() {
    let report = super::MockDataReport {
        mock_candidates: vec![super::DataCandidate {
            file_path: "tests/fixtures/demo.json".to_string(),
            confidence: "high",
            review_priority: "medium",
            path_classification: "test_only",
            rule_hits: vec!["path.test_only".to_string()],
            matched_keywords: vec!["demo".to_string()],
            reasons: vec!["mock".to_string()],
            evidence: vec!["fixture".to_string()],
            access_count: 0,
            file_type: "json".to_string(),
        }],
        hardcoded_candidates: vec![],
        mixed_review_files: vec![],
    };

    assert_eq!(
        report.data_risk_focus(),
        json!({
            "primary_focus": "mock",
            "priority_order": ["mock", "hardcoded", "mixed"],
            "basis": ["mock_candidates_present"]
        })
    );
}

#[test]
fn mock_data_report_derives_none_focus_when_no_candidates_exist() {
    let report = super::MockDataReport::default();

    assert_eq!(
        report.data_risk_focus(),
        json!({
            "primary_focus": "none",
            "priority_order": [],
            "basis": ["no_candidates_detected"]
        })
    );
}
