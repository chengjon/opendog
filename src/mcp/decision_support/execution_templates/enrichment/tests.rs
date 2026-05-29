use super::*;

fn make_template(id: &str) -> Value {
    json!({ "template_id": id })
}

// ── empty input returns empty array ──────────────────────────────

#[test]
fn empty_templates_returns_empty_array() {
    let result = enrich_templates("start_monitor", vec![], false, false);
    assert!(result.as_array().unwrap().is_empty());
}

// ── priority is 1-indexed per template ───────────────────────────

#[test]
fn priority_is_sequential() {
    let templates = vec![make_template("repo.status"), make_template("repo.diff")];
    let result = enrich_templates("stabilize_repository_state", templates, false, false);
    let arr = result.as_array().unwrap();
    assert_eq!(arr[0]["priority"], 1);
    assert_eq!(arr[1]["priority"], 2);
}

// ── should_run_if / skip_if per action ───────────────────────────

#[test]
fn review_failing_verification_run_if() {
    let result = enrich_templates(
        "review_failing_verification",
        vec![make_template("x")],
        false,
        false,
    );
    let t = &result.as_array().unwrap()[0];
    let run = t["should_run_if"].as_array().unwrap();
    assert!(run.iter().any(|r| r.as_str().unwrap().contains("failing")));
    assert!(t["skip_if"].as_array().unwrap().is_empty());
}

#[test]
fn stabilize_repository_run_if_and_skip_if() {
    let result = enrich_templates(
        "stabilize_repository_state",
        vec![make_template("y")],
        false,
        false,
    );
    let t = &result.as_array().unwrap()[0];
    let run = t["should_run_if"].as_array().unwrap();
    assert!(run
        .iter()
        .any(|r| r.as_str().unwrap().contains("repository operation state")));
    let skip = t["skip_if"].as_array().unwrap();
    assert!(!skip.is_empty());
}

#[test]
fn start_monitor_run_if_and_skip_if() {
    let result = enrich_templates("start_monitor", vec![make_template("z")], false, false);
    let t = &result.as_array().unwrap()[0];
    assert!(!t["should_run_if"].as_array().unwrap().is_empty());
    assert!(!t["skip_if"].as_array().unwrap().is_empty());
}

#[test]
fn review_unused_files_skip_if_depends_on_cleanup_ready() {
    // cleanup NOT ready: skip_if should have an entry
    let result = enrich_templates(
        "review_unused_files",
        vec![make_template("a")],
        false,
        false,
    );
    let skip = result.as_array().unwrap()[0]["skip_if"].as_array().unwrap();
    assert!(!skip.is_empty());

    // cleanup ready: skip_if should be empty
    let result2 = enrich_templates("review_unused_files", vec![make_template("a")], true, false);
    let skip2 = result2.as_array().unwrap()[0]["skip_if"]
        .as_array()
        .unwrap();
    assert!(skip2.is_empty());
}

#[test]
fn inspect_hot_files_skip_if_depends_on_refactor_ready() {
    // refactor NOT ready
    let result = enrich_templates("inspect_hot_files", vec![make_template("b")], false, false);
    let skip = result.as_array().unwrap()[0]["skip_if"].as_array().unwrap();
    assert!(!skip.is_empty());

    // refactor ready
    let result2 = enrich_templates("inspect_hot_files", vec![make_template("b")], false, true);
    let skip2 = result2.as_array().unwrap()[0]["skip_if"]
        .as_array()
        .unwrap();
    assert!(skip2.is_empty());
}

#[test]
fn unknown_action_has_generic_run_if() {
    let result = enrich_templates("unknown", vec![make_template("c")], false, false);
    let t = &result.as_array().unwrap()[0];
    let run = t["should_run_if"].as_array().unwrap();
    assert!(run
        .iter()
        .any(|r| r.as_str().unwrap().contains("no narrower")));
    assert!(t["skip_if"].as_array().unwrap().is_empty());
}

