use serde_json::{json, Value};

use super::{DataRiskRuleMeta, DATA_RISK_RULES};

pub(super) fn data_risk_rule_meta(rule: &str) -> Option<&'static DataRiskRuleMeta> {
    DATA_RISK_RULES.iter().find(|meta| meta.rule == rule)
}

pub(super) fn data_risk_severity_score(severity: &str) -> usize {
    match severity {
        "critical" => 4,
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

pub(crate) fn normalize_candidate_type(candidate_type: Option<String>) -> Result<String, Value> {
    let candidate_type = candidate_type.unwrap_or_else(|| "all".to_string());
    if matches!(candidate_type.as_str(), "all" | "mock" | "hardcoded") {
        Ok(candidate_type)
    } else {
        Err(json!({
            "error": "candidate_type must be one of: all, mock, hardcoded"
        }))
    }
}

pub(crate) fn normalize_min_review_priority(
    min_review_priority: Option<String>,
) -> Result<String, Value> {
    let min_review_priority = min_review_priority.unwrap_or_else(|| "low".to_string());
    if matches!(min_review_priority.as_str(), "low" | "medium" | "high") {
        Ok(min_review_priority)
    } else {
        Err(json!({
            "error": "min_review_priority must be one of: low, medium, high"
        }))
    }
}

pub(crate) fn review_priority_score(priority: &str) -> i32 {
    match priority {
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

pub(crate) fn path_kind_score(kind: &str) -> i32 {
    match kind {
        "runtime_shared" => 3,
        "unknown" => 2,
        "test_only" => 1,
        "generated_artifact" => 0,
        _ => 0,
    }
}
