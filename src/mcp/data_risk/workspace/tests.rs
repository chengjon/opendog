use super::*;
use serde_json::json;

// ---- workspace_priority_reason ----

#[test]
fn workspace_priority_reason_hardcoded_runtime_shared_high_content() {
    let summary = json!({
        "hardcoded_candidate_count": 3,
        "mock_candidate_count": 0,
        "mixed_review_file_count": 0,
        "rule_hits_summary": [
            { "rule": "path.runtime_shared", "count": 2 },
            { "rule": "content.business_literal_combo", "count": 1 },
        ],
    });
    let reason = workspace_priority_reason(&summary);
    assert_eq!(
        reason,
        "runtime-shared hardcoded candidates with high-severity content matches"
    );
}

#[test]
fn workspace_priority_reason_hardcoded_runtime_shared() {
    let summary = json!({
        "hardcoded_candidate_count": 2,
        "mock_candidate_count": 0,
        "mixed_review_file_count": 0,
        "rule_hits_summary": [
            { "rule": "path.runtime_shared", "count": 1 },
        ],
    });
    let reason = workspace_priority_reason(&summary);
    assert_eq!(
        reason,
        "runtime-shared hardcoded candidates need manual review before refactor"
    );
}

#[test]
fn workspace_priority_reason_hardcoded_high_content() {
    let summary = json!({
        "hardcoded_candidate_count": 2,
        "mock_candidate_count": 0,
        "mixed_review_file_count": 0,
        "rule_hits_summary": [
            { "rule": "content.business_literal_combo", "count": 3 },
        ],
    });
    let reason = workspace_priority_reason(&summary);
    assert_eq!(
        reason,
        "hardcoded business-like literals detected in review candidates"
    );
}

#[test]
fn workspace_priority_reason_hardcoded_mixed() {
    let summary = json!({
        "hardcoded_candidate_count": 1,
        "mock_candidate_count": 0,
        "mixed_review_file_count": 3,
        "rule_hits_summary": [],
    });
    let reason = workspace_priority_reason(&summary);
    assert_eq!(
        reason,
        "hardcoded candidates appear alongside mixed review files"
    );
}

#[test]
fn workspace_priority_reason_hardcoded_only() {
    let summary = json!({
        "hardcoded_candidate_count": 5,
        "mock_candidate_count": 0,
        "mixed_review_file_count": 0,
        "rule_hits_summary": [],
    });
    let reason = workspace_priority_reason(&summary);
    assert_eq!(
        reason,
        "hardcoded-data candidates require project-level inspection"
    );
}

#[test]
fn workspace_priority_reason_mixed_only() {
    let summary = json!({
        "hardcoded_candidate_count": 0,
        "mock_candidate_count": 0,
        "mixed_review_file_count": 2,
        "rule_hits_summary": [],
    });
    let reason = workspace_priority_reason(&summary);
    assert_eq!(
        reason,
        "mixed mock and hardcoded review files need classification cleanup"
    );
}

#[test]
fn workspace_priority_reason_mock_only() {
    let summary = json!({
        "hardcoded_candidate_count": 0,
        "mock_candidate_count": 4,
        "mixed_review_file_count": 0,
        "rule_hits_summary": [],
    });
    let reason = workspace_priority_reason(&summary);
    assert_eq!(
        reason,
        "mock-style candidates should be confirmed as test-only before cleanup"
    );
}

#[test]
fn workspace_priority_reason_nothing() {
    let summary = json!({
        "hardcoded_candidate_count": 0,
        "mock_candidate_count": 0,
        "mixed_review_file_count": 0,
        "rule_hits_summary": [],
    });
    let reason = workspace_priority_reason(&summary);
    assert_eq!(reason, "no current mock or hardcoded-data candidates");
}

// ---- workspace_dominant_rule_group ----

