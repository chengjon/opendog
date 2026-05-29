use super::*;
use crate::mcp::data_risk::{DataCandidate, MockDataReport};

fn make_candidate(
    file_path: &str,
    review_priority: &'static str,
    rule_hits: Vec<&str>,
) -> DataCandidate {
    DataCandidate {
        file_path: file_path.to_string(),
        confidence: "medium",
        review_priority,
        path_classification: "unknown",
        rule_hits: rule_hits.into_iter().map(|s| s.to_string()).collect(),
        matched_keywords: vec![],
        reasons: vec![],
        evidence: vec![],
        access_count: 0,
        file_type: String::new(),
    }
}

// ---- data_risk_focus ----

#[test]
fn test_data_risk_focus_no_candidates() {
    let report = MockDataReport::default();
    let focus = report.data_risk_focus();
    assert_eq!(focus["primary_focus"], "none");
    assert_eq!(focus["priority_order"], json!([]));
    assert_eq!(focus["basis"], json!(["no_candidates_detected"]));
}

#[test]
fn test_data_risk_focus_mock_only() {
    let report = MockDataReport {
        mock_candidates: vec![make_candidate("mock.py", "medium", vec!["path.mock_token"])],
        hardcoded_candidates: vec![],
        mixed_review_files: vec![],
    };
    let focus = report.data_risk_focus();
    assert_eq!(focus["primary_focus"], "mock");
    assert_eq!(
        focus["priority_order"],
        json!(["mock", "hardcoded", "mixed"])
    );
    assert_eq!(focus["basis"], json!(["mock_candidates_present"]));
}

#[test]
fn test_data_risk_focus_hardcoded_only() {
    let report = MockDataReport {
        mock_candidates: vec![],
        hardcoded_candidates: vec![make_candidate(
            "data.py",
            "high",
            vec!["content.business_literal_combo"],
        )],
        mixed_review_files: vec![],
    };
    let focus = report.data_risk_focus();
    // hardcoded present with high_severity_content -> "hardcoded"
    assert_eq!(focus["primary_focus"], "hardcoded");
}

#[test]
fn test_data_risk_focus_hardcoded_with_runtime_shared() {
    let report = MockDataReport {
        mock_candidates: vec![],
        hardcoded_candidates: vec![make_candidate(
            "src/data.py",
            "high",
            vec!["path.runtime_shared"],
        )],
        mixed_review_files: vec![],
    };
    let focus = report.data_risk_focus();
    assert_eq!(focus["primary_focus"], "hardcoded");
    assert_eq!(
        focus["priority_order"],
        json!(["hardcoded", "mixed", "mock"])
    );
    assert!(
        (focus["basis"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b == "runtime_shared_candidates_present"))
    );
}

#[test]
fn test_data_risk_focus_hardcoded_with_business_literal_combo() {
    let report = MockDataReport {
        mock_candidates: vec![],
        hardcoded_candidates: vec![make_candidate(
            "data.py",
            "high",
            vec!["content.business_literal_combo"],
        )],
        mixed_review_files: vec![],
    };
    let focus = report.data_risk_focus();
    assert_eq!(focus["primary_focus"], "hardcoded");
    assert!(focus["basis"]
        .as_array()
        .unwrap()
        .iter()
        .any(|b| b == "high_severity_content_hits_present"));
}

#[test]
fn test_data_risk_focus_mixed() {
    let report = MockDataReport {
        mock_candidates: vec![make_candidate("mock.py", "medium", vec!["path.mock_token"])],
        hardcoded_candidates: vec![make_candidate(
            "mock.py",
            "high",
            vec!["content.business_literal_combo"],
        )],
        mixed_review_files: vec!["mock.py".to_string()],
    };
    let focus = report.data_risk_focus();
    assert_eq!(focus["primary_focus"], "hardcoded");
    assert!(focus["basis"]
        .as_array()
        .unwrap()
        .iter()
        .any(|b| b == "mixed_review_files_present"));
}

#[test]
fn test_data_risk_focus_mixed_without_hardcoded_runtime() {
    // mixed files present but hardcoded has no runtime_shared or business_literal_combo
    // However has_hardcoded && has_mixed is true, so first branch matches
    let report = MockDataReport {
        mock_candidates: vec![],
        hardcoded_candidates: vec![make_candidate(
            "data.py",
            "medium",
            vec!["content.template_placeholder"],
        )],
        mixed_review_files: vec!["data.py".to_string()],
    };
    let focus = report.data_risk_focus();
    // has_hardcoded && has_mixed -> first branch -> "hardcoded"
    assert_eq!(focus["primary_focus"], "hardcoded");
}

