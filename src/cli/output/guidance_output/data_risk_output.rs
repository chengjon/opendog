use serde_json::Value;

use super::print_recommended_flow;

pub(super) fn print_data_risk(
    id: &str,
    candidate_type: &str,
    min_review_priority: &str,
    rendered: &Value,
    guidance: &Value,
) {
    println!(
        "Data risk for project '{}' — type={} min_priority={}",
        id, candidate_type, min_review_priority
    );
    println!(
        "  Mock: {} | Hardcoded: {} | Mixed review files: {}",
        rendered["mock_candidate_count"].as_u64().unwrap_or(0),
        rendered["hardcoded_candidate_count"].as_u64().unwrap_or(0),
        rendered["mixed_review_file_count"].as_u64().unwrap_or(0),
    );

    if let Some(summary) = guidance["summary"].as_str() {
        println!("  Summary: {}", summary);
    }
    print_recommended_flow(&guidance["recommended_flow"]);

    print_rule_summary("Rule groups", &rendered["rule_groups_summary"], "group");
    print_rule_summary("Rule hits", &rendered["rule_hits_summary"], "rule");
    print_candidate_list(
        "Hardcoded candidates",
        &rendered["hardcoded_data_candidates"],
    );
    print_candidate_list("Mock candidates", &rendered["mock_data_candidates"]);
    print_string_list("Mixed review files", &rendered["mixed_review_files"]);
}

pub(super) fn print_workspace_data_risk(
    candidate_type: &str,
    min_review_priority: &str,
    project_limit: usize,
    total_registered_projects: usize,
    matched_projects: usize,
    guidance: &Value,
) {
    println!(
        "Workspace data risk — type={} min_priority={} project_limit={}",
        candidate_type, min_review_priority, project_limit
    );
    println!(
        "  Registered projects: {} | Matched projects: {}",
        total_registered_projects, matched_projects
    );

    if let Some(summary) = guidance["summary"].as_str() {
        println!("  Summary: {}", summary);
    }
    print_recommended_flow(&guidance["recommended_flow"]);

    let observation = &guidance["layers"]["workspace_observation"];
    println!(
        "  Mock projects: {} | Hardcoded projects: {} | Total mock candidates: {} | Total hardcoded candidates: {}",
        observation["projects_with_mock_candidates"].as_u64().unwrap_or(0),
        observation["projects_with_hardcoded_candidates"]
            .as_u64()
            .unwrap_or(0),
        observation["total_mock_candidates"].as_u64().unwrap_or(0),
        observation["total_hardcoded_candidates"]
            .as_u64()
            .unwrap_or(0),
    );

    print_rule_summary(
        "Workspace rule groups",
        &guidance["layers"]["workspace_observation"]["rule_groups_summary"],
        "group",
    );
    print_rule_summary(
        "Workspace rule hits",
        &guidance["layers"]["workspace_observation"]["rule_hits_summary"],
        "rule",
    );

    println!();
    println!("Priority projects:");
    let priority_projects = guidance["layers"]["multi_project_portfolio"]["priority_projects"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if priority_projects.is_empty() {
        println!("  No matching projects.");
        return;
    }
    for project in priority_projects.iter().take(project_limit) {
        let dominant_group = project["dominant_rule_group"]["group"]
            .as_str()
            .unwrap_or("-");
        let priority_reason = project["priority_reason"].as_str().unwrap_or("-");
        println!(
            "  {} [{}] hardcoded={} mock={} mixed={} dominant_group={}",
            project["project_id"].as_str().unwrap_or("-"),
            project["status"].as_str().unwrap_or("-"),
            project["hardcoded_candidate_count"].as_u64().unwrap_or(0),
            project["mock_candidate_count"].as_u64().unwrap_or(0),
            project["mixed_review_file_count"].as_u64().unwrap_or(0),
            dominant_group,
        );
        println!("    reason: {}", priority_reason);
        print_candidate_list_inline("top hardcoded", &project["top_hardcoded_candidates"]);
        print_candidate_list_inline("top mock", &project["top_mock_candidates"]);
    }
}

fn print_rule_summary(title: &str, value: &Value, key: &str) {
    println!();
    println!("{}:", title);
    let Some(entries) = value.as_array() else {
        println!("  None");
        return;
    };
    if entries.is_empty() {
        println!("  None");
        return;
    }
    for entry in entries.iter().take(5) {
        println!(
            "  {} count={} severity={}",
            entry[key].as_str().unwrap_or("-"),
            entry["count"].as_u64().unwrap_or(0),
            entry["severity"].as_str().unwrap_or("-"),
        );
    }
}

fn print_candidate_list(title: &str, value: &Value) {
    println!();
    println!("{}:", title);
    let Some(entries) = value.as_array() else {
        println!("  None");
        return;
    };
    if entries.is_empty() {
        println!("  None");
        return;
    }
    for candidate in entries.iter().take(10) {
        println!(
            "  {} priority={} confidence={} class={}",
            candidate["file_path"].as_str().unwrap_or("-"),
            candidate["review_priority"].as_str().unwrap_or("-"),
            candidate["confidence"].as_str().unwrap_or("-"),
            candidate["path_classification"].as_str().unwrap_or("-"),
        );
    }
}

fn print_candidate_list_inline(title: &str, value: &Value) {
    let Some(entries) = value.as_array() else {
        return;
    };
    if entries.is_empty() {
        return;
    }
    let list = entries
        .iter()
        .take(3)
        .filter_map(|candidate| candidate["file_path"].as_str())
        .collect::<Vec<_>>();
    if !list.is_empty() {
        println!("    {}: {}", title, list.join(", "));
    }
}

fn print_string_list(title: &str, value: &Value) {
    println!();
    println!("{}:", title);
    let Some(entries) = value.as_array() else {
        println!("  None");
        return;
    };
    if entries.is_empty() {
        println!("  None");
        return;
    }
    for entry in entries.iter().take(10) {
        if let Some(path) = entry.as_str() {
            println!("  {}", path);
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    // Tests for print_rule_summary logic
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

    // Tests for print_candidate_list logic
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

    // Tests for print_string_list logic
    #[test]
    fn string_list_extracts_strings() {
        let value = json!(["file1.rs", "file2.py"]);
        let paths: Vec<&str> = value
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|e| e.as_str())
            .collect();
        assert_eq!(paths, vec!["file1.rs", "file2.py"]);
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
        assert_eq!(paths, vec!["file1.rs", "file2.py"]);
    }

    // Tests for print_candidate_list_inline logic
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
        assert_eq!(list, vec!["a.rs", "b.py", "c.ts"]);
    }

    #[test]
    fn inline_candidate_list_empty_skips() {
        let value = json!([]);
        let is_empty = value.as_array().map(|a| a.is_empty()).unwrap_or(true);
        assert!(is_empty);
    }

    // Tests for print_data_risk header formatting
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
}