// ── plan_stage per template_id ───────────────────────────────────

#[test]
fn plan_stage_for_verification_templates() {
    for id in &["verification.review_status", "verification.status"] {
        let result = enrich_templates(
            "review_failing_verification",
            vec![make_template(id)],
            false,
            false,
        );
        assert_eq!(result.as_array().unwrap()[0]["plan_stage"], "verify");
    }
}

#[test]
fn plan_stage_for_repo_and_activity_templates() {
    for id in &[
        "repo.status",
        "repo.diff",
        "activity.generate",
        "unused.search",
        "hot.diff",
    ] {
        let result = enrich_templates(
            "stabilize_repository_state",
            vec![make_template(id)],
            false,
            false,
        );
        assert_eq!(
            result.as_array().unwrap()[0]["plan_stage"],
            "inspect",
            "failed for template_id {}",
            id
        );
    }
}

#[test]
fn plan_stage_for_monitor_start() {
    let result = enrich_templates(
        "start_monitor",
        vec![make_template("monitor.start")],
        false,
        false,
    );
    assert_eq!(result.as_array().unwrap()[0]["plan_stage"], "observe");
}

#[test]
fn plan_stage_for_snapshot_templates() {
    for id in &["snapshot.baseline", "snapshot.take"] {
        let result = enrich_templates("take_snapshot", vec![make_template(id)], false, false);
        assert_eq!(result.as_array().unwrap()[0]["plan_stage"], "observe");
    }
}

#[test]
fn plan_stage_for_stats_templates() {
    for id in &[
        "stats.inspect",
        "stats.refresh",
        "stats.hot_files",
        "unused.list",
    ] {
        let result = enrich_templates("take_snapshot", vec![make_template(id)], false, false);
        assert_eq!(
            result.as_array().unwrap()[0]["plan_stage"],
            "analyze",
            "failed for template_id {}",
            id
        );
    }
}

#[test]
fn plan_stage_for_guidance_refresh() {
    let result = enrich_templates(
        "unknown",
        vec![make_template("guidance.refresh")],
        false,
        false,
    );
    assert_eq!(result.as_array().unwrap()[0]["plan_stage"], "decide");
}

#[test]
fn plan_stage_for_unknown_template_defaults_to_inspect() {
    let result = enrich_templates(
        "unknown",
        vec![make_template("some.unknown.id")],
        false,
        false,
    );
    assert_eq!(result.as_array().unwrap()[0]["plan_stage"], "inspect");
}

// ── terminality ─────────────────────────────────────────────────

#[test]
fn terminality_decision_gate_for_verification_review() {
    let result = enrich_templates(
        "review_failing_verification",
        vec![make_template("verification.review_status")],
        false,
        false,
    );
    assert_eq!(
        result.as_array().unwrap()[0]["terminality"],
        "decision_gate"
    );
}

#[test]
fn terminality_terminal_on_success_for_guidance_refresh() {
    let result = enrich_templates(
        "unknown",
        vec![make_template("guidance.refresh")],
        false,
        false,
    );
    assert_eq!(
        result.as_array().unwrap()[0]["terminality"],
        "terminal_on_success"
    );
}

#[test]
fn terminality_non_terminal_for_most_templates() {
    for id in &[
        "repo.status",
        "monitor.start",
        "snapshot.baseline",
        "stats.inspect",
        "verification.rerun",
    ] {
        let result = enrich_templates("some_action", vec![make_template(id)], false, false);
        assert_eq!(
            result.as_array().unwrap()[0]["terminality"],
            "non_terminal",
            "failed for {}",
            id
        );
    }
}

// ── requires_human_confirmation ──────────────────────────────────

#[test]
fn human_confirmation_for_verification_rerun() {
    for id in &["verification.rerun", "verification.execute"] {
        let result = enrich_templates("some_action", vec![make_template(id)], false, false);
        assert_eq!(
            result.as_array().unwrap()[0]["requires_human_confirmation"],
            true,
            "expected true for {}",
            id
        );
    }
}