// ---- filtered ----

#[test]
fn test_filtered_by_type_mock() {
    let report = MockDataReport {
        mock_candidates: vec![make_candidate("mock.py", "medium", vec!["path.mock_token"])],
        hardcoded_candidates: vec![make_candidate(
            "data.py",
            "high",
            vec!["content.business_literal_combo"],
        )],
        mixed_review_files: vec!["mock.py".to_string(), "data.py".to_string()],
    };
    let filtered = report.filtered("mock", None);
    assert_eq!(filtered.mock_candidates.len(), 1);
    assert!(filtered.hardcoded_candidates.is_empty());
    assert_eq!(filtered.mixed_review_files.len(), 1);
    assert_eq!(filtered.mixed_review_files[0], "mock.py");
}

#[test]
fn test_filtered_by_type_hardcoded() {
    let report = MockDataReport {
        mock_candidates: vec![make_candidate("mock.py", "medium", vec!["path.mock_token"])],
        hardcoded_candidates: vec![make_candidate(
            "data.py",
            "high",
            vec!["content.business_literal_combo"],
        )],
        mixed_review_files: vec!["mock.py".to_string(), "data.py".to_string()],
    };
    let filtered = report.filtered("hardcoded", None);
    assert!(filtered.mock_candidates.is_empty());
    assert_eq!(filtered.hardcoded_candidates.len(), 1);
    assert_eq!(filtered.mixed_review_files.len(), 1);
    assert_eq!(filtered.mixed_review_files[0], "data.py");
}

#[test]
fn test_filtered_by_type_all() {
    let report = MockDataReport {
        mock_candidates: vec![make_candidate("mock.py", "medium", vec!["path.mock_token"])],
        hardcoded_candidates: vec![make_candidate(
            "data.py",
            "high",
            vec!["content.business_literal_combo"],
        )],
        mixed_review_files: vec!["mock.py".to_string(), "data.py".to_string()],
    };
    let filtered = report.filtered("all", None);
    assert_eq!(filtered.mock_candidates.len(), 1);
    assert_eq!(filtered.hardcoded_candidates.len(), 1);
    assert_eq!(filtered.mixed_review_files.len(), 2);
}

#[test]
fn test_filtered_by_priority_threshold() {
    let report = MockDataReport {
        mock_candidates: vec![
            make_candidate("low.py", "low", vec!["path.mock_token"]),
            make_candidate("med.py", "medium", vec!["path.mock_token"]),
            make_candidate("hi.py", "high", vec!["path.mock_token"]),
        ],
        hardcoded_candidates: vec![],
        mixed_review_files: vec![],
    };
    let filtered = report.filtered("all", Some("medium"));
    assert_eq!(filtered.mock_candidates.len(), 2); // medium + high
}

#[test]
fn test_filtered_by_priority_high_only() {
    let report = MockDataReport {
        mock_candidates: vec![
            make_candidate("low.py", "low", vec!["path.mock_token"]),
            make_candidate("med.py", "medium", vec!["path.mock_token"]),
            make_candidate("hi.py", "high", vec!["path.mock_token"]),
        ],
        hardcoded_candidates: vec![],
        mixed_review_files: vec![],
    };
    let filtered = report.filtered("all", Some("high"));
    assert_eq!(filtered.mock_candidates.len(), 1);
    assert_eq!(filtered.mock_candidates[0].file_path, "hi.py");
}

// ---- to_value ----

#[test]
fn test_to_value_counts() {
    let report = MockDataReport {
        mock_candidates: vec![make_candidate("a.py", "medium", vec!["path.mock_token"])],
        hardcoded_candidates: vec![
            make_candidate("b.py", "high", vec!["content.business_literal_combo"]),
            make_candidate("c.py", "low", vec!["path.documentation"]),
        ],
        mixed_review_files: vec![],
    };
    let val = report.to_value(100);
    assert_eq!(val["mock_candidate_count"], 1);
    assert_eq!(val["hardcoded_candidate_count"], 2);
    assert_eq!(val["mixed_review_file_count"], 0);
}

#[test]
fn test_to_value_respects_limit() {
    let report = MockDataReport {
        mock_candidates: vec![
            make_candidate("a.py", "medium", vec!["path.mock_token"]),
            make_candidate("b.py", "medium", vec!["path.mock_token"]),
            make_candidate("c.py", "medium", vec!["path.mock_token"]),
        ],
        hardcoded_candidates: vec![],
        mixed_review_files: vec!["x.py".to_string(), "y.py".to_string(), "z.py".to_string()],
    };
    let val = report.to_value(2);
    assert_eq!(val["mock_candidate_count"], 3); // count is total
    let mock_list = val["mock_data_candidates"].as_array().unwrap();
    assert_eq!(mock_list.len(), 2); // but list is limited
    let mixed_list = val["mixed_review_files"].as_array().unwrap();
    assert_eq!(mixed_list.len(), 2);
}

