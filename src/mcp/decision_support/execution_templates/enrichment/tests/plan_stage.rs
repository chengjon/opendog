use super::*;

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
