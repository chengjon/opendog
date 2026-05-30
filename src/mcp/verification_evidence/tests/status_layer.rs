use super::*;

#[test]
fn verification_has_failures_empty() {
    let runs: Vec<VerificationRun> = vec![];
    assert!(!verification_has_failures(&runs));
}

#[test]
fn verification_has_failures_all_passed() {
    let runs = vec![make_run("test", "passed", "1700000000")];
    assert!(!verification_has_failures(&runs));
}

#[test]
fn verification_has_failures_with_failure() {
    let runs = vec![make_run("test", "failed", "1700000000")];
    assert!(verification_has_failures(&runs));
}

#[test]
fn verification_has_failures_uncertain() {
    let runs = vec![make_run("build", "uncertain", "1700000000")];
    assert!(verification_has_failures(&runs));
}

// ---- verification_is_missing ----

#[test]
fn verification_is_missing_empty() {
    let runs: Vec<VerificationRun> = vec![];
    assert!(verification_is_missing(&runs));
}

#[test]
fn verification_is_missing_with_runs() {
    let runs = vec![make_run("test", "passed", "1700000000")];
    assert!(!verification_is_missing(&runs));
}

// ---- project_gate_level ----

#[test]
fn project_gate_level_from_assessment() {
    let project = json!({
        "verification_evidence": {
            "gate_assessment": {
                "cleanup": { "level": "allow" },
                "refactor": { "level": "caution" },
            }
        }
    });
    assert_eq!(project_gate_level(&project, "cleanup"), "allow");
    assert_eq!(project_gate_level(&project, "refactor"), "caution");
}

#[test]
fn project_gate_level_fallback_safe_for_cleanup() {
    let project = json!({
        "safe_for_cleanup": true,
    });
    assert_eq!(project_gate_level(&project, "cleanup"), "allow");
}

#[test]
fn project_gate_level_fallback_safe_for_refactor() {
    let project = json!({
        "safe_for_refactor": true,
    });
    assert_eq!(project_gate_level(&project, "refactor"), "allow");
}

#[test]
fn project_gate_level_fallback_blocked() {
    let project = json!({
        "safe_for_cleanup": false,
    });
    assert_eq!(project_gate_level(&project, "cleanup"), "blocked");
}

// ---- verification_status_layer ----

#[test]
fn verification_status_layer_empty_runs() {
    let runs: Vec<VerificationRun> = vec![];
    let result = verification_status_layer(&runs);
    assert_eq!(result["status"], "not_recorded");
    assert_eq!(result["missing_kinds"], json!(["test", "lint", "build"]));
    assert_eq!(result["all_expected_kinds_recorded"], false);
    assert_eq!(result["safe_for_cleanup"], false);
    assert_eq!(result["safe_for_refactor"], false);
    assert!(result["latest_runs"].as_array().unwrap().is_empty());
}

#[test]
fn verification_status_layer_all_passing() {
    let ts = current_unix_secs().to_string();
    let runs = vec![
        make_run("test", "passed", &ts),
        make_run("lint", "passed", &ts),
        make_run("build", "passed", &ts),
    ];
    let result = verification_status_layer(&runs);
    assert_eq!(result["status"], "available");
    assert_eq!(result["all_expected_kinds_recorded"], true);
    assert_eq!(result["safe_for_cleanup"], true);
    assert_eq!(result["safe_for_refactor"], true);
    assert!(result["failing_runs"].as_array().unwrap().is_empty());
}

#[test]
fn verification_status_layer_with_failures() {
    let ts = current_unix_secs().to_string();
    let runs = vec![
        make_run("test", "passed", &ts),
        make_run("lint", "failed", &ts),
    ];
    let result = verification_status_layer(&runs);
    assert_eq!(result["status"], "available");
    assert_eq!(result["safe_for_cleanup"], false);
    let failing = result["failing_runs"].as_array().unwrap();
    assert_eq!(failing.len(), 1);
    assert_eq!(failing[0]["kind"], "lint");
}

#[test]
fn verification_status_layer_partial_kinds() {
    let ts = current_unix_secs().to_string();
    let runs = vec![make_run("test", "passed", &ts)];
    let result = verification_status_layer(&runs);
    assert_eq!(result["status"], "available");
    assert_eq!(result["all_expected_kinds_recorded"], false);
    let missing = result["missing_kinds"].as_array().unwrap();
    assert!(missing.iter().any(|k| k == "lint"));
    assert!(missing.iter().any(|k| k == "build"));
}

#[test]
fn verification_status_layer_has_gate_assessment() {
    let runs: Vec<VerificationRun> = vec![];
    let result = verification_status_layer(&runs);
    assert!(result["gate_assessment"]["cleanup"].is_object());
    assert!(result["gate_assessment"]["refactor"].is_object());
}

#[test]
fn verification_status_layer_has_freshness() {
    let ts = current_unix_secs().to_string();
    let runs = vec![make_run("test", "passed", &ts)];
    let result = verification_status_layer(&runs);
    assert!(result["freshness"].is_object());
    assert_eq!(result["freshness"]["label"], "verification");
}

#[test]
fn verification_status_layer_stale_runs_not_safe() {
    // Use a timestamp far enough in the past to be stale (>7 days)
    let old_ts = (current_unix_secs() - 10 * 86400).to_string();
    let runs = vec![make_run("test", "passed", &old_ts)];
    let result = verification_status_layer(&runs);
    // Stale required test => blocked for cleanup
    assert_eq!(result["safe_for_cleanup"], false);
}

// ---- workspace_verification_evidence_layer ----
