use super::*;

#[test]
fn parallel_for_inspect_templates() {
    for id in &[
        "repo.status",
        "repo.diff",
        "activity.generate",
        "unused.search",
        "hot.diff",
        "stats.inspect",
        "stats.refresh",
        "stats.hot_files",
        "unused.list",
    ] {
        let result = enrich_templates("some_action", vec![make_template(id)], false, false);
        assert_eq!(
            result.as_array().unwrap()[0]["can_run_in_parallel"],
            true,
            "expected true for {}",
            id
        );
    }
}

#[test]
fn not_parallel_for_verification_templates() {
    for id in &[
        "verification.review_status",
        "verification.rerun",
        "monitor.start",
        "snapshot.baseline",
        "guidance.refresh",
    ] {
        let result = enrich_templates("some_action", vec![make_template(id)], false, false);
        assert_eq!(
            result.as_array().unwrap()[0]["can_run_in_parallel"],
            false,
            "expected false for {}",
            id
        );
    }
}

// ── retry_policy structure ───────────────────────────────────────

#[test]
fn retry_policy_has_allowed_field() {
    let templates = vec![make_template("repo.status")];
    let result = enrich_templates("stabilize_repository_state", templates, false, false);
    let retry = &result.as_array().unwrap()[0]["retry_policy"];
    assert!(retry["allowed"].is_boolean());
    assert!(retry["max_attempts"].is_number());
    assert!(retry["strategy"].is_string());
    assert!(retry["retry_when"].is_array());
}

#[test]
fn unknown_template_has_no_retry_allowed() {
    let result = enrich_templates(
        "some_action",
        vec![make_template("totally.unknown")],
        false,
        false,
    );
    let retry = &result.as_array().unwrap()[0]["retry_policy"];
    assert_eq!(retry["allowed"], false);
    assert_eq!(retry["max_attempts"], 1);
}

// ── expected_output_fields ───────────────────────────────────────

#[test]
fn expected_output_fields_for_verification_review_status() {
    let result = enrich_templates(
        "review_failing_verification",
        vec![make_template("verification.review_status")],
        false,
        false,
    );
    let fields = result.as_array().unwrap()[0]["expected_output_fields"]
        .as_array()
        .unwrap();
    assert!(fields.contains(&json!("verification.latest_runs")));
    assert!(fields.contains(&json!("verification.failing_runs")));
}

#[test]
fn expected_output_fields_for_guidance_refresh() {
    let result = enrich_templates(
        "unknown",
        vec![make_template("guidance.refresh")],
        false,
        false,
    );
    let fields = result.as_array().unwrap()[0]["expected_output_fields"]
        .as_array()
        .unwrap();
    assert!(fields.contains(&json!("guidance.recommended_flow")));
    assert!(fields
        .iter()
        .any(|f| f.as_str().unwrap().contains("execution_strategy")));
}

#[test]
fn expected_output_fields_empty_for_unknown_template() {
    let result = enrich_templates(
        "unknown",
        vec![make_template("nonexistent.id")],
        false,
        false,
    );
    let fields = result.as_array().unwrap()[0]["expected_output_fields"]
        .as_array()
        .unwrap();
    assert!(fields.is_empty());
}

// ── follow_up_on_success / follow_up_on_failure ──────────────────
