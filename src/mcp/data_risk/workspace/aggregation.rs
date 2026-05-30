use serde_json::{json, Value};
use std::collections::BTreeMap;

use super::super::rules::{data_risk_rule_meta, data_risk_severity_score};

pub(super) fn aggregate_workspace_data_risk_focus(project_summaries: &[Value]) -> Value {
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

pub(super) fn aggregate_workspace_rule_hits(project_summaries: &[Value]) -> Value {
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

pub(super) fn aggregate_workspace_rule_groups(project_summaries: &[Value]) -> Value {
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
