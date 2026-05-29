use super::*;
use crate::storage::queries::VerificationRun;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

const NOW: i64 = 1_700_000_000;

fn current_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn make_run(kind: &str, status: &str, finished_at: &str) -> VerificationRun {
    VerificationRun {
        id: 1,
        kind: kind.to_string(),
        status: status.to_string(),
        command: format!("run-{}", kind),
        exit_code: Some(0),
        summary: Some(format!("{} summary", kind)),
        source: "test".to_string(),
        started_at: Some(finished_at.to_string()),
        finished_at: finished_at.to_string(),
    }
}

// ---- gate_kinds ----

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
fn gate_assessment_blocked_when_no_runs() {
    let runs: Vec<VerificationRun> = vec![];
    let result = gate_assessment(&runs, "cleanup", NOW);
    assert_eq!(result["level"], "blocked");
    assert_eq!(result["allowed"], false);
    assert!(!result["missing_kinds"].as_array().unwrap().is_empty());
}

#[test]
fn gate_assessment_blocked_when_failing() {
    let runs = vec![make_run("test", "failed", &NOW.to_string())];
    let result = gate_assessment(&runs, "cleanup", NOW);
    assert_eq!(result["level"], "blocked");
    assert_eq!(result["allowed"], false);
}

#[test]
fn gate_assessment_caution_when_advisory_missing() {
    // cleanup: required=["test"], advisory=["lint","build"]
    // Provide test only
    let runs = vec![make_run("test", "passed", &NOW.to_string())];
    let result = gate_assessment(&runs, "cleanup", NOW);
    assert_eq!(result["level"], "caution");
    assert_eq!(result["allowed"], true);
}

#[test]
fn gate_assessment_allow_when_all_present_and_fresh() {
    let runs = vec![
        make_run("test", "passed", &NOW.to_string()),
        make_run("lint", "passed", &NOW.to_string()),
        make_run("build", "passed", &NOW.to_string()),
    ];
    let result = gate_assessment(&runs, "cleanup", NOW);
    assert_eq!(result["level"], "allow");
    assert_eq!(result["allowed"], true);
    assert!(result["missing_kinds"].as_array().unwrap().is_empty());
}

#[test]
fn gate_assessment_refactor_requires_build() {
    // refactor: required=["test","build"], advisory=["lint"]
    let runs = vec![make_run("test", "passed", &NOW.to_string())];
    let result = gate_assessment(&runs, "refactor", NOW);
    assert_eq!(result["level"], "blocked");
    assert_eq!(result["allowed"], false);
}

#[test]
fn gate_assessment_refactor_allow_with_all() {
    let runs = vec![
        make_run("test", "passed", &NOW.to_string()),
        make_run("build", "passed", &NOW.to_string()),
        make_run("lint", "passed", &NOW.to_string()),
    ];
    let result = gate_assessment(&runs, "refactor", NOW);
    assert_eq!(result["level"], "allow");
    assert_eq!(result["allowed"], true);
}

#[test]
fn gate_assessment_includes_freshness_policy() {
    let runs: Vec<VerificationRun> = vec![];
    let result = gate_assessment(&runs, "cleanup", NOW);
    assert!(result["freshness_policy"].is_object());
}

// ---- pipeline caution ----

fn make_pipeline_run(kind: &str, status: &str, finished_at: &str) -> VerificationRun {
    VerificationRun {
        id: 1,
        kind: kind.to_string(),
        status: status.to_string(),
        command: "npx vue-tsc --noEmit 2>&1 | tail -30".to_string(),
        exit_code: Some(0),
        summary: Some(format!("{} summary", kind)),
        source: "test".to_string(),
        started_at: Some(finished_at.to_string()),
        finished_at: finished_at.to_string(),
    }
}