#[test]
fn no_human_confirmation_for_read_only_templates() {
    for id in &[
        "repo.status",
        "stats.inspect",
        "monitor.start",
        "guidance.refresh",
    ] {
        let result = enrich_templates("some_action", vec![make_template(id)], false, false);
        assert_eq!(
            result.as_array().unwrap()[0]["requires_human_confirmation"],
            false,
            "expected false for {}",
            id
        );
    }
}

// ── evidence_written_to_opendog ─────────────────────────────────

#[test]
fn evidence_written_for_verification_rerun_and_snapshot() {
    for id in &[
        "verification.rerun",
        "snapshot.baseline",
        "snapshot.take",
        "verification.execute",
    ] {
        let result = enrich_templates("some_action", vec![make_template(id)], false, false);
        assert_eq!(
            result.as_array().unwrap()[0]["evidence_written_to_opendog"],
            true,
            "expected true for {}",
            id
        );
    }
}

#[test]
fn no_evidence_written_for_read_templates() {
    for id in &[
        "repo.status",
        "repo.diff",
        "stats.inspect",
        "guidance.refresh",
        "monitor.start",
    ] {
        let result = enrich_templates("some_action", vec![make_template(id)], false, false);
        assert_eq!(
            result.as_array().unwrap()[0]["evidence_written_to_opendog"],
            false,
            "expected false for {}",
            id
        );
    }
}

// ── can_run_in_parallel ──────────────────────────────────────────

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

#[test]
fn follow_ups_for_verification_rerun() {
    let result = enrich_templates(
        "review_failing_verification",
        vec![make_template("verification.rerun")],
        false,
        false,
    );
    let t = &result.as_array().unwrap()[0];
    let success = t["follow_up_on_success"].as_array().unwrap();
    let failure = t["follow_up_on_failure"].as_array().unwrap();
    assert!(success.contains(&json!("verification.review_status")));
    assert!(success.contains(&json!("unused.list")));
    assert!(failure.contains(&json!("verification.review_status")));
    assert!(failure.contains(&json!("repo.diff")));
}

#[test]
fn guidance_refresh_has_no_follow_ups() {
    let result = enrich_templates(
        "unknown",
        vec![make_template("guidance.refresh")],
        false,
        false,
    );
    let t = &result.as_array().unwrap()[0];
    assert!(t["follow_up_on_success"].as_array().unwrap().is_empty());
    assert!(t["follow_up_on_failure"].as_array().unwrap().is_empty());
}

// ── unused.search follow_up varies by cleanup_ready ──────────────

#[test]
fn unused_search_follow_up_varies_by_cleanup_ready() {
    // cleanup NOT ready: follow_up_on_success includes verification.status
    let result = enrich_templates(
        "review_unused_files",
        vec![make_template("unused.search")],
        false,
        false,
    );
    let success = result.as_array().unwrap()[0]["follow_up_on_success"]
        .as_array()
        .unwrap();
    assert!(success.contains(&json!("verification.status")));

    // cleanup ready: follow_up_on_success includes guidance.refresh
    let result2 = enrich_templates(
        "review_unused_files",
        vec![make_template("unused.search")],
        true,
        false,
    );
    let success2 = result2.as_array().unwrap()[0]["follow_up_on_success"]
        .as_array()
        .unwrap();
    assert!(success2.contains(&json!("guidance.refresh")));
}

// ── multiple templates processed independently ───────────────────

#[test]
fn multiple_templates_each_get_own_priority() {
    let templates = vec![
        make_template("repo.status"),
        make_template("repo.diff"),
        make_template("guidance.refresh"),
    ];
    let result = enrich_templates("stabilize_repository_state", templates, false, false);
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0]["priority"], 1);
    assert_eq!(arr[1]["priority"], 2);
    assert_eq!(arr[2]["priority"], 3);
}
