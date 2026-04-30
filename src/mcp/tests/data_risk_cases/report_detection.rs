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

    assert_eq!(report.mock_candidates.len(), 3);
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
    assert!(report
        .mixed_review_files
        .iter()
        .any(|path| path == "src/customer_seed.rs"));
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
