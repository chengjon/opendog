#![allow(dead_code)]

use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CandidateFreshness {
    pub(crate) snapshot_stale: bool,
    pub(crate) activity_stale: bool,
}

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

pub(crate) fn candidate_basis_for(
    kind: &str,
    mock_summary: &Value,
    file_path: &str,
) -> Vec<&'static str> {
    let mut basis = match kind {
        "hot_file" => vec!["highest_access_activity", "activity_present"],
        _ => vec!["zero_recorded_access", "snapshot_present"],
    };

    if summary_contains_path(mock_summary, "mock_data_candidates", file_path) {
        basis.push("mock_data_overlap");
    }
    if summary_contains_path(mock_summary, "hardcoded_data_candidates", file_path) {
        basis.push("hardcoded_data_overlap");
    }

    basis
}

pub(crate) fn candidate_risk_hints_for(
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

pub(crate) fn build_review_candidate(
    kind: &str,
    file_path: &str,
    priority: &str,
    reason: &str,
    suggested_commands: Vec<String>,
    mock_summary: &Value,
    freshness: CandidateFreshness,
    repo_risk: &Value,
) -> Value {
    json!({
        "kind": kind,
        "file_path": file_path,
        "reason": reason,
        "suggested_commands": suggested_commands,
        "candidate_basis": candidate_basis_for(kind, mock_summary, file_path),
        "candidate_risk_hints": candidate_risk_hints_for(kind, freshness, repo_risk),
        "candidate_priority": priority,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn low_repo_risk() -> Value {
        json!({
            "risk_level": "low",
            "large_diff": false
        })
    }

    #[test]
    fn build_review_candidate_marks_hot_file_overlap_basis() {
        let summary = json!({
            "mock_data_candidates": [{"file_path": "src/main.rs"}],
            "hardcoded_data_candidates": []
        });

        let candidate = build_review_candidate(
            "hot_file",
            "src/main.rs",
            "primary",
            "hot",
            vec!["cargo test".to_string()],
            &summary,
            CandidateFreshness::default(),
            &low_repo_risk(),
        );

        assert_eq!(
            candidate["candidate_basis"],
            json!([
                "highest_access_activity",
                "activity_present",
                "mock_data_overlap"
            ])
        );
        assert_eq!(candidate["candidate_risk_hints"], json!([]));
        assert_eq!(candidate["candidate_priority"], json!("primary"));
    }

    #[test]
    fn build_review_candidate_marks_unused_snapshot_staleness_and_hardcoded_overlap() {
        let summary = json!({
            "mock_data_candidates": [],
            "hardcoded_data_candidates": [{"file_path": "src/legacy.rs"}]
        });

        let candidate = build_review_candidate(
            "unused_candidate",
            "src/legacy.rs",
            "secondary",
            "unused",
            vec!["git grep <symbol>".to_string()],
            &summary,
            CandidateFreshness {
                snapshot_stale: true,
                activity_stale: false,
            },
            &low_repo_risk(),
        );

        assert_eq!(
            candidate["candidate_basis"],
            json!([
                "zero_recorded_access",
                "snapshot_present",
                "hardcoded_data_overlap"
            ])
        );
        assert_eq!(
            candidate["candidate_risk_hints"],
            json!(["snapshot_evidence_stale"])
        );
        assert_eq!(candidate["candidate_priority"], json!("secondary"));
    }

    #[test]
    fn build_review_candidate_marks_hot_file_activity_staleness() {
        let candidate = build_review_candidate(
            "hot_file",
            "src/main.rs",
            "primary",
            "hot",
            vec![],
            &json!({}),
            CandidateFreshness {
                snapshot_stale: false,
                activity_stale: true,
            },
            &low_repo_risk(),
        );

        assert_eq!(
            candidate["candidate_risk_hints"],
            json!(["activity_evidence_stale"])
        );
    }

    #[test]
    fn build_review_candidate_marks_hot_file_repo_risk_from_large_diff() {
        let candidate = build_review_candidate(
            "hot_file",
            "src/main.rs",
            "primary",
            "hot",
            vec![],
            &json!({}),
            CandidateFreshness::default(),
            &json!({
                "risk_level": "low",
                "large_diff": true
            }),
        );

        assert_eq!(
            candidate["candidate_risk_hints"],
            json!(["repo_risk_elevated"])
        );
    }
}
