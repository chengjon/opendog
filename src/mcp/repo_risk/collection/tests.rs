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
