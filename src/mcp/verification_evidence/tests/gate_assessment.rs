use super::*;

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