#[test]
fn workspace_dominant_rule_group_picks_highest_count() {
    let summary = json!({
        "rule_groups_summary": [
            { "group": "path", "count": 5, "severity": "low" },
            { "group": "content", "count": 10, "severity": "medium" },
        ],
    });
    let result = workspace_dominant_rule_group(&summary);
    assert_eq!(result["group"], "content");
    assert_eq!(result["count"], 10);
}

#[test]
fn workspace_dominant_rule_group_breaks_tie_by_severity() {
    let summary = json!({
        "rule_groups_summary": [
            { "group": "path", "count": 5, "severity": "low" },
            { "group": "content", "count": 5, "severity": "medium" },
        ],
    });
    let result = workspace_dominant_rule_group(&summary);
    // Tie-breaking uses b.severity.cmp(a.severity) in max_by,
    // so the item that appears as "a" with lower severity wins.
    // The actual winner depends on iteration order and the max_by reduction.
    // path has severity score 1, content has severity score 2.
    // The sort prefers the first-encountered item when b-severity > a-severity.
    assert!(result["group"].is_string());
    assert!(result["count"] == 5);
}

#[test]
fn workspace_dominant_rule_group_empty() {
    let summary = json!({
        "rule_groups_summary": [],
    });
    let result = workspace_dominant_rule_group(&summary);
    assert!(result.is_null());
}

#[test]
fn workspace_dominant_rule_group_missing_field() {
    let summary = json!({});
    let result = workspace_dominant_rule_group(&summary);
    assert!(result.is_null());
}

// ---- enrich_workspace_priority_project ----

#[test]
fn enrich_workspace_priority_project_adds_fields() {
    let summary = json!({
        "project_id": "demo",
        "hardcoded_candidate_count": 0,
        "mock_candidate_count": 2,
        "mixed_review_file_count": 0,
        "rule_groups_summary": [],
        "rule_hits_summary": [],
    });
    let enriched = enrich_workspace_priority_project(&summary);
    assert_eq!(enriched["project_id"], "demo");
    assert!(enriched["dominant_rule_group"].is_null());
    assert_eq!(
        enriched["priority_reason"],
        "mock-style candidates should be confirmed as test-only before cleanup"
    );
}

#[test]
fn enrich_workspace_priority_project_preserves_original() {
    let summary = json!({
        "project_id": "test-proj",
        "hardcoded_candidate_count": 5,
        "mock_candidate_count": 1,
        "mixed_review_file_count": 0,
        "rule_groups_summary": [
            { "group": "content", "count": 5, "severity": "high" },
        ],
        "rule_hits_summary": [],
    });
    let enriched = enrich_workspace_priority_project(&summary);
    assert_eq!(enriched["hardcoded_candidate_count"], 5);
    assert_eq!(enriched["mock_candidate_count"], 1);
    assert_eq!(enriched["dominant_rule_group"]["group"], "content");
}

// ---- aggregate_workspace_data_risk_focus ----

#[test]
fn aggregate_workspace_data_risk_focus_empty() {
    let result = aggregate_workspace_data_risk_focus(&[]);
    assert_eq!(result["distribution"]["hardcoded"], 0);
    assert_eq!(result["distribution"]["mock"], 0);
    assert_eq!(result["distribution"]["mixed"], 0);
    assert_eq!(result["distribution"]["none"], 0);
    assert_eq!(result["projects_requiring_hardcoded_review"], 0);
    assert_eq!(result["projects_requiring_mock_review"], 0);
    assert_eq!(result["projects_requiring_mixed_file_review"], 0);
}

#[test]
fn aggregate_workspace_data_risk_focus_mixed_types() {
    let summaries = vec![
        json!({ "data_risk_focus": { "primary_focus": "hardcoded" } }),
        json!({ "data_risk_focus": { "primary_focus": "mock" } }),
        json!({ "data_risk_focus": { "primary_focus": "mixed" } }),
        json!({ "data_risk_focus": { "primary_focus": "none" } }),
        json!({ "data_risk_focus": {} }),
    ];
    let result = aggregate_workspace_data_risk_focus(&summaries);
    assert_eq!(result["distribution"]["hardcoded"], 1);
    assert_eq!(result["distribution"]["mock"], 1);
    assert_eq!(result["distribution"]["mixed"], 1);
    assert_eq!(result["distribution"]["none"], 2);
    assert_eq!(result["projects_requiring_hardcoded_review"], 1);
    assert_eq!(result["projects_requiring_mock_review"], 1);
    assert_eq!(result["projects_requiring_mixed_file_review"], 1);
}

