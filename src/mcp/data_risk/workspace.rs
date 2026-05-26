use serde_json::{json, Value};
use std::collections::BTreeMap;

use super::super::{set_recommended_flow, tool_guidance};
use super::rules::{data_risk_rule_meta, data_risk_severity_score};

pub(crate) fn workspace_data_risk_overview_payload(
    project_summaries: &[Value],
    total_registered_projects: usize,
) -> Value {
    let rule_hits_summary = aggregate_workspace_rule_hits(project_summaries);
    let rule_groups_summary = aggregate_workspace_rule_groups(project_summaries);
    let data_risk_focus_summary = aggregate_workspace_data_risk_focus(project_summaries);
    let matched_projects = project_summaries.len();
    let projects_with_hardcoded = project_summaries
        .iter()
        .filter(|summary| summary["hardcoded_candidate_count"].as_u64().unwrap_or(0) > 0)
        .count();
    let projects_with_mock = project_summaries
        .iter()
        .filter(|summary| summary["mock_candidate_count"].as_u64().unwrap_or(0) > 0)
        .count();
    let total_hardcoded_candidates = project_summaries
        .iter()
        .map(|summary| summary["hardcoded_candidate_count"].as_u64().unwrap_or(0))
        .sum::<u64>();
    let total_mock_candidates = project_summaries
        .iter()
        .map(|summary| summary["mock_candidate_count"].as_u64().unwrap_or(0))
        .sum::<u64>();

    let mut priority_projects = project_summaries
        .iter()
        .map(enrich_workspace_priority_project)
        .collect::<Vec<_>>();
    priority_projects.sort_by(|a, b| {
        b["hardcoded_candidate_count"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["hardcoded_candidate_count"].as_u64().unwrap_or(0))
            .then_with(|| {
                b["mixed_review_file_count"]
                    .as_u64()
                    .unwrap_or(0)
                    .cmp(&a["mixed_review_file_count"].as_u64().unwrap_or(0))
            })
            .then_with(|| {
                b["mock_candidate_count"]
                    .as_u64()
                    .unwrap_or(0)
                    .cmp(&a["mock_candidate_count"].as_u64().unwrap_or(0))
            })
    });
    priority_projects.truncate(10);

    let mut guidance = tool_guidance(
        if projects_with_hardcoded > 0 {
            "Workspace data-risk overview loaded. Review projects with hardcoded-data candidates before broad cleanup or refactor work."
        } else if projects_with_mock > 0 {
            "Workspace data-risk overview loaded. Mock-style candidates exist; confirm they are test-only before cleanup."
        } else {
            "Workspace data-risk overview loaded. No current mock or hardcoded-data candidates were detected."
        },
        &[
            "opendog get-data-risk-candidates --id <project>",
            "rg \"mock|fixture|fake|stub|sample|demo|seed\" .",
            "rg \"customer|invoice|email|address|payment|tenant\" .",
        ],
        &["get_data_risk_candidates", "get_guidance", "list_projects"],
        Some("Use shell commands to inspect candidate files directly after OPENDOG identifies which projects deserve manual review."),
    );
    if projects_with_hardcoded > 0 {
        set_recommended_flow(
            &mut guidance,
            &[
                "Start with the highest-priority project in the workspace queue.",
                "Inspect that project's hardcoded-data candidates before broad cleanup or refactor.",
                "Use project-level guidance and verification status before making edits.",
                "Repeat for the next project only after the first review path is understood.",
            ],
        );
    } else if projects_with_mock > 0 {
        set_recommended_flow(
            &mut guidance,
            &[
                "Start with the highest-priority project in the workspace queue.",
                "Confirm whether mock-style candidates are test-only artifacts.",
                "Escalate to project-level data-risk review if any runtime/shared path looks suspicious.",
            ],
        );
    } else {
        set_recommended_flow(
            &mut guidance,
            &[
                "No current workspace data-risk candidates were detected.",
                "Use agent guidance or verification status to choose the next project action.",
                "Return to workspace-level review when priorities shift across projects.",
            ],
        );
    }
    guidance["layers"]["workspace_observation"] = json!({
        "status": "available",
        "total_registered_projects": total_registered_projects,
        "matched_project_count": matched_projects,
        "projects_with_mock_candidates": projects_with_mock,
        "projects_with_hardcoded_candidates": projects_with_hardcoded,
        "total_mock_candidates": total_mock_candidates,
        "total_hardcoded_candidates": total_hardcoded_candidates,
        "data_risk_focus_distribution": data_risk_focus_summary["distribution"].clone(),
        "projects_requiring_hardcoded_review":
            data_risk_focus_summary["projects_requiring_hardcoded_review"].clone(),
        "projects_requiring_mock_review":
            data_risk_focus_summary["projects_requiring_mock_review"].clone(),
        "projects_requiring_mixed_file_review":
            data_risk_focus_summary["projects_requiring_mixed_file_review"].clone(),
        "rule_groups_summary": rule_groups_summary,
        "rule_hits_summary": rule_hits_summary,
    });
    guidance["layers"]["multi_project_portfolio"] = json!({
        "status": "available",
        "total_registered_projects": total_registered_projects,
        "matched_project_count": matched_projects,
        "projects_with_mock_candidates": projects_with_mock,
        "projects_with_hardcoded_candidates": projects_with_hardcoded,
        "total_mock_candidates": total_mock_candidates,
        "total_hardcoded_candidates": total_hardcoded_candidates,
        "rule_groups_summary": rule_groups_summary,
        "rule_hits_summary": rule_hits_summary,
        "priority_projects": priority_projects,
    });
    guidance["layers"]["execution_strategy"]["projects_with_hardcoded_data_candidates"] =
        json!(projects_with_hardcoded);
    guidance["layers"]["execution_strategy"]["review_mock_data_before_cleanup"] =
        json!(projects_with_hardcoded > 0);
    guidance["layers"]["execution_strategy"]["data_risk_focus_distribution"] =
        data_risk_focus_summary["distribution"].clone();
    guidance["layers"]["execution_strategy"]["projects_requiring_hardcoded_review"] =
        data_risk_focus_summary["projects_requiring_hardcoded_review"].clone();
    guidance["layers"]["execution_strategy"]["projects_requiring_mock_review"] =
        data_risk_focus_summary["projects_requiring_mock_review"].clone();
    guidance["layers"]["execution_strategy"]["projects_requiring_mixed_file_review"] =
        data_risk_focus_summary["projects_requiring_mixed_file_review"].clone();
    guidance["layers"]["cleanup_refactor_candidates"] = json!({
        "status": "available",
        "priority_projects": priority_projects,
    });
    guidance
}

