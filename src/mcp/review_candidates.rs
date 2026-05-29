use serde_json::{json, Value};

use crate::core::file_classification::classify_file_path;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CandidateFreshness {
    pub(crate) snapshot_stale: bool,
    pub(crate) activity_stale: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy)]
pub(crate) struct ReviewCandidateContext<'a> {
    pub(crate) mock_summary: &'a Value,
    pub(crate) freshness: CandidateFreshness,
    pub(crate) repo_risk: &'a Value,
}

#[cfg_attr(not(test), allow(dead_code))]
fn summary_contains_path(summary: &Value, key: &str, file_path: &str) -> bool {
    summary[key]
        .as_array()
        .map(|items| {
            items.iter().any(|item| {
                item["file_path"]
                    .as_str()
                    .map(|path| path == file_path)
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

#[cfg_attr(not(test), allow(dead_code))]
fn candidate_basis_for(kind: &str, mock_summary: &Value, file_path: &str) -> Vec<&'static str> {
    let mut basis = match kind {
        "hot_file" => vec!["highest_access_activity", "activity_present"],
        "unused_candidate" => vec!["zero_recorded_access", "snapshot_present"],
        _ => Vec::new(),
    };

    if basis.is_empty() {
        return basis;
    }

    if summary_contains_path(mock_summary, "mock_data_candidates", file_path) {
        basis.push("mock_data_overlap");
    }
    if summary_contains_path(mock_summary, "hardcoded_data_candidates", file_path) {
        basis.push("hardcoded_data_overlap");
    }

    basis
}

#[cfg_attr(not(test), allow(dead_code))]
fn candidate_risk_hints_for(
    kind: &str,
    freshness: CandidateFreshness,
    repo_risk: &Value,
) -> Vec<&'static str> {
    let mut risk_hints = Vec::new();
    if kind == "hot_file" && freshness.activity_stale {
        risk_hints.push("activity_evidence_stale");
    }
    if kind == "unused_candidate" && freshness.snapshot_stale {
        risk_hints.push("snapshot_evidence_stale");
    }
    if kind == "hot_file"
        && (repo_risk["risk_level"].as_str().unwrap_or("low") != "low"
            || repo_risk["large_diff"].as_bool().unwrap_or(false))
    {
        risk_hints.push("repo_risk_elevated");
    }

    risk_hints
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_review_candidate(
    kind: &str,
    file_path: &str,
    priority: &str,
    reason: &str,
    suggested_commands: Vec<String>,
    context: ReviewCandidateContext<'_>,
) -> Value {
    json!({
        "kind": kind,
        "file_path": file_path,
        "path_classification": classify_file_path(file_path).as_str(),
        "reason": reason,
        "suggested_commands": suggested_commands,
        "candidate_basis": candidate_basis_for(kind, context.mock_summary, file_path),
        "candidate_risk_hints": candidate_risk_hints_for(kind, context.freshness, context.repo_risk),
        "candidate_priority": priority,
    })
}

#[cfg(test)]
mod tests;
