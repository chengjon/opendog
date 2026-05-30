use super::*;

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