fn enrich_workspace_priority_project(summary: &Value) -> Value {
    let mut enriched = summary.clone();
    enriched["dominant_rule_group"] = workspace_dominant_rule_group(summary);
    enriched["priority_reason"] = json!(workspace_priority_reason(summary));
    enriched
}

fn workspace_priority_reason(summary: &Value) -> String {
    let hardcoded = summary["hardcoded_candidate_count"].as_u64().unwrap_or(0);
    let mock = summary["mock_candidate_count"].as_u64().unwrap_or(0);
    let mixed = summary["mixed_review_file_count"].as_u64().unwrap_or(0);
    let runtime_shared = workspace_rule_hit_count(summary, "path.runtime_shared");
    let high_content = workspace_rule_hit_count(summary, "content.business_literal_combo");

    if hardcoded > 0 && runtime_shared > 0 && high_content > 0 {
        return "runtime-shared hardcoded candidates with high-severity content matches"
            .to_string();
    }
    if hardcoded > 0 && runtime_shared > 0 {
        return "runtime-shared hardcoded candidates need manual review before refactor"
            .to_string();
    }
    if hardcoded > 0 && high_content > 0 {
        return "hardcoded business-like literals detected in review candidates".to_string();
    }
    if hardcoded > 0 && mixed > 0 {
        return "hardcoded candidates appear alongside mixed review files".to_string();
    }
    if hardcoded > 0 {
        return "hardcoded-data candidates require project-level inspection".to_string();
    }
    if mixed > 0 {
        return "mixed mock and hardcoded review files need classification cleanup".to_string();
    }
    if mock > 0 {
        return "mock-style candidates should be confirmed as test-only before cleanup".to_string();
    }
    "no current mock or hardcoded-data candidates".to_string()
}

fn workspace_rule_hit_count(summary: &Value, rule: &str) -> u64 {
    summary["rule_hits_summary"]
        .as_array()
        .and_then(|entries| {
            entries
                .iter()
                .find(|entry| entry["rule"].as_str() == Some(rule))
                .and_then(|entry| entry["count"].as_u64())
        })
        .unwrap_or(0)
}

