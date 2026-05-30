use super::*;

#[test]
fn blocker_reasons_no_blockers() {
    let reasons = blocker_reasons("cleanup", &[], &[], &[]);
    assert!(reasons.is_empty());
}

#[test]
fn blocker_reasons_failing_kinds() {
    let reasons = blocker_reasons("cleanup", &[], &[], &["test"]);
    assert_eq!(reasons.len(), 1);
    assert!(reasons[0].contains("failing or uncertain"));
}

#[test]
fn blocker_reasons_missing_test() {
    let reasons = blocker_reasons("cleanup", &["test"], &[], &[]);
    assert!(reasons
        .iter()
        .any(|r| r.contains("Missing recorded test evidence")));
}

#[test]
fn blocker_reasons_missing_build_for_refactor() {
    let reasons = blocker_reasons("refactor", &["test", "build"], &[], &[]);
    assert!(reasons.iter().any(|r| r.contains("Missing recorded test")));
    assert!(reasons.iter().any(|r| r.contains("Missing recorded build")));
}

#[test]
fn blocker_reasons_missing_build_not_reported_for_cleanup() {
    let reasons = blocker_reasons("cleanup", &["build"], &[], &[]);
    // "build" is not "test", so the test-missing check won't fire.
    // And the refactor-specific build check won't fire for cleanup target.
    assert!(reasons.is_empty());
}

#[test]
fn blocker_reasons_stale() {
    let reasons = blocker_reasons("cleanup", &[], &["test"], &[]);
    assert!(reasons.iter().any(|r| r.contains("stale")));
}

#[test]
fn blocker_reasons_all_combined() {
    let reasons = blocker_reasons("refactor", &["test", "build"], &["lint"], &["lint"]);
    // failing + missing test + missing build + stale = 4
    assert_eq!(reasons.len(), 4);
}

// ---- gate_reasons ----

#[test]
fn gate_reasons_returns_blocker_reasons_when_blockers_exist() {
    let reasons = gate_reasons("cleanup", &["test"], &[], &[], &[], &[]);
    assert!(!reasons.is_empty());
    assert!(reasons.iter().any(|r| r.contains("Missing recorded test")));
}

#[test]
fn gate_reasons_advisory_gaps_when_no_blockers() {
    let reasons = gate_reasons("cleanup", &[], &[], &["lint"], &[], &[]);
    assert_eq!(reasons.len(), 1);
    assert!(reasons[0].contains("Advisory verification evidence is incomplete"));
}

#[test]
fn gate_reasons_empty_when_fully_satisfied() {
    let reasons = gate_reasons("cleanup", &[], &[], &[], &[], &[]);
    assert!(reasons.is_empty());
}

#[test]
fn gate_reasons_advisory_stale_only() {
    let reasons = gate_reasons("cleanup", &[], &[], &[], &["lint"], &[]);
    assert_eq!(reasons.len(), 1);
    assert!(reasons[0].contains("Advisory verification evidence is incomplete"));
}

// ---- gate_next_steps ----

#[test]
fn gate_next_steps_failing_kinds() {
    let steps = gate_next_steps("cleanup", &[], &[], &[], &[], &["test"]);
    assert!(steps.iter().any(|s| s.contains("Stabilize failing")));
}

#[test]
fn gate_next_steps_missing_test() {
    let steps = gate_next_steps("cleanup", &["test"], &[], &[], &[], &[]);
    assert!(steps
        .iter()
        .any(|s| s.contains("Run and record project-native test")));
}

#[test]
fn gate_next_steps_missing_build_for_refactor() {
    let steps = gate_next_steps("refactor", &["test", "build"], &[], &[], &[], &[]);
    assert!(steps
        .iter()
        .any(|s| s.contains("Run and record project-native test")));
    assert!(steps
        .iter()
        .any(|s| s.contains("Run and record project-native build")));
}

#[test]
fn gate_next_steps_stale() {
    let steps = gate_next_steps("cleanup", &[], &["test"], &[], &[], &[]);
    assert!(steps.iter().any(|s| s.contains("Refresh stale")));
}

#[test]
fn gate_next_steps_advisory_gaps_only() {
    let steps = gate_next_steps("cleanup", &[], &[], &["lint"], &[], &[]);
    assert!(steps
        .iter()
        .any(|s| s.contains("Refresh advisory verification")));
}

#[test]
fn gate_next_steps_fully_satisfied() {
    let steps = gate_next_steps("cleanup", &[], &[], &[], &[], &[]);
    assert_eq!(steps.len(), 1);
    assert!(steps[0].contains("supports the requested review mode"));
}

// ---- gate_assessment ----

#[test]
fn gate_blockers_empty_when_all_good() {
    let runs = vec![
        make_run("test", "passed", &NOW.to_string()),
        make_run("lint", "passed", &NOW.to_string()),
        make_run("build", "passed", &NOW.to_string()),
    ];
    let blockers = gate_blockers(&runs, "cleanup", NOW);
    assert!(blockers.is_empty());
}

#[test]
fn gate_blockers_present_when_missing_required() {
    let runs: Vec<VerificationRun> = vec![];
    let blockers = gate_blockers(&runs, "cleanup", NOW);
    assert!(!blockers.is_empty());
}

// ---- verification_has_failures ----
