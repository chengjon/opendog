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
