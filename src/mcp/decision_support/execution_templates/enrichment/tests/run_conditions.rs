use super::*;

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
