use super::*;

// ---- risk_severity_priority ----

#[test]
fn severity_priority_high() {
    assert_eq!(risk_severity_priority("high"), 3);
}

#[test]
fn severity_priority_medium() {
    assert_eq!(risk_severity_priority("medium"), 2);
}

#[test]
fn severity_priority_low() {
    assert_eq!(risk_severity_priority("low"), 1);
}

#[test]
fn severity_priority_unknown() {
    assert_eq!(risk_severity_priority("unknown"), 0);
    assert_eq!(risk_severity_priority(""), 0);
    assert_eq!(risk_severity_priority("critical"), 0);
}

// ---- risk_priority_rank ----

#[test]
fn priority_rank_immediate() {
    assert_eq!(risk_priority_rank("immediate"), 4);
}

#[test]
fn priority_rank_high() {
    assert_eq!(risk_priority_rank("high"), 3);
}

#[test]
fn priority_rank_medium() {
    assert_eq!(risk_priority_rank("medium"), 2);
}

#[test]
fn priority_rank_low() {
    assert_eq!(risk_priority_rank("low"), 1);
}

#[test]
fn priority_rank_unknown() {
    assert_eq!(risk_priority_rank("anything"), 0);
    assert_eq!(risk_priority_rank(""), 0);
}

// ---- repo_risk_findings ----

#[derive(Default)]
struct SnapshotSpec {
    changed_file_count: usize,
    conflicted_count: usize,
    staged_count: usize,
    unstaged_count: usize,
    untracked_count: usize,
    large_diff: bool,
    operation_states: Vec<&'static str>,
    lockfile_anomalies: Vec<Value>,
}

fn make_snapshot(spec: SnapshotSpec) -> RepoRiskSnapshot {
    RepoRiskSnapshot {
        status: "available",
        branch: Some("test-branch".to_string()),
        is_dirty: spec.changed_file_count > 0,
        changed_file_count: spec.changed_file_count,
        staged_count: spec.staged_count,
        unstaged_count: spec.unstaged_count,
        untracked_count: spec.untracked_count,
        conflicted_count: spec.conflicted_count,
        operation_states: spec
            .operation_states
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        top_changed_directories: vec![],
        large_diff: spec.large_diff,
        lockfile_anomalies: spec.lockfile_anomalies,
        evidence: vec![format!("Changed files: {}.", spec.changed_file_count)],
        risk_reasons: vec![],
        risk_level: "low",
    }
}

#[test]
fn findings_empty_clean_repo() {
    let snap = make_snapshot(SnapshotSpec::default());
    let findings = repo_risk_findings(&snap);
    assert!(findings.is_empty());
}

#[test]
fn findings_conflicted_paths() {
    let snap = make_snapshot(SnapshotSpec {
        changed_file_count: 3,
        conflicted_count: 2,
        staged_count: 1,
        unstaged_count: 2,
        ..SnapshotSpec::default()
    });
    let findings = repo_risk_findings(&snap);
    let conflicted: Vec<&Value> = findings
        .iter()
        .filter(|f| f["kind"] == "conflicted_paths")
        .collect();
    assert_eq!(conflicted.len(), 1);
    assert_eq!(conflicted[0]["severity"], "high");
    assert_eq!(conflicted[0]["priority"], "immediate");
}

#[test]
fn findings_operation_in_progress() {
    let snap = make_snapshot(SnapshotSpec {
        operation_states: vec!["merge"],
        ..SnapshotSpec::default()
    });
    let findings = repo_risk_findings(&snap);
    let ops: Vec<&Value> = findings
        .iter()
        .filter(|f| f["kind"] == "repository_operation_in_progress")
        .collect();
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0]["severity"], "high");
    assert_eq!(ops[0]["priority"], "immediate");
}

#[test]
fn findings_large_diff() {
    let snap = make_snapshot(SnapshotSpec {
        changed_file_count: 30,
        staged_count: 10,
        unstaged_count: 20,
        untracked_count: 5,
        large_diff: true,
        ..SnapshotSpec::default()
    });
    let findings = repo_risk_findings(&snap);
    let large: Vec<&Value> = findings
        .iter()
        .filter(|f| f["kind"] == "large_working_diff")
        .collect();
    assert_eq!(large.len(), 1);
    assert_eq!(large[0]["severity"], "medium");
    assert_eq!(large[0]["priority"], "high");
}

