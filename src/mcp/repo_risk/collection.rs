use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use super::{GitStatusEntry, RepoRiskSnapshot};

pub(in crate::mcp) fn git_output(
    root: &Path,
    args: &[&str],
) -> std::io::Result<std::process::Output> {
    Command::new("git").args(args).current_dir(root).output()
}

fn git_stdout(root: &Path, args: &[&str]) -> Option<String> {
    let output = git_output(root, args).ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub(in crate::mcp) fn parse_status_porcelain(
    stdout: &str,
) -> (Option<String>, Vec<GitStatusEntry>) {
    let mut branch = None;
    let mut entries = Vec::new();

    for (idx, line) in stdout.lines().enumerate() {
        if idx == 0 && line.starts_with("## ") {
            branch = Some(
                line.trim_start_matches("## ")
                    .split("...")
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
            );
            continue;
        }
        if line.len() < 3 {
            continue;
        }
        let staged = line.chars().next().unwrap_or(' ');
        let unstaged = line.chars().nth(1).unwrap_or(' ');
        let mut path = line[3..].trim().to_string();
        if let Some((_, renamed_to)) = path.split_once(" -> ") {
            path = renamed_to.trim().to_string();
        }
        entries.push(GitStatusEntry {
            staged,
            unstaged,
            path,
        });
    }

    (branch, entries)
}

fn git_operation_states(root: &Path) -> Vec<String> {
    let checks = [
        ("MERGE_HEAD", "merge"),
        ("REBASE_HEAD", "rebase"),
        ("CHERRY_PICK_HEAD", "cherry-pick"),
        ("BISECT_LOG", "bisect"),
    ];

    checks
        .iter()
        .filter_map(|(git_path_name, label)| {
            let resolved = git_stdout(root, &["rev-parse", "--git-path", git_path_name])?;
            if Path::new(&resolved).exists() {
                Some((*label).to_string())
            } else {
                None
            }
        })
        .collect()
}

fn top_changed_directories(entries: &[GitStatusEntry]) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for entry in entries {
        let dir = Path::new(&entry.path)
            .parent()
            .map(|p| {
                let s = p.to_string_lossy().to_string();
                if s.is_empty() {
                    ".".to_string()
                } else {
                    s
                }
            })
            .unwrap_or_else(|| ".".to_string());
        *counts.entry(dir).or_insert(0) += 1;
    }

    let mut items: Vec<(String, usize)> = counts.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    items.truncate(5);
    items
}

pub(in crate::mcp) fn detect_lockfile_anomalies(entries: &[GitStatusEntry]) -> Vec<Value> {
    let mut changed_by_dir: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for entry in entries {
        let dir = Path::new(&entry.path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let basename = Path::new(&entry.path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| entry.path.clone());
        changed_by_dir.entry(dir).or_default().push(basename);
    }

    let rules = [
        ("Cargo.lock", "Cargo.toml"),
        ("package-lock.json", "package.json"),
        ("pnpm-lock.yaml", "package.json"),
        ("yarn.lock", "package.json"),
        ("poetry.lock", "pyproject.toml"),
        ("Pipfile.lock", "Pipfile"),
    ];

    let mut anomalies = Vec::new();
    for (dir, files) in changed_by_dir {
        for (lockfile, manifest) in rules {
            let lock_changed = files.iter().any(|f| f == lockfile);
            let manifest_changed = files.iter().any(|f| f == manifest);
            if lock_changed && !manifest_changed {
                anomalies.push(json!({
                    "kind": "lockfile_without_manifest_change",
                    "directory": if dir.is_empty() { "." } else { &dir },
                    "lockfile": lockfile,
                    "expected_manifest": manifest,
                }));
            }
            if manifest_changed && !lock_changed {
                anomalies.push(json!({
                    "kind": "manifest_without_lockfile_change",
                    "directory": if dir.is_empty() { "." } else { &dir },
                    "manifest": manifest,
                    "expected_lockfile": lockfile,
                }));
            }
        }
    }
    anomalies
}

pub(in crate::mcp) fn collect_repo_risk_snapshot(root: &Path) -> Option<RepoRiskSnapshot> {
    let git_root = git_stdout(root, &["rev-parse", "--show-toplevel"])?;
    let status_output = git_stdout(root, &["status", "--porcelain=1", "-b"])?;
    let repo_root = Path::new(&git_root);
    let (branch, entries) = parse_status_porcelain(&status_output);
    let operation_states = git_operation_states(repo_root);

    let staged_count = entries
        .iter()
        .filter(|e| e.staged != ' ' && e.staged != '?')
        .count();
    let unstaged_count = entries.iter().filter(|e| e.unstaged != ' ').count();
    let untracked_count = entries
        .iter()
        .filter(|e| e.staged == '?' && e.unstaged == '?')
        .count();
    let conflicted_count = entries
        .iter()
        .filter(|e| matches!(e.staged, 'U' | 'A' | 'D') && matches!(e.unstaged, 'U' | 'A' | 'D'))
        .count();
    let changed_file_count = entries.len();
    let top_changed_directories = top_changed_directories(&entries);
    let large_diff = changed_file_count >= 25;
    let lockfile_anomalies = detect_lockfile_anomalies(&entries);

    let mut evidence = vec![format!(
        "Changed files observed via git status: {}.",
        changed_file_count
    )];
    if let Some(branch_name) = &branch {
        evidence.push(format!("Current branch: {}.", branch_name));
    }
    if !operation_states.is_empty() {
        evidence.push(format!(
            "Git operation in progress: {}.",
            operation_states.join(", ")
        ));
    }
    if let Some((dir, count)) = top_changed_directories.first() {
        evidence.push(format!(
            "Most changed directory in current working state: {} ({} files).",
            dir, count
        ));
    }

    let mut risk_reasons = Vec::new();
    if conflicted_count > 0 {
        risk_reasons.push(format!("{} conflicted paths detected.", conflicted_count));
    }
    if !operation_states.is_empty() {
        risk_reasons.push(format!(
            "Repository is mid-operation: {}.",
            operation_states.join(", ")
        ));
    }
    if large_diff {
        risk_reasons.push(format!(
            "Large working diff detected ({} changed files).",
            changed_file_count
        ));
    }
    if !lockfile_anomalies.is_empty() {
        risk_reasons.push(format!(
            "{} dependency lock/manifest mismatch signals detected.",
            lockfile_anomalies.len()
        ));
    }
    if changed_file_count > 0 && risk_reasons.is_empty() {
        risk_reasons.push(
            "Repository has local changes but no higher-risk signal was detected.".to_string(),
        );
    }

    let risk_level = if conflicted_count > 0 || !operation_states.is_empty() {
        "high"
    } else if large_diff || !lockfile_anomalies.is_empty() {
        "medium"
    } else {
        "low"
    };

    Some(RepoRiskSnapshot {
        status: "available",
        branch,
        is_dirty: changed_file_count > 0,
        changed_file_count,
        staged_count,
        unstaged_count,
        untracked_count,
        conflicted_count,
        operation_states,
        top_changed_directories,
        large_diff,
        lockfile_anomalies,
        evidence,
        risk_reasons,
        risk_level,
    })
}
