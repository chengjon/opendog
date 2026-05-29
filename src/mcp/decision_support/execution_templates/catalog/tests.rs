use super::*;

fn call(action: &str) -> Vec<Value> {
    base_templates(
        action,
        "test-project",
        "available",
        "low",
        false,
        false,
        &json!([]),
    )
}

fn template_ids(templates: &[Value]) -> Vec<&str> {
    templates
        .iter()
        .map(|t| t["template_id"].as_str().unwrap())
        .collect()
}

// ── action → expected template IDs ──────────────────────────────

#[test]
fn review_failing_verification_templates() {
    let t = call("review_failing_verification");
    assert_eq!(
        template_ids(&t),
        vec!["verification.review_status", "verification.rerun"]
    );
}

#[test]
fn stabilize_repository_templates() {
    let t = call("stabilize_repository_state");
    assert_eq!(template_ids(&t), vec!["repo.status", "repo.diff"]);
}

#[test]
fn start_monitor_templates() {
    let t = call("start_monitor");
    assert_eq!(template_ids(&t), vec!["monitor.start", "snapshot.baseline"]);
}

#[test]
fn take_snapshot_templates() {
    let t = call("take_snapshot");
    assert_eq!(template_ids(&t), vec!["snapshot.take", "stats.inspect"]);
}

#[test]
fn generate_activity_then_stats_templates() {
    let t = call("generate_activity_then_stats");
    assert_eq!(template_ids(&t), vec!["activity.generate", "stats.refresh"]);
}

#[test]
fn run_verification_before_high_risk_templates() {
    let t = call("run_verification_before_high_risk_changes");
    assert_eq!(
        template_ids(&t),
        vec!["verification.status", "verification.execute"]
    );
}

#[test]
fn review_unused_files_templates() {
    let t = call("review_unused_files");
    assert_eq!(template_ids(&t), vec!["unused.list", "unused.search"]);
}

#[test]
fn inspect_hot_files_templates() {
    let t = call("inspect_hot_files");
    assert_eq!(template_ids(&t), vec!["stats.hot_files", "hot.diff"]);
}

#[test]
fn unknown_action_returns_guidance_refresh() {
    let t = call("unknown_action");
    assert_eq!(template_ids(&t), vec!["guidance.refresh"]);
}

// ── template structure invariants ───────────────────────────────

#[test]
fn every_template_has_required_fields() {
    let all_actions = vec![
        "review_failing_verification",
        "stabilize_repository_state",
        "start_monitor",
        "take_snapshot",
        "generate_activity_then_stats",
        "run_verification_before_high_risk_changes",
        "review_unused_files",
        "inspect_hot_files",
        "fallback_action",
    ];
    for action in all_actions {
        let templates = call(action);
        for t in &templates {
            assert!(
                t["template_id"].is_string(),
                "missing template_id for action {}",
                action
            );
            assert!(
                t["preconditions"].is_array(),
                "missing preconditions for action {}",
                action
            );
            assert!(
                t["blocking_conditions"].is_array(),
                "missing blocking_conditions for action {}",
                action
            );
            assert!(
                t["success_signal"].is_string(),
                "missing success_signal for action {}",
                action
            );
            assert!(
                t["parameter_schema"].is_object(),
                "missing parameter_schema for action {}",
                action
            );
            assert!(
                t["default_values"].is_object(),
                "missing default_values for action {}",
                action
            );
        }
    }
}

// ── project_id propagation ──────────────────────────────────────

#[test]
fn project_id_appears_in_args_template() {
    let templates = base_templates(
        "start_monitor",
        "my-proj",
        "available",
        "low",
        false,
        false,
        &json!([]),
    );
    for t in &templates {
        let args = &t["args_template"];
        if args.is_object() && args["id"].is_string() {
            assert_eq!(args["id"], "my-proj");
        }
    }
}

// ── cleanup_ready / refactor_ready blocking conditions ──────────

#[test]
fn review_unused_files_blocked_when_cleanup_not_ready() {
    let templates = base_templates(
        "review_unused_files",
        "p",
        "available",
        "low",
        false,
        false,
        &json!([]),
    );
    for t in &templates {
        let bc = t["blocking_conditions"].as_array().unwrap();
        assert!(
            !bc.is_empty(),
            "expected blocking_conditions when cleanup_ready=false"
        );
    }
}

#[test]
fn review_unused_files_unblocked_when_cleanup_ready() {
    let templates = base_templates(
        "review_unused_files",
        "p",
        "available",
        "low",
        true,
        false,
        &json!([]),
    );
    for t in &templates {
        let bc = t["blocking_conditions"].as_array().unwrap();
        assert!(
            bc.is_empty(),
            "expected empty blocking_conditions when cleanup_ready=true"
        );
    }
}

#[test]
fn inspect_hot_files_blocked_when_refactor_not_ready() {
    let templates = base_templates(
        "inspect_hot_files",
        "p",
        "available",
        "low",
        false,
        false,
        &json!([]),
    );
    let stats_t = &templates[0];
    let bc = stats_t["blocking_conditions"].as_array().unwrap();
    assert!(!bc.is_empty());
}

#[test]
fn inspect_hot_files_unblocked_when_refactor_ready() {
    let templates = base_templates(
        "inspect_hot_files",
        "p",
        "available",
        "low",
        false,
        true,
        &json!([]),
    );
    let stats_t = &templates[0];
    let bc = stats_t["blocking_conditions"].as_array().unwrap();
    assert!(bc.is_empty());
}

// ── repo_risk_level gating on hot.diff ──────────────────────────

#[test]
fn hot_diff_blocked_when_repo_risk_high() {
    let templates = base_templates(
        "inspect_hot_files",
        "p",
        "available",
        "high",
        false,
        false,
        &json!([]),
    );
    let diff_t = &templates[1];
    assert_eq!(diff_t["template_id"], "hot.diff");
    let bc = diff_t["blocking_conditions"].as_array().unwrap();
    assert!(!bc.is_empty());
}

#[test]
fn hot_diff_unblocked_when_repo_risk_low() {
    let templates = base_templates(
        "inspect_hot_files",
        "p",
        "available",
        "low",
        false,
        false,
        &json!([]),
    );
    let diff_t = &templates[1];
    assert_eq!(diff_t["template_id"], "hot.diff");
    let bc = diff_t["blocking_conditions"].as_array().unwrap();
    assert!(bc.is_empty());
}

// ── fallback success_signal embeds verification_status ──────────

#[test]
fn fallback_success_signal_includes_verification_status() {
    let templates = base_templates("whatever", "p", "missing", "low", false, false, &json!([]));
    let sig = templates[0]["success_signal"].as_str().unwrap();
    assert!(
        sig.contains("missing"),
        "expected verification status in success_signal"
    );
}