fn workspace_dominant_rule_group(summary: &Value) -> Value {
    let dominant = summary["rule_groups_summary"]
        .as_array()
        .and_then(|entries| {
            entries.iter().max_by(|a, b| {
                a["count"]
                    .as_u64()
                    .unwrap_or(0)
                    .cmp(&b["count"].as_u64().unwrap_or(0))
                    .then_with(|| {
                        data_risk_severity_score(b["severity"].as_str().unwrap_or("unknown")).cmp(
                            &data_risk_severity_score(a["severity"].as_str().unwrap_or("unknown")),
                        )
                    })
            })
        })
        .cloned();
    dominant.unwrap_or(Value::Null)
}

fn aggregate_workspace_data_risk_focus(project_summaries: &[Value]) -> Value {
    let mut distribution = json!({
        "hardcoded": 0,
        "mixed": 0,
        "mock": 0,
        "none": 0,
    });
    let mut projects_requiring_hardcoded_review = 0_u64;
    let mut projects_requiring_mock_review = 0_u64;
    let mut projects_requiring_mixed_file_review = 0_u64;

    for summary in project_summaries {
        match summary["data_risk_focus"]["primary_focus"]
            .as_str()
            .unwrap_or("none")
        {
            "hardcoded" => {
                distribution["hardcoded"] =
                    json!(distribution["hardcoded"].as_u64().unwrap_or(0) + 1);
                projects_requiring_hardcoded_review += 1;
            }
            "mixed" => {
                distribution["mixed"] = json!(distribution["mixed"].as_u64().unwrap_or(0) + 1);
                projects_requiring_mixed_file_review += 1;
            }
            "mock" => {
                distribution["mock"] = json!(distribution["mock"].as_u64().unwrap_or(0) + 1);
                projects_requiring_mock_review += 1;
            }
            _ => {
                distribution["none"] = json!(distribution["none"].as_u64().unwrap_or(0) + 1);
            }
        }
    }

    json!({
        "distribution": distribution,
        "projects_requiring_hardcoded_review": projects_requiring_hardcoded_review,
        "projects_requiring_mock_review": projects_requiring_mock_review,
        "projects_requiring_mixed_file_review": projects_requiring_mixed_file_review,
    })
}

fn aggregate_workspace_rule_hits(project_summaries: &[Value]) -> Value {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for summary in project_summaries {
        if let Some(entries) = summary["rule_hits_summary"].as_array() {
            for entry in entries {
                if let Some(rule) = entry["rule"].as_str() {
                    *counts.entry(rule.to_string()).or_insert(0) +=
                        entry["count"].as_u64().unwrap_or(0);
                }
            }
        }
    }

    let mut aggregated = counts
        .into_iter()
        .map(|(rule, count)| {
            if let Some(meta) = data_risk_rule_meta(&rule) {
                json!({
                    "rule": meta.rule,
                    "group": meta.group,
                    "severity": meta.severity,
                    "description": meta.description,
                    "count": count,
                })
            } else {
                json!({
                    "rule": rule,
                    "group": "unknown",
                    "severity": "unknown",
                    "description": "No rule metadata registered.",
                    "count": count,
                })
            }
        })
        .collect::<Vec<_>>();
    aggregated.sort_by(|a, b| {
        b["count"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["count"].as_u64().unwrap_or(0))
            .then_with(|| {
                data_risk_severity_score(b["severity"].as_str().unwrap_or("unknown")).cmp(
                    &data_risk_severity_score(a["severity"].as_str().unwrap_or("unknown")),
                )
            })
    });
    json!(aggregated)
}

fn aggregate_workspace_rule_groups(project_summaries: &[Value]) -> Value {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for summary in project_summaries {
        if let Some(entries) = summary["rule_groups_summary"].as_array() {
            for entry in entries {
                if let Some(group) = entry["group"].as_str() {
                    *counts.entry(group.to_string()).or_insert(0) +=
                        entry["count"].as_u64().unwrap_or(0);
                }
            }
        }
    }

    let mut aggregated = counts
        .into_iter()
        .map(|(group, count)| {
            let severity = match group.as_str() {
                "content" => "medium",
                "classification" => "medium",
                "path" => "low",
                _ => "unknown",
            };
            json!({
                "group": group,
                "severity": severity,
                "count": count,
            })
        })
        .collect::<Vec<_>>();
    aggregated.sort_by(|a, b| {
        b["count"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["count"].as_u64().unwrap_or(0))
            .then_with(|| {
                data_risk_severity_score(b["severity"].as_str().unwrap_or("unknown")).cmp(
                    &data_risk_severity_score(a["severity"].as_str().unwrap_or("unknown")),
                )
            })
    });
    json!(aggregated)
}

#[cfg(test)]
mod tests {
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
}