#[test]
fn findings_lockfile_anomaly() {
    let anomaly = json!({
        "kind": "manifest_without_lockfile_change",
        "directory": ".",
        "manifest": "package.json",
        "expected_lockfile": "package-lock.json",
    });
    let snap = make_snapshot(SnapshotSpec {
        changed_file_count: 2,
        staged_count: 1,
        unstaged_count: 1,
        lockfile_anomalies: vec![anomaly],
        ..SnapshotSpec::default()
    });
    let findings = repo_risk_findings(&snap);
    let lockfile: Vec<&Value> = findings
        .iter()
        .filter(|f| f["kind"] == "dependency_lockfile_anomaly")
        .collect();
    assert_eq!(lockfile.len(), 1);
    assert_eq!(lockfile[0]["severity"], "medium");
}

#[test]
fn findings_local_changes_low_severity() {
    let snap = make_snapshot(SnapshotSpec {
        changed_file_count: 5,
        staged_count: 2,
        unstaged_count: 3,
        untracked_count: 1,
        ..SnapshotSpec::default()
    });
    let findings = repo_risk_findings(&snap);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["kind"], "local_working_tree_changes");
    assert_eq!(findings[0]["severity"], "low");
    assert_eq!(findings[0]["priority"], "medium");
}

#[test]
fn findings_sorted_by_priority_then_severity() {
    let anomaly = json!({"kind": "manifest_without_lockfile_change", "directory": "."});
    let snap = make_snapshot(SnapshotSpec {
        changed_file_count: 30,
        conflicted_count: 2,
        staged_count: 10,
        unstaged_count: 20,
        untracked_count: 5,
        large_diff: true,
        operation_states: vec!["rebase"],
        lockfile_anomalies: vec![anomaly],
    });
    let findings = repo_risk_findings(&snap);

    // conflicted (priority=immediate, severity=high) should come before
    // operation (priority=immediate, severity=high)
    // large_diff (priority=high, severity=medium)
    // lockfile anomaly (priority=high, severity=medium)
    assert!(findings.len() >= 3);

    // First two should be priority "immediate"
    assert_eq!(findings[0]["priority"], "immediate");
    assert_eq!(findings[1]["priority"], "immediate");

    // Among immediate, "conflicted_paths" < "repository_operation_in_progress" alphabetically
    assert_eq!(findings[0]["kind"], "conflicted_paths");
    assert_eq!(findings[1]["kind"], "repository_operation_in_progress");

    // Next should be priority "high"
    assert_eq!(findings[2]["priority"], "high");
}

// ---- repo_risk_finding_counts ----

#[test]
fn finding_counts_mixed_severities() {
    let findings = vec![
        json!({"severity": "high", "kind": "a"}),
        json!({"severity": "high", "kind": "b"}),
        json!({"severity": "medium", "kind": "c"}),
        json!({"severity": "low", "kind": "d"}),
    ];
    let counts = repo_risk_finding_counts(&findings);
    assert_eq!(counts["total"], 4);
    assert_eq!(counts["high"], 2);
    assert_eq!(counts["medium"], 1);
    assert_eq!(counts["low"], 1);
}

#[test]
fn finding_counts_empty() {
    let findings: Vec<Value> = vec![];
    let counts = repo_risk_finding_counts(&findings);
    assert_eq!(counts["total"], 0);
    assert_eq!(counts["high"], 0);
    assert_eq!(counts["medium"], 0);
    assert_eq!(counts["low"], 0);
}

#[test]
fn finding_counts_all_same_severity() {
    let findings = vec![
        json!({"severity": "medium", "kind": "a"}),
        json!({"severity": "medium", "kind": "b"}),
    ];
    let counts = repo_risk_finding_counts(&findings);
    assert_eq!(counts["total"], 2);
    assert_eq!(counts["high"], 0);
    assert_eq!(counts["medium"], 2);
    assert_eq!(counts["low"], 0);
}
