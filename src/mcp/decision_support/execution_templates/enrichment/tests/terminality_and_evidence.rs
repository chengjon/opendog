use super::*;

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
