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
