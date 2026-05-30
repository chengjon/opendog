use serde_json::{json, Value};

use super::super::rules::data_risk_severity_score;

pub(super) fn enrich_workspace_priority_project(summary: &Value) -> Value {
    let mut enriched = summary.clone();
    enriched["dominant_rule_group"] = workspace_dominant_rule_group(summary);
    enriched["priority_reason"] = json!(workspace_priority_reason(summary));
    enriched
}

pub(super) fn workspace_priority_reason(summary: &Value) -> String {
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

pub(super) fn workspace_dominant_rule_group(summary: &Value) -> Value {
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