#[test]
fn test_to_value_has_focus_and_summaries() {
    let report = MockDataReport::default();
    let val = report.to_value(100);
    assert!(val["data_risk_focus"].is_object());
    assert!(val["rule_hits_summary"].is_array());
    assert!(val["rule_groups_summary"].is_array());
}

// ---- rule_hits_summary ----

#[test]
fn test_rule_hits_summary_aggregation() {
    let report = MockDataReport {
        mock_candidates: vec![make_candidate(
            "a.py",
            "medium",
            vec!["path.mock_token", "path.test_only"],
        )],
        hardcoded_candidates: vec![make_candidate(
            "b.py",
            "high",
            vec!["path.mock_token", "content.business_literal_combo"],
        )],
        mixed_review_files: vec![],
    };
    let summary = report.rule_hits_summary();
    let arr = summary.as_array().unwrap();
    // path.mock_token appears twice
    let mock_token_entry = arr.iter().find(|e| e["rule"] == "path.mock_token").unwrap();
    assert_eq!(mock_token_entry["count"], 2);
    // path.test_only appears once
    let test_only_entry = arr.iter().find(|e| e["rule"] == "path.test_only").unwrap();
    assert_eq!(test_only_entry["count"], 1);
    // content.business_literal_combo appears once
    let combo_entry = arr
        .iter()
        .find(|e| e["rule"] == "content.business_literal_combo")
        .unwrap();
    assert_eq!(combo_entry["count"], 1);
}

#[test]
fn test_rule_hits_summary_empty() {
    let report = MockDataReport::default();
    let summary = report.rule_hits_summary();
    assert!(summary.as_array().unwrap().is_empty());
}

#[test]
fn test_rule_hits_summary_unknown_rule() {
    let report = MockDataReport {
        mock_candidates: vec![DataCandidate {
            file_path: "x.py".to_string(),
            confidence: "medium",
            review_priority: "medium",
            path_classification: "unknown",
            rule_hits: vec!["custom.unknown_rule".to_string()],
            matched_keywords: vec![],
            reasons: vec![],
            evidence: vec![],
            access_count: 0,
            file_type: String::new(),
        }],
        hardcoded_candidates: vec![],
        mixed_review_files: vec![],
    };
    let summary = report.rule_hits_summary();
    let arr = summary.as_array().unwrap();
    let entry = &arr[0];
    assert_eq!(entry["rule"], "custom.unknown_rule");
    assert_eq!(entry["severity"], "unknown");
    assert_eq!(entry["count"], 1);
}

// ---- rule_groups_summary ----

#[test]
fn test_rule_groups_summary_aggregation() {
    let report = MockDataReport {
        mock_candidates: vec![make_candidate("a.py", "medium", vec!["path.mock_token"])],
        hardcoded_candidates: vec![make_candidate(
            "b.py",
            "high",
            vec!["path.mock_token", "content.business_literal_combo"],
        )],
        mixed_review_files: vec![],
    };
    let summary = report.rule_groups_summary();
    let arr = summary.as_array().unwrap();
    // path group: path.mock_token x2
    let path_entry = arr.iter().find(|e| e["group"] == "path").unwrap();
    assert_eq!(path_entry["count"], 2);
    // content group: content.business_literal_combo x1
    let content_entry = arr.iter().find(|e| e["group"] == "content").unwrap();
    assert_eq!(content_entry["count"], 1);
}

#[test]
fn test_rule_groups_summary_empty() {
    let report = MockDataReport::default();
    let summary = report.rule_groups_summary();
    assert!(summary.as_array().unwrap().is_empty());
}

#[test]
fn test_rule_groups_summary_severity_mapping() {
    let report = MockDataReport {
        mock_candidates: vec![make_candidate("a.py", "medium", vec!["path.mock_token"])],
        hardcoded_candidates: vec![make_candidate(
            "b.py",
            "high",
            vec!["content.business_literal_combo"],
        )],
        mixed_review_files: vec![],
    };
    let summary = report.rule_groups_summary();
    let arr = summary.as_array().unwrap();
    // "path" group -> "low"
    let path_entry = arr.iter().find(|e| e["group"] == "path").unwrap();
    assert_eq!(path_entry["severity"], "low");
    // "content" group -> "medium"
    let content_entry = arr.iter().find(|e| e["group"] == "content").unwrap();
    assert_eq!(content_entry["severity"], "medium");
}
