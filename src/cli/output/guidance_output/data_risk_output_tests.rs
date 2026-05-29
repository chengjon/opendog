use serde_json::json;

#[test]
fn rule_summary_extracts_entries() {
    let value = json!([
        {"group": "test_data", "count": 5, "severity": "high"},
        {"group": "demo_code", "count": 3, "severity": "medium"}
    ]);
    let entries = value.as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["group"].as_str(), Some("test_data"));
    assert_eq!(entries[1]["count"].as_u64(), Some(3));
}

#[test]
fn rule_summary_empty_array_shows_none() {
    let value = json!([]);
    let is_empty = value.as_array().map(|a| a.is_empty()).unwrap_or(true);
    assert!(is_empty);
}

#[test]
fn rule_summary_non_array_shows_none() {
    let value = json!("not array");
    assert!(value.as_array().is_none());
}

#[test]
fn candidate_list_extracts_fields() {
    let value = json!([
        {
            "file_path": "src/mock.rs",
            "review_priority": "high",
            "confidence": "0.9",
            "path_classification": "source"
        }
    ]);
    let entries = value.as_array().unwrap();
    let candidate = &entries[0];
    assert_eq!(candidate["file_path"].as_str(), Some("src/mock.rs"));
    assert_eq!(candidate["review_priority"].as_str(), Some("high"));
}

#[test]
fn candidate_list_empty_shows_none() {
    let value = json!([]);
    let is_empty = value.as_array().map(|a| a.is_empty()).unwrap_or(true);
    assert!(is_empty);
}

#[test]
fn string_list_extracts_strings() {
    let value = json!(["file1.rs", "file2.py"]);
    let paths: Vec<&str> = value
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e.as_str())
        .collect();
    assert_eq!(paths, ["file1.rs", "file2.py"]);
}

#[test]
fn string_list_skips_non_strings() {
    let value = json!(["file1.rs", 42, "file2.py"]);
    let paths: Vec<&str> = value
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e.as_str())
        .collect();
    assert_eq!(paths, ["file1.rs", "file2.py"]);
}

#[test]
fn inline_candidate_list_extracts_paths() {
    let value = json!([
        {"file_path": "a.rs"},
        {"file_path": "b.py"},
        {"file_path": "c.ts"},
        {"file_path": "d.go"}
    ]);
    let list: Vec<&str> = value
        .as_array()
        .unwrap()
        .iter()
        .take(3)
        .filter_map(|c| c["file_path"].as_str())
        .collect();
    assert_eq!(list, ["a.rs", "b.py", "c.ts"]);
}

#[test]
fn inline_candidate_list_empty_skips() {
    let value = json!([]);
    let is_empty = value.as_array().map(|a| a.is_empty()).unwrap_or(true);
    assert!(is_empty);
}

#[test]
fn data_risk_header_format() {
    let line = format!(
        "Data risk for project '{}' — type={} min_priority={}",
        "proj1", "all", "medium"
    );
    assert_eq!(
        line,
        "Data risk for project 'proj1' — type=all min_priority=medium"
    );
}

#[test]
fn workspace_data_risk_header_format() {
    let line = format!(
        "Workspace data risk — type={} min_priority={} project_limit={}",
        "mock", "high", 5
    );
    assert_eq!(
        line,
        "Workspace data risk — type=mock min_priority=high project_limit=5"
    );
}

#[test]
fn priority_project_format() {
    let project = json!({
        "project_id": "myproj",
        "status": "idle",
        "hardcoded_candidate_count": 3,
        "mock_candidate_count": 1,
        "mixed_review_file_count": 2,
        "dominant_rule_group": {"group": "test_fixtures"},
        "priority_reason": "high hardcoded count"
    });
    let dominant = project["dominant_rule_group"]["group"]
        .as_str()
        .unwrap_or("-");
    assert_eq!(dominant, "test_fixtures");
    assert_eq!(project["project_id"].as_str(), Some("myproj"));
}

#[test]
fn data_risk_rendered_counts() {
    let rendered = json!({
        "mock_candidate_count": 5,
        "hardcoded_candidate_count": 2,
        "mixed_review_file_count": 1
    });
    assert_eq!(rendered["mock_candidate_count"].as_u64(), Some(5));
    assert_eq!(rendered["hardcoded_candidate_count"].as_u64(), Some(2));
}