#[test]
fn gate_assessment_caution_when_pipeline_passed() {
    let runs = vec![make_pipeline_run("test", "passed", &NOW.to_string())];
    let result = gate_assessment(&runs, "cleanup", NOW);
    assert_eq!(result["level"], "caution");
    assert_eq!(result["pipeline_caution_kinds"], json!(["test"]));
    assert!(result["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r.as_str().unwrap().contains("pipeline")));
    assert!(result["next_steps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s.as_str().unwrap().contains("without pipes")));
}

#[test]
fn gate_assessment_no_pipeline_caution_for_clean_commands() {
    let runs = vec![make_run("test", "passed", &NOW.to_string())];
    let result = gate_assessment(&runs, "cleanup", NOW);
    assert!(result["pipeline_caution_kinds"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn gate_assessment_pipeline_does_not_block() {
    let runs = vec![make_pipeline_run("test", "passed", &NOW.to_string())];
    let result = gate_assessment(&runs, "cleanup", NOW);
    assert_eq!(result["allowed"], true, "pipeline caution should not block");
}

#[test]
fn verification_status_layer_includes_trust_level() {
    let ts = NOW.to_string();
    let runs = vec![make_pipeline_run("test", "passed", &ts)];
    let result = verification_status_layer(&runs);
    let latest = result["latest_runs"].as_array().unwrap();
    assert_eq!(latest[0]["trust_level"], "caution");
    assert_eq!(latest[0]["exit_code_masked_possible"], true);
}

#[test]
fn verification_status_layer_trusted_for_clean_commands() {
    let ts = NOW.to_string();
    let runs = vec![make_run("test", "passed", &ts)];
    let result = verification_status_layer(&runs);
    let latest = result["latest_runs"].as_array().unwrap();
    assert_eq!(latest[0]["trust_level"], "trusted");
    assert_eq!(latest[0]["exit_code_masked_possible"], false);
}

#[test]
fn verification_status_layer_cautions_suspicious_pass_summary() {
    let ts = NOW.to_string();
    let mut run = make_run("test", "passed", &ts);
    run.summary = Some("src/App.vue(10,5): error TS2304: Cannot find name X".to_string());
    let runs = vec![run];
    let result = verification_status_layer(&runs);
    let latest = result["latest_runs"].as_array().unwrap();
    assert_eq!(latest[0]["trust_level"], "caution");
    assert!(!latest[0]["suspicious_pass_signals"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn gate_assessment_caution_when_suspicious_pass_summary() {
    let mut run = make_run("test", "passed", &NOW.to_string());
    run.summary = Some("FAILED keyword despite recorded passed status".to_string());
    let runs = vec![run];
    let result = gate_assessment(&runs, "cleanup", NOW);
    assert_eq!(result["level"], "caution");
    assert_eq!(result["suspicious_summary_kinds"], json!(["test"]));
    assert!(result["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r.as_str().unwrap().contains("suspicious pass")));
}

// ---- gate_blockers ----

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

#[test]
fn workspace_verification_evidence_layer_empty() {
    let result = workspace_verification_evidence_layer(&[], 0, 0);
    assert_eq!(result["status"], "available");
    assert_eq!(result["projects_with_recorded_verification"], 0);
    assert_eq!(result["projects_missing_verification"], 0);
    assert_eq!(result["confidence"], "low");
    assert!(result["blocking_projects"].as_array().unwrap().is_empty());
}

#[test]
fn workspace_verification_evidence_layer_single_project_all_passing() {
    let project = json!({
        "project_id": "proj-a",
        "verification_evidence": {
            "status": "available",
            "failing_runs": [],
        },
        "observation": {
            "freshness": {
                "verification": { "status": "fresh" }
            }
        },
        "safe_for_cleanup": true,
        "safe_for_refactor": true,
        "safe_for_cleanup_reason": "ok",
        "safe_for_refactor_reason": "ok",
    });
    let result = workspace_verification_evidence_layer(&[project], 1, 1);
    assert_eq!(result["projects_with_recorded_verification"], 1);
    assert_eq!(result["projects_missing_verification"], 0);
    assert_eq!(result["projects_with_failing_verification"], 0);
    assert_eq!(result["projects_safe_for_cleanup"], 1);
    assert_eq!(result["projects_safe_for_refactor"], 1);
    assert_eq!(result["confidence"], "high");
    assert!(result["blocking_projects"].as_array().unwrap().is_empty());
}

#[test]
fn workspace_verification_evidence_layer_mixed_projects() {
    let passing = json!({
        "project_id": "proj-good",
        "verification_evidence": {
            "status": "available",
            "failing_runs": [],
        },
        "observation": {
            "freshness": {
                "verification": { "status": "fresh" }
            }
        },
        "safe_for_cleanup": true,
        "safe_for_refactor": true,
        "safe_for_cleanup_reason": "ok",
        "safe_for_refactor_reason": "ok",
    });
    let failing = json!({
        "project_id": "proj-bad",
        "verification_evidence": {
            "status": "available",
            "failing_runs": [{"kind": "test", "status": "failed", "command": "make test"}],
        },
        "observation": {
            "freshness": {
                "verification": { "status": "fresh" }
            }
        },
        "safe_for_cleanup": false,
        "safe_for_refactor": false,
        "safe_for_cleanup_reason": "blocked",
        "safe_for_refactor_reason": "blocked",
    });
    let result = workspace_verification_evidence_layer(&[passing, failing], 2, 1);
    assert_eq!(result["projects_with_recorded_verification"], 2);
    assert_eq!(result["projects_with_failing_verification"], 1);
    assert_eq!(result["projects_safe_for_cleanup"], 1);
    assert_eq!(result["blocking_projects"].as_array().unwrap().len(), 1);
}

#[test]
fn workspace_verification_evidence_layer_missing_verification() {
    let project = json!({
        "project_id": "proj-new",
        "verification_evidence": {
            "status": "not_recorded",
            "failing_runs": [],
        },
        "observation": {
            "freshness": {
                "verification": { "status": "missing" }
            }
        },
        "safe_for_cleanup": false,
        "safe_for_refactor": false,
        "safe_for_cleanup_reason": "no evidence",
        "safe_for_refactor_reason": "no evidence",
    });
    let result = workspace_verification_evidence_layer(&[project], 1, 0);
    assert_eq!(result["projects_missing_verification"], 1);
    assert_eq!(result["confidence"], "low");
}

#[test]
fn workspace_verification_evidence_layer_has_gate_distribution() {
    let project = json!({
        "project_id": "p1",
        "verification_evidence": {
            "status": "available",
            "gate_assessment": {
                "cleanup": { "level": "allow" },
                "refactor": { "level": "allow" },
            },
            "failing_runs": [],
        },
        "observation": {
            "freshness": { "verification": { "status": "fresh" } }
        },
        "safe_for_cleanup": true,
        "safe_for_refactor": true,
        "safe_for_cleanup_reason": "ok",
        "safe_for_refactor_reason": "ok",
    });
    let result = workspace_verification_evidence_layer(&[project], 1, 1);
    assert_eq!(result["cleanup_gate_distribution"]["allow"], 1);
    assert_eq!(result["refactor_gate_distribution"]["allow"], 1);
}

#[test]
fn workspace_verification_evidence_layer_direct_observations_count() {
    let result = workspace_verification_evidence_layer(&[], 5, 2);
    let obs = result["direct_observations"].as_array().unwrap();
    assert!(obs
        .iter()
        .any(|o| o.as_str().unwrap().contains("Registered projects: 5")));
    assert!(obs
        .iter()
        .any(|o| o.as_str().unwrap().contains("monitoring: 2")));
}

#[test]
fn workspace_verification_evidence_layer_verified_conclusions() {
    let project = json!({
        "project_id": "p1",
        "verification_evidence": {
            "status": "available",
            "failing_runs": [],
        },
        "observation": {
            "freshness": { "verification": { "status": "fresh" } }
        },
        "safe_for_cleanup": true,
        "safe_for_refactor": true,
        "safe_for_cleanup_reason": "ok",
        "safe_for_refactor_reason": "ok",
    });
    let result = workspace_verification_evidence_layer(&[project], 1, 1);
    let vc = result["verified_conclusions"].as_array().unwrap();
    assert!(vc
        .iter()
        .any(|c| c["summary"].as_str().unwrap().contains("1 project(s)")));
}

#[test]
fn workspace_verification_evidence_layer_unverified_conclusions() {
    let project = json!({
        "project_id": "p1",
        "verification_evidence": {
            "status": "not_recorded",
            "failing_runs": [],
        },
        "observation": {
            "freshness": { "verification": { "status": "missing" } }
        },
        "safe_for_cleanup": false,
        "safe_for_refactor": false,
        "safe_for_cleanup_reason": "no evidence",
        "safe_for_refactor_reason": "no evidence",
    });
    let result = workspace_verification_evidence_layer(&[project], 1, 0);
    let uc = result["unverified_conclusions"].as_array().unwrap();
    assert!(uc.iter().any(|c| c["summary"]
        .as_str()
        .unwrap()
        .contains("missing verification")));
}
