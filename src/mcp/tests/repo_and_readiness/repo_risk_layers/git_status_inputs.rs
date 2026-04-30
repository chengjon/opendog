use super::*;

#[test]
fn parse_status_porcelain_extracts_branch_and_paths() {
    let (branch, entries) = parse_status_porcelain(
        "## main...origin/main\nM  src/main.rs\n?? Cargo.lock\nR  old.rs -> new.rs\n",
    );

    assert_eq!(branch.as_deref(), Some("main"));
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].path, "src/main.rs");
    assert_eq!(entries[1].path, "Cargo.lock");
    assert_eq!(entries[2].path, "new.rs");
}

#[test]
fn detect_lockfile_anomalies_flags_missing_manifest_pair() {
    let anomalies = detect_lockfile_anomalies(&[
        GitStatusEntry {
            staged: 'M',
            unstaged: ' ',
            path: "Cargo.lock".to_string(),
        },
        GitStatusEntry {
            staged: 'M',
            unstaged: ' ',
            path: "src/main.rs".to_string(),
        },
    ]);

    assert_eq!(anomalies.len(), 1);
    assert_eq!(anomalies[0]["kind"], "lockfile_without_manifest_change");
    assert_eq!(anomalies[0]["expected_manifest"], "Cargo.toml");
}
