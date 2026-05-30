use super::*;

#[test]
fn gate_kinds_cleanup_returns_test_required() {
    let (required, advisory) = gate_kinds("cleanup");
    assert_eq!(required, &["test"]);
    assert_eq!(advisory, &["lint", "build"]);
}

#[test]
fn gate_kinds_refactor_returns_test_build_required() {
    let (required, advisory) = gate_kinds("refactor");
    assert_eq!(required, &["test", "build"]);
    assert_eq!(advisory, &["lint"]);
}

#[test]
fn gate_kinds_unknown_target_defaults_to_cleanup() {
    let (required, advisory) = gate_kinds("other");
    assert_eq!(required, &["test"]);
    assert_eq!(advisory, &["lint", "build"]);
}

// ---- failing_kinds ----

#[test]
fn failing_kinds_empty_runs() {
    let runs: Vec<VerificationRun> = vec![];
    assert!(failing_kinds(&runs).is_empty());
}

#[test]
fn failing_kinds_all_passed() {
    let runs = vec![
        make_run("test", "passed", "1700000000"),
        make_run("lint", "passed", "1700000000"),
    ];
    assert!(failing_kinds(&runs).is_empty());
}

#[test]
fn failing_kinds_some_failing() {
    let runs = vec![
        make_run("test", "passed", "1700000000"),
        make_run("lint", "failed", "1700000000"),
        make_run("build", "uncertain", "1700000000"),
    ];
    let failed = failing_kinds(&runs);
    assert_eq!(failed, vec!["lint", "build"]);
}

// ---- kind_state_sets ----

#[test]
fn kind_state_sets_all_missing() {
    let runs: Vec<VerificationRun> = vec![];
    let (missing, stale) = kind_state_sets(&runs, &["test", "build"], NOW);
    assert_eq!(missing, vec!["test", "build"]);
    assert!(stale.is_empty());
}

#[test]
fn kind_state_sets_none_missing_fresh() {
    let runs = vec![
        make_run("test", "passed", &NOW.to_string()),
        make_run("build", "passed", &NOW.to_string()),
    ];
    let (missing, stale) = kind_state_sets(&runs, &["test", "build"], NOW);
    assert!(missing.is_empty());
    assert!(stale.is_empty());
}

#[test]
fn kind_state_sets_stale_runs() {
    let old_ts = (NOW - 10 * 86400).to_string();
    let runs = vec![make_run("test", "passed", &old_ts)];
    let (missing, stale) = kind_state_sets(&runs, &["test", "build"], NOW);
    assert_eq!(missing, vec!["build"]);
    assert_eq!(stale, vec!["test"]);
}

#[test]
fn kind_state_sets_partial_kinds_present() {
    let runs = vec![make_run("test", "passed", &NOW.to_string())];
    let (missing, stale) = kind_state_sets(&runs, &["test"], NOW);
    assert!(missing.is_empty());
    assert!(stale.is_empty());
}

// ---- blocker_reasons ----
