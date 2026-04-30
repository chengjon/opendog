use super::*;

#[test]
fn recommend_project_action_prioritizes_failing_verification() {
    let repo_risk = json!({
        "operation_states": [],
        "risk_level": "low",
        "is_dirty": false
    });
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            id: "demo".to_string(),
            status: "monitoring".to_string(),
            root_path: std::path::PathBuf::from("/tmp/demo"),
            total_files: 20,
            accessed_files: 8,
            unused_files: 4,
            latest_snapshot_captured_at: Some(fresh_ts()),
            latest_activity_at: Some(fresh_ts()),
            latest_verification_at: Some(fresh_ts()),
        },
        &repo_risk,
        &[VerificationRun {
            id: 1,
            kind: "test".to_string(),
            status: "failed".to_string(),
            command: "cargo test".to_string(),
            exit_code: Some(101),
            summary: Some("fail".to_string()),
            source: "cli".to_string(),
            started_at: None,
            finished_at: "1".to_string(),
        }],
    );

    assert_eq!(
        recommendation["recommended_next_action"],
        json!("review_failing_verification")
    );
    assert_eq!(recommendation["confidence"], json!("high"));
    assert!(recommendation["recommended_flow"]
        .as_array()
        .unwrap()
        .iter()
        .any(|step| step.as_str().unwrap().contains("verification")));
}

#[test]
fn recommend_project_action_prefers_verification_when_missing() {
    let repo_risk = json!({
        "operation_states": [],
        "risk_level": "low",
        "is_dirty": false
    });
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            id: "demo".to_string(),
            status: "monitoring".to_string(),
            root_path: std::path::PathBuf::from("/tmp/demo"),
            total_files: 20,
            accessed_files: 8,
            unused_files: 4,
            latest_snapshot_captured_at: Some(fresh_ts()),
            latest_activity_at: Some(fresh_ts()),
            latest_verification_at: None,
        },
        &repo_risk,
        &[],
    );

    assert_eq!(
        recommendation["recommended_next_action"],
        json!("run_verification_before_high_risk_changes")
    );
    assert_eq!(
        recommendation["strategy_mode"],
        json!("verify_before_modify")
    );
    assert_eq!(
        recommendation["strategy_profile"]["preferred_primary_tool"],
        json!("shell")
    );
}

#[test]
fn recommend_project_action_marks_advisory_only_gaps_as_caution() {
    let repo_risk = json!({
        "operation_states": [],
        "risk_level": "low",
        "is_dirty": false
    });
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            id: "demo".to_string(),
            status: "monitoring".to_string(),
            root_path: std::path::PathBuf::from("/tmp/demo"),
            total_files: 20,
            accessed_files: 8,
            unused_files: 0,
            latest_snapshot_captured_at: Some(fresh_ts()),
            latest_activity_at: Some(fresh_ts()),
            latest_verification_at: Some(fresh_ts()),
        },
        &repo_risk,
        &[
            VerificationRun {
                id: 1,
                kind: "test".to_string(),
                status: "passed".to_string(),
                command: "cargo test".to_string(),
                exit_code: Some(0),
                summary: Some("ok".to_string()),
                source: "cli".to_string(),
                started_at: None,
                finished_at: fresh_ts(),
            },
            VerificationRun {
                id: 2,
                kind: "build".to_string(),
                status: "passed".to_string(),
                command: "cargo check".to_string(),
                exit_code: Some(0),
                summary: Some("ok".to_string()),
                source: "cli".to_string(),
                started_at: None,
                finished_at: fresh_ts(),
            },
        ],
    );

    assert_eq!(
        recommendation["recommended_next_action"],
        json!("inspect_hot_files")
    );
    assert_eq!(
        recommendation["verification_gate_levels"]["cleanup"],
        json!("caution")
    );
    assert_eq!(
        recommendation["verification_gate_levels"]["refactor"],
        json!("caution")
    );
    assert_eq!(recommendation["confidence"], json!("medium"));
    assert!(recommendation["reason"]
        .as_str()
        .unwrap()
        .contains("cautious"));
}
