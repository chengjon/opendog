use super::*;

#[test]
fn repo_risk_findings_expose_priority_and_confidence() {
    let findings = repo_risk_findings(&RepoRiskSnapshot {
        status: "available",
        branch: Some("main".to_string()),
        is_dirty: true,
        changed_file_count: 28,
        staged_count: 10,
        unstaged_count: 8,
        untracked_count: 1,
        conflicted_count: 0,
        operation_states: vec!["rebase".to_string()],
        top_changed_directories: vec![("src".to_string(), 12)],
        large_diff: true,
        lockfile_anomalies: vec![json!({
            "kind": "manifest_without_lockfile_change",
            "directory": ".",
            "manifest": "Cargo.toml",
            "expected_lockfile": "Cargo.lock"
        })],
        evidence: vec!["Changed files observed via git status: 28.".to_string()],
        risk_reasons: vec![],
        risk_level: "high",
    });

    assert_eq!(findings[0]["kind"], "repository_operation_in_progress");
    assert_eq!(findings[0]["severity"], "high");
    assert_eq!(findings[0]["priority"], "immediate");
    assert_eq!(findings[0]["confidence"], "high");
    assert!(findings
        .iter()
        .any(|item| item["kind"] == "large_working_diff"));
    assert!(findings
        .iter()
        .any(|item| item["kind"] == "dependency_lockfile_anomaly"));
}

#[test]
fn repo_status_risk_layer_marks_non_git_roots_explicitly() {
    let dir = TempDir::new().unwrap();
    let value = repo_status_risk_layer(dir.path());

    assert_eq!(value["status"], "not_git_repository");
}
