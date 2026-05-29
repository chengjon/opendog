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
        ReviewCandidateContext {
            mock_summary: &summary,
            freshness: CandidateFreshness::default(),
            repo_risk: &low_repo_risk(),
        },
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
        ReviewCandidateContext {
            mock_summary: &summary,
            freshness: CandidateFreshness {
                snapshot_stale: true,
                activity_stale: false,
            },
            repo_risk: &low_repo_risk(),
        },
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
        ReviewCandidateContext {
            mock_summary: &json!({}),
            freshness: CandidateFreshness {
                snapshot_stale: false,
                activity_stale: true,
            },
            repo_risk: &low_repo_risk(),
        },
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
        ReviewCandidateContext {
            mock_summary: &json!({}),
            freshness: CandidateFreshness::default(),
            repo_risk: &json!({
                "risk_level": "low",
                "large_diff": true
            }),
        },
    );

    assert_eq!(
        candidate["candidate_risk_hints"],
        json!(["repo_risk_elevated"])
    );
}

#[test]
fn build_review_candidate_marks_hot_file_repo_risk_from_high_risk_level() {
    let candidate = build_review_candidate(
        "hot_file",
        "src/main.rs",
        "primary",
        "hot",
        vec![],
        ReviewCandidateContext {
            mock_summary: &json!({}),
            freshness: CandidateFreshness::default(),
            repo_risk: &json!({
                "risk_level": "high",
                "large_diff": false
            }),
        },
    );

    assert_eq!(
        candidate["candidate_risk_hints"],
        json!(["repo_risk_elevated"])
    );
}

#[test]
fn build_review_candidate_keeps_unknown_kind_empty() {
    let candidate = build_review_candidate(
        "unknown_kind",
        "src/main.rs",
        "secondary",
        "hot",
        vec![],
        ReviewCandidateContext {
            mock_summary: &json!({
                "mock_data_candidates": [{"file_path": "src/main.rs"}],
                "hardcoded_data_candidates": [{"file_path": "src/main.rs"}]
            }),
            freshness: CandidateFreshness {
                snapshot_stale: true,
                activity_stale: true,
            },
            repo_risk: &json!({
                "risk_level": "high",
                "large_diff": true
            }),
        },
    );

    assert_eq!(candidate["candidate_basis"], json!([]));
    assert_eq!(candidate["candidate_risk_hints"], json!([]));
}