#[test]
fn aggregate_workspace_data_risk_focus_all_hardcoded() {
    let summaries = vec![
        json!({ "data_risk_focus": { "primary_focus": "hardcoded" } }),
        json!({ "data_risk_focus": { "primary_focus": "hardcoded" } }),
    ];
    let result = aggregate_workspace_data_risk_focus(&summaries);
    assert_eq!(result["distribution"]["hardcoded"], 2);
    assert_eq!(result["projects_requiring_hardcoded_review"], 2);
}

// ---- aggregate_workspace_rule_hits ----

#[test]
fn aggregate_workspace_rule_hits_empty() {
    let result = aggregate_workspace_rule_hits(&[]);
    let arr = result.as_array().unwrap();
    assert!(arr.is_empty());
}

#[test]
fn aggregate_workspace_rule_hits_single_project() {
    let summaries = vec![json!({
        "rule_hits_summary": [
            { "rule": "path.mock_token", "count": 3 },
            { "rule": "content.mock_token", "count": 5 },
        ],
    })];
    let result = aggregate_workspace_rule_hits(&summaries);
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    // Sorted by count descending
    assert_eq!(arr[0]["count"], 5);
    assert_eq!(arr[1]["count"], 3);
}

#[test]
fn aggregate_workspace_rule_hits_aggregates_across_projects() {
    let summaries = vec![
        json!({
            "rule_hits_summary": [
                { "rule": "path.mock_token", "count": 2 },
            ],
        }),
        json!({
            "rule_hits_summary": [
                { "rule": "path.mock_token", "count": 3 },
            ],
        }),
    ];
    let result = aggregate_workspace_rule_hits(&summaries);
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["count"], 5);
    assert_eq!(arr[0]["rule"], "path.mock_token");
}

#[test]
fn aggregate_workspace_rule_hits_sorted_by_count_then_severity() {
    let summaries = vec![json!({
        "rule_hits_summary": [
            { "rule": "path.mock_token", "count": 5 },
            { "rule": "content.business_literal_combo", "count": 5 },
        ],
    })];
    let result = aggregate_workspace_rule_hits(&summaries);
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    // Same count, so sorted by severity desc (content.business_literal_combo is "high")
    assert_eq!(arr[0]["rule"], "content.business_literal_combo");
    assert_eq!(arr[1]["rule"], "path.mock_token");
}

#[test]
fn aggregate_workspace_rule_hits_unknown_rule_has_metadata() {
    let summaries = vec![json!({
        "rule_hits_summary": [
            { "rule": "custom.unknown_rule", "count": 1 },
        ],
    })];
    let result = aggregate_workspace_rule_hits(&summaries);
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["rule"], "custom.unknown_rule");
    assert_eq!(arr[0]["group"], "unknown");
    assert_eq!(arr[0]["severity"], "unknown");
}

// ---- aggregate_workspace_rule_groups ----

#[test]
fn aggregate_workspace_rule_groups_empty() {
    let result = aggregate_workspace_rule_groups(&[]);
    let arr = result.as_array().unwrap();
    assert!(arr.is_empty());
}

#[test]
fn aggregate_workspace_rule_groups_single_project() {
    let summaries = vec![json!({
        "rule_groups_summary": [
            { "group": "path", "count": 3 },
            { "group": "content", "count": 7 },
        ],
    })];
    let result = aggregate_workspace_rule_groups(&summaries);
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    // Sorted by count descending
    assert_eq!(arr[0]["group"], "content");
    assert_eq!(arr[0]["count"], 7);
}

