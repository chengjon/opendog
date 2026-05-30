use super::*;

#[test]
fn detect_mock_data_report_downgrades_markdown_template_hardcoded_noise() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("docs/operations")).unwrap();
    std::fs::write(
        dir.path().join("docs/operations/DEPLOYMENT.md"),
        r#"
Set CLIENT_EMAIL=${CLIENT_EMAIL}, ORDER_AMOUNT=${ORDER_AMOUNT}, USER_ADDRESS=${USER_ADDRESS}.
Example placeholders may look like customer@example.com, $ORDER_AMOUNT, or 1 Market Street.
This deployment guide describes customer, client, order, amount, email, user, address, and payment placeholders.
"#,
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[StatsEntry {
            file_path: "docs/operations/DEPLOYMENT.md".to_string(),
            size: 10,
            file_type: "md".to_string(),
            access_count: 0,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }],
    );

    assert_eq!(report.hardcoded_candidates.len(), 1);
    assert_eq!(
        report.hardcoded_candidates[0].path_classification,
        "documentation"
    );
    assert_eq!(report.hardcoded_candidates[0].review_priority, "low");
    assert_eq!(report.hardcoded_candidates[0].confidence, "low");
    assert!(report.hardcoded_candidates[0]
        .rule_hits
        .iter()
        .any(|hit| hit == "path.documentation"));
    assert!(report.hardcoded_candidates[0]
        .rule_hits
        .iter()
        .any(|hit| hit == "content.template_placeholder"));
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
    assert_eq!(
        report.mock_candidates[0].path_classification,
        "documentation"
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
