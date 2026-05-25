use serde_json::{json, Value};
use std::path::Path;

use super::collection::{collect_repo_risk_snapshot, git_output};
use super::RepoRiskSnapshot;

fn risk_severity_priority(score: &str) -> i32 {
    match score {
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

fn risk_priority_rank(priority: &str) -> i32 {
    match priority {
        "immediate" => 4,
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

fn confidence_priority(score: &str) -> i32 {
    match score {
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

pub(in crate::mcp) fn repo_risk_findings(snapshot: &RepoRiskSnapshot) -> Vec<Value> {
    let mut findings = Vec::new();

    if snapshot.conflicted_count > 0 {
        findings.push(json!({
            "kind": "conflicted_paths",
            "severity": "high",
            "priority": "immediate",
            "confidence": "high",
            "summary": format!("{} conflicted paths detected in the working tree.", snapshot.conflicted_count),
            "evidence": [format!("git status reported {} conflicted paths.", snapshot.conflicted_count)],
            "source": "git_status",
        }));
    }

    if !snapshot.operation_states.is_empty() {
        findings.push(json!({
            "kind": "repository_operation_in_progress",
            "severity": "high",
            "priority": "immediate",
            "confidence": "high",
            "summary": format!(
                "Repository is mid-operation: {}.",
                snapshot.operation_states.join(", ")
            ),
            "evidence": [format!(
                "Git metadata indicates an in-progress operation: {}.",
                snapshot.operation_states.join(", ")
            )],
            "source": "git_metadata",
        }));
    }

    if snapshot.large_diff {
        findings.push(json!({
            "kind": "large_working_diff",
            "severity": "medium",
            "priority": "high",
            "confidence": "high",
            "summary": format!(
                "Large working diff detected ({} changed files).",
                snapshot.changed_file_count
            ),
            "evidence": [format!(
                "git status reported {} changed files.",
                snapshot.changed_file_count
            )],
            "source": "git_status",
        }));
    }

    for anomaly in &snapshot.lockfile_anomalies {
        let kind = anomaly["kind"].as_str().unwrap_or("lockfile_anomaly");
        let directory = anomaly["directory"].as_str().unwrap_or(".");
        let summary = match kind {
            "manifest_without_lockfile_change" => format!(
                "Manifest changed without corresponding lockfile change in {}.",
                directory
            ),
            "lockfile_without_manifest_change" => format!(
                "Lockfile changed without corresponding manifest change in {}.",
                directory
            ),
            _ => format!("Dependency lockfile anomaly detected in {}.", directory),
        };
        findings.push(json!({
            "kind": "dependency_lockfile_anomaly",
            "severity": "medium",
            "priority": "high",
            "confidence": "high",
            "summary": summary,
            "evidence": [format!("git status flagged dependency file drift in {}.", directory)],
            "source": "git_status",
            "details": anomaly.clone(),
        }));
    }

    if snapshot.changed_file_count > 0 && findings.is_empty() {
        findings.push(json!({
            "kind": "local_working_tree_changes",
            "severity": "low",
            "priority": "medium",
            "confidence": "high",
            "summary": format!(
                "Repository has local changes ({} changed files) without a higher-risk signal.",
                snapshot.changed_file_count
            ),
            "evidence": [format!(
                "git status reported {} changed files.",
                snapshot.changed_file_count
            )],
            "source": "git_status",
        }));
    }

    findings.sort_by(|a, b| {
        let a_priority = risk_priority_rank(a["priority"].as_str().unwrap_or(""));
        let b_priority = risk_priority_rank(b["priority"].as_str().unwrap_or(""));
        let a_severity = risk_severity_priority(a["severity"].as_str().unwrap_or(""));
        let b_severity = risk_severity_priority(b["severity"].as_str().unwrap_or(""));
        b_priority
            .cmp(&a_priority)
            .then_with(|| b_severity.cmp(&a_severity))
            .then_with(|| {
                confidence_priority(b["confidence"].as_str().unwrap_or(""))
                    .cmp(&confidence_priority(a["confidence"].as_str().unwrap_or("")))
            })
            .then_with(|| {
                a["kind"]
                    .as_str()
                    .unwrap_or("")
                    .cmp(b["kind"].as_str().unwrap_or(""))
            })
    });

    findings
}

fn repo_risk_finding_counts(findings: &[Value]) -> Value {
    let high = findings
        .iter()
        .filter(|item| item["severity"] == "high")
        .count();
    let medium = findings
        .iter()
        .filter(|item| item["severity"] == "medium")
        .count();
    let low = findings
        .iter()
        .filter(|item| item["severity"] == "low")
        .count();

    json!({
        "total": findings.len(),
        "high": high,
        "medium": medium,
        "low": low,
    })
}

pub(in crate::mcp) fn repo_status_risk_layer(root: &Path) -> Value {
    let git_check = git_output(root, &["rev-parse", "--show-toplevel"]);
    match git_check {
        Ok(output) if output.status.success() => {
            if let Some(snapshot) = collect_repo_risk_snapshot(root) {
                let findings = repo_risk_findings(&snapshot);
                let finding_counts = repo_risk_finding_counts(&findings);
                let highest_priority_finding = findings.first().cloned().unwrap_or(Value::Null);
                json!({
                    "status": snapshot.status,
                    "risk_level": snapshot.risk_level,
                    "branch": snapshot.branch,
                    "is_dirty": snapshot.is_dirty,
                    "changed_file_count": snapshot.changed_file_count,
                    "staged_count": snapshot.staged_count,
                    "unstaged_count": snapshot.unstaged_count,
                    "untracked_count": snapshot.untracked_count,
                    "conflicted_count": snapshot.conflicted_count,
                    "operation_states": snapshot.operation_states,
                    "large_diff": snapshot.large_diff,
                    "top_changed_directories": snapshot.top_changed_directories.iter().map(|(dir, count)| json!({
                        "directory": dir,
                        "changed_files": count,
                    })).collect::<Vec<_>>(),
                    "lockfile_anomalies": snapshot.lockfile_anomalies,
                    "evidence": snapshot.evidence,
                    "risk_reasons": snapshot.risk_reasons,
                    "risk_findings": findings,
                    "highest_priority_finding": highest_priority_finding,
                    "finding_counts": finding_counts,
                })
            } else {
                json!({
                    "status": "error",
                    "summary": "Git repository detected, but OPENDOG could not collect a stable risk snapshot.",
                    "risk_findings": [],
                    "highest_priority_finding": Value::Null,
                    "finding_counts": {
                        "total": 0,
                        "high": 0,
                        "medium": 0,
                        "low": 0
                    },
                })
            }
        }
        Ok(_) => json!({
            "status": "not_git_repository",
            "summary": "The registered root is not inside a Git work tree, so repository risk signals are unavailable.",
            "risk_findings": [],
            "highest_priority_finding": Value::Null,
            "finding_counts": {
                "total": 0,
                "high": 0,
                "medium": 0,
                "low": 0
            },
        }),
        Err(e) => json!({
            "status": "error",
            "summary": "Failed to execute git for repository risk collection.",
            "error": e.to_string(),
            "risk_findings": [],
            "highest_priority_finding": Value::Null,
            "finding_counts": {
                "total": 0,
                "high": 0,
                "medium": 0,
                "low": 0
            },
        }),
    }
}

#[cfg(test)]
mod tests {
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

    fn make_snapshot(
        changed_file_count: usize,
        conflicted_count: usize,
        staged_count: usize,
        unstaged_count: usize,
        untracked_count: usize,
        large_diff: bool,
        operation_states: Vec<&str>,
        lockfile_anomalies: Vec<Value>,
    ) -> RepoRiskSnapshot {
        RepoRiskSnapshot {
            status: "available",
            branch: Some("test-branch".to_string()),
            is_dirty: changed_file_count > 0,
            changed_file_count,
            staged_count,
            unstaged_count,
            untracked_count,
            conflicted_count,
            operation_states: operation_states.into_iter().map(|s| s.to_string()).collect(),
            top_changed_directories: vec![],
            large_diff,
            lockfile_anomalies,
            evidence: vec![format!("Changed files: {}.", changed_file_count)],
            risk_reasons: vec![],
            risk_level: "low",
        }
    }

    #[test]
    fn findings_empty_clean_repo() {
        let snap = make_snapshot(0, 0, 0, 0, 0, false, vec![], vec![]);
        let findings = repo_risk_findings(&snap);
        assert!(findings.is_empty());
    }

    #[test]
    fn findings_conflicted_paths() {
        let snap = make_snapshot(3, 2, 1, 2, 0, false, vec![], vec![]);
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
        let snap = make_snapshot(0, 0, 0, 0, 0, false, vec!["merge"], vec![]);
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
        let snap = make_snapshot(30, 0, 10, 20, 5, true, vec![], vec![]);
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
        let snap = make_snapshot(2, 0, 1, 1, 0, false, vec![], vec![anomaly]);
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
        let snap = make_snapshot(5, 0, 2, 3, 1, false, vec![], vec![]);
        let findings = repo_risk_findings(&snap);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0]["kind"], "local_working_tree_changes");
        assert_eq!(findings[0]["severity"], "low");
        assert_eq!(findings[0]["priority"], "medium");
    }

    #[test]
    fn findings_sorted_by_priority_then_severity() {
        let anomaly = json!({"kind": "manifest_without_lockfile_change", "directory": "."});
        let snap = make_snapshot(30, 2, 10, 20, 5, true, vec!["rebase"], vec![anomaly]);
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
}