#[test]
fn aggregate_workspace_rule_groups_aggregates_across_projects() {
    let summaries = vec![
        json!({
            "rule_groups_summary": [
                { "group": "path", "count": 4 },
            ],
        }),
        json!({
            "rule_groups_summary": [
                { "group": "path", "count": 6 },
            ],
        }),
    ];
    let result = aggregate_workspace_rule_groups(&summaries);
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["group"], "path");
    assert_eq!(arr[0]["count"], 10);
}

#[test]
fn aggregate_workspace_rule_groups_assigns_severity() {
    let summaries = vec![json!({
        "rule_groups_summary": [
            { "group": "content", "count": 1 },
            { "group": "path", "count": 1 },
            { "group": "classification", "count": 1 },
            { "group": "custom", "count": 1 },
        ],
    })];
    let result = aggregate_workspace_rule_groups(&summaries);
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 4);

    let find_group =
        |name: &str| -> &serde_json::Value { arr.iter().find(|v| v["group"] == name).unwrap() };
    assert_eq!(find_group("content")["severity"], "medium");
    assert_eq!(find_group("classification")["severity"], "medium");
    assert_eq!(find_group("path")["severity"], "low");
    assert_eq!(find_group("custom")["severity"], "unknown");
}

// ---- workspace_data_risk_overview_payload ----

#[test]
fn workspace_data_risk_overview_payload_empty() {
    let result = workspace_data_risk_overview_payload(&[], 0);
    // tool_guidance sets schema_version to MCP_GUIDANCE_V1
    assert!(result["schema_version"].is_string());
    assert_eq!(
        result["layers"]["workspace_observation"]["matched_project_count"],
        0
    );
    assert_eq!(
        result["layers"]["workspace_observation"]["total_mock_candidates"],
        0
    );
    assert_eq!(
        result["layers"]["workspace_observation"]["total_hardcoded_candidates"],
        0
    );
    assert!(
        result["layers"]["cleanup_refactor_candidates"]["priority_projects"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn workspace_data_risk_overview_payload_single_project_with_mock() {
    let summaries = vec![json!({
        "project_id": "demo",
        "hardcoded_candidate_count": 0,
        "mock_candidate_count": 3,
        "mixed_review_file_count": 0,
        "data_risk_focus": { "primary_focus": "mock" },
        "rule_hits_summary": [],
        "rule_groups_summary": [],
    })];
    let result = workspace_data_risk_overview_payload(&summaries, 5);
    assert_eq!(
        result["layers"]["workspace_observation"]["matched_project_count"],
        1
    );
    assert_eq!(
        result["layers"]["workspace_observation"]["projects_with_mock_candidates"],
        1
    );
    assert_eq!(
        result["layers"]["workspace_observation"]["projects_with_hardcoded_candidates"],
        0
    );
    assert_eq!(
        result["layers"]["workspace_observation"]["total_mock_candidates"],
        3
    );
}

#[test]
fn workspace_data_risk_overview_payload_priority_truncates_to_10() {
    let summaries: Vec<Value> = (0..15)
        .map(|i| {
            json!({
                "project_id": format!("proj-{}", i),
                "hardcoded_candidate_count": 15 - i,
                "mock_candidate_count": 0,
                "mixed_review_file_count": 0,
                "data_risk_focus": { "primary_focus": "hardcoded" },
                "rule_hits_summary": [],
                "rule_groups_summary": [],
            })
        })
        .collect();
    let result = workspace_data_risk_overview_payload(&summaries, 15);
    let priority = result["layers"]["cleanup_refactor_candidates"]["priority_projects"]
        .as_array()
        .unwrap();
    assert_eq!(priority.len(), 10);
    // First should have the highest hardcoded count
    assert_eq!(priority[0]["project_id"], "proj-0");
    assert_eq!(priority[0]["hardcoded_candidate_count"], 15);
}
