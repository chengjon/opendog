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

#[cfg(test)]
mod tests {
    use super::*;

    // ---- parse_status_porcelain ----

    #[test]
    fn parse_status_empty_string() {
        let (branch, entries) = parse_status_porcelain("");
        assert_eq!(branch, None);
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_status_branch_and_entries() {
        let input = "## main...origin/main\n M src/main.rs\n?? new_file.txt";
        let (branch, entries) = parse_status_porcelain(input);
        assert_eq!(branch.as_deref(), Some("main"));
        assert_eq!(entries.len(), 2);

        assert_eq!(entries[0].staged, ' ');
        assert_eq!(entries[0].unstaged, 'M');
        assert_eq!(entries[0].path, "src/main.rs");

        assert_eq!(entries[1].staged, '?');
        assert_eq!(entries[1].unstaged, '?');
        assert_eq!(entries[1].path, "new_file.txt");
    }

    #[test]
    fn parse_status_various_xy_codes() {
        let input = "## develop\nM  staged_only.rs\n D deleted_unstaged.rs\nA  new_staged.rs\n!! ignored_file.log";
        let (branch, entries) = parse_status_porcelain(input);
        assert_eq!(branch.as_deref(), Some("develop"));
        assert_eq!(entries.len(), 4);

        // M  staged_only.rs  (staged='M', unstaged=' ')
        assert_eq!(entries[0].staged, 'M');
        assert_eq!(entries[0].unstaged, ' ');
        assert_eq!(entries[0].path, "staged_only.rs");

        //  D deleted_unstaged.rs  (staged=' ', unstaged='D')
        assert_eq!(entries[1].staged, ' ');
        assert_eq!(entries[1].unstaged, 'D');
        assert_eq!(entries[1].path, "deleted_unstaged.rs");

        // A  new_staged.rs
        assert_eq!(entries[2].staged, 'A');
        assert_eq!(entries[2].unstaged, ' ');
        assert_eq!(entries[2].path, "new_staged.rs");

        // !! ignored_file.log
        assert_eq!(entries[3].staged, '!');
        assert_eq!(entries[3].unstaged, '!');
        assert_eq!(entries[3].path, "ignored_file.log");
    }

    #[test]
    fn parse_status_rename_entry() {
        let input = "R  old_name.rs -> new_name.rs";
        let (branch, entries) = parse_status_porcelain(input);
        assert!(branch.is_none());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].staged, 'R');
        assert_eq!(entries[0].path, "new_name.rs");
    }

    #[test]
    fn parse_status_branch_only() {
        let input = "## feature-branch...origin/feature-branch";
        let (branch, entries) = parse_status_porcelain(input);
        assert_eq!(branch.as_deref(), Some("feature-branch"));
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_status_entries_only_no_branch() {
        let input = " M modified.rs\n?? untracked.rs";
        let (branch, entries) = parse_status_porcelain(input);
        assert!(branch.is_none());
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, "modified.rs");
        assert_eq!(entries[1].path, "untracked.rs");
    }

    #[test]
    fn parse_status_short_lines_skipped() {
        let input = "## main\nab\n M ok.rs";
        let (branch, entries) = parse_status_porcelain(input);
        assert_eq!(branch.as_deref(), Some("main"));
        // "ab" is length 2 < 3, so it's skipped
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "ok.rs");
    }

    #[test]
    fn parse_status_branch_no_upstream() {
        let input = "## feature-solo";
        let (branch, entries) = parse_status_porcelain(input);
        assert_eq!(branch.as_deref(), Some("feature-solo"));
        assert!(entries.is_empty());
    }

    // ---- detect_lockfile_anomalies ----

    #[test]
    fn lockfile_anomaly_manifest_without_lockfile() {
        // package.json matches 3 lockfile rules (package-lock.json, pnpm-lock.yaml, yarn.lock)
        let entries = vec![GitStatusEntry {
            staged: ' ',
            unstaged: 'M',
            path: "package.json".to_string(),
        }];
        let anomalies = detect_lockfile_anomalies(&entries);
        assert_eq!(anomalies.len(), 3);
        for anomaly in &anomalies {
            assert_eq!(
                anomaly["kind"].as_str().unwrap(),
                "manifest_without_lockfile_change"
            );
            assert_eq!(anomaly["manifest"].as_str().unwrap(), "package.json");
        }
    }

    #[test]
    fn lockfile_anomaly_lockfile_without_manifest() {
        let entries = vec![GitStatusEntry {
            staged: ' ',
            unstaged: 'M',
            path: "Cargo.lock".to_string(),
        }];
        let anomalies = detect_lockfile_anomalies(&entries);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(
            anomalies[0]["kind"].as_str().unwrap(),
            "lockfile_without_manifest_change"
        );
        assert_eq!(anomalies[0]["lockfile"].as_str().unwrap(), "Cargo.lock");
    }

    #[test]
    fn lockfile_no_anomaly_both_changed() {
        // package.json + package-lock.json resolves one rule, but pnpm-lock.yaml and yarn.lock
        // still produce manifest_without_lockfile_change anomalies (2 remaining).
        let entries = vec![
            GitStatusEntry {
                staged: ' ',
                unstaged: 'M',
                path: "package.json".to_string(),
            },
            GitStatusEntry {
                staged: ' ',
                unstaged: 'M',
                path: "package-lock.json".to_string(),
            },
        ];
        let anomalies = detect_lockfile_anomalies(&entries);
        assert_eq!(anomalies.len(), 2);
        for a in &anomalies {
            assert_eq!(a["kind"], "manifest_without_lockfile_change");
        }
    }

    #[test]
    fn lockfile_no_anomaly_neither_changed() {
        let entries = vec![GitStatusEntry {
            staged: ' ',
            unstaged: 'M',
            path: "src/main.rs".to_string(),
        }];
        let anomalies = detect_lockfile_anomalies(&entries);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn lockfile_anomaly_yarn_pattern() {
        let entries = vec![GitStatusEntry {
            staged: 'M',
            unstaged: ' ',
            path: "yarn.lock".to_string(),
        }];
        let anomalies = detect_lockfile_anomalies(&entries);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0]["lockfile"].as_str().unwrap(), "yarn.lock");
        assert_eq!(
            anomalies[0]["expected_manifest"].as_str().unwrap(),
            "package.json"
        );
    }

    #[test]
    fn lockfile_anomaly_pnpm_pattern() {
        let entries = vec![GitStatusEntry {
            staged: ' ',
            unstaged: 'M',
            path: "pnpm-lock.yaml".to_string(),
        }];
        let anomalies = detect_lockfile_anomalies(&entries);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0]["lockfile"].as_str().unwrap(), "pnpm-lock.yaml");
    }

    #[test]
    fn lockfile_anomaly_cargo_pattern() {
        let entries = vec![GitStatusEntry {
            staged: ' ',
            unstaged: 'M',
            path: "Cargo.lock".to_string(),
        }];
        let anomalies = detect_lockfile_anomalies(&entries);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0]["lockfile"].as_str().unwrap(), "Cargo.lock");
        assert_eq!(
            anomalies[0]["expected_manifest"].as_str().unwrap(),
            "Cargo.toml"
        );
    }

    #[test]
    fn lockfile_anomaly_poetry_pattern() {
        let entries = vec![GitStatusEntry {
            staged: ' ',
            unstaged: 'M',
            path: "pyproject.toml".to_string(),
        }];
        let anomalies = detect_lockfile_anomalies(&entries);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(
            anomalies[0]["kind"].as_str().unwrap(),
            "manifest_without_lockfile_change"
        );
        assert_eq!(anomalies[0]["manifest"].as_str().unwrap(), "pyproject.toml");
    }

    #[test]
    fn lockfile_anomaly_pipfile_pattern() {
        let entries = vec![GitStatusEntry {
            staged: ' ',
            unstaged: 'M',
            path: "Pipfile".to_string(),
        }];
        let anomalies = detect_lockfile_anomalies(&entries);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0]["manifest"].as_str().unwrap(), "Pipfile");
    }

    #[test]
    fn lockfile_anomaly_in_subdirectory() {
        // package.json in subdir matches 3 lockfile rules
        let entries = vec![GitStatusEntry {
            staged: ' ',
            unstaged: 'M',
            path: "subdir/package.json".to_string(),
        }];
        let anomalies = detect_lockfile_anomalies(&entries);
        assert_eq!(anomalies.len(), 3);
        for a in &anomalies {
            assert_eq!(a["directory"].as_str().unwrap(), "subdir");
        }
    }

    // ---- top_changed_directories ----

    #[test]
    fn top_changed_dirs_multiple_directories() {
        let entries = vec![
            GitStatusEntry {
                staged: ' ',
                unstaged: 'M',
                path: "src/a.rs".to_string(),
            },
            GitStatusEntry {
                staged: ' ',
                unstaged: 'M',
                path: "src/b.rs".to_string(),
            },
            GitStatusEntry {
                staged: ' ',
                unstaged: 'M',
                path: "src/c.rs".to_string(),
            },
            GitStatusEntry {
                staged: ' ',
                unstaged: 'M',
                path: "docs/readme.md".to_string(),
            },
            GitStatusEntry {
                staged: ' ',
                unstaged: 'M',
                path: "root_file.txt".to_string(),
            },
        ];
        let top = top_changed_directories(&entries);
        assert_eq!(top.len(), 3);
        // src has 3 changes, should be first
        assert_eq!(top[0], ("src".to_string(), 3));
        // docs has 1, root has 1 (represented as ".")
        // "." sorts before "docs" alphabetically
        assert_eq!(top[1], (".".to_string(), 1));
        assert_eq!(top[2], ("docs".to_string(), 1));
    }

    #[test]
    fn top_changed_dirs_truncates_to_five() {
        let mut entries = Vec::new();
        for i in 0..8 {
            entries.push(GitStatusEntry {
                staged: ' ',
                unstaged: 'M',
                path: format!("dir{}/file.rs", i),
            });
        }
        let top = top_changed_directories(&entries);
        assert!(top.len() <= 5);
    }

    #[test]
    fn top_changed_dirs_empty_entries() {
        let entries: Vec<GitStatusEntry> = vec![];
        let top = top_changed_directories(&entries);
        assert!(top.is_empty());
    }

    #[test]
    fn top_changed_dirs_single_entry_root() {
        let entries = vec![GitStatusEntry {
            staged: ' ',
            unstaged: 'M',
            path: "toplevel.txt".to_string(),
        }];
        let top = top_changed_directories(&entries);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0], (".".to_string(), 1));
    }

    #[test]
    fn top_changed_dirs_sorting_tiebreak_alphabetical() {
        let entries = vec![
            GitStatusEntry {
                staged: ' ',
                unstaged: 'M',
                path: "beta/f1.rs".to_string(),
            },
            GitStatusEntry {
                staged: ' ',
                unstaged: 'M',
                path: "alpha/f2.rs".to_string(),
            },
        ];
        let top = top_changed_directories(&entries);
        // Both have count 1, so sorted alphabetically
        assert_eq!(top[0].0, "alpha");
        assert_eq!(top[1].0, "beta");
    }
}
