use serde_json::{json, Value};

pub(super) fn enrich_templates(
    action: &str,
    templates: Vec<Value>,
    cleanup_ready: bool,
    refactor_ready: bool,
) -> Value {
    let templates = templates
        .into_iter()
        .enumerate()
        .map(|(index, mut template)| {
            let (priority, should_run_if, skip_if) = match action {
                "review_failing_verification" => (
                    index + 1,
                    vec!["run when verification evidence is failing or uncertain".to_string()],
                    vec![],
                ),
                "stabilize_repository_state" => (
                    index + 1,
                    vec!["run when repository operation state is not clean".to_string()],
                    vec!["skip once merge/rebase/cherry-pick/bisect state is resolved".to_string()],
                ),
                "start_monitor" => (
                    index + 1,
                    vec!["run when monitoring is not active".to_string()],
                    vec!["skip if project is already monitoring".to_string()],
                ),
                "take_snapshot" => (
                    index + 1,
                    vec!["run when snapshot baseline is missing or stale".to_string()],
                    vec!["skip if a fresh snapshot baseline already exists".to_string()],
                ),
                "generate_activity_then_stats" => (
                    index + 1,
                    vec!["run when accessed files are still zero".to_string()],
                    vec!["skip if activity-derived file usage already exists".to_string()],
                ),
                "run_verification_before_high_risk_changes" => (
                    index + 1,
                    vec!["run when verification evidence is missing before risky work".to_string()],
                    vec![
                        "skip once test/lint/build evidence is already recorded and trusted"
                            .to_string(),
                    ],
                ),
                "review_unused_files" => (
                    index + 1,
                    vec!["run when unused-file candidates exist".to_string()],
                    if cleanup_ready {
                        vec![]
                    } else {
                        vec![
                            "skip destructive follow-up while cleanup blockers still exist"
                                .to_string(),
                        ]
                    },
                ),
                "inspect_hot_files" => (
                    index + 1,
                    vec!["run when activity hotspots should guide targeted review".to_string()],
                    if refactor_ready {
                        vec![]
                    } else {
                        vec![
                            "skip broad refactor follow-up while refactor blockers still exist"
                                .to_string(),
                        ]
                    },
                ),
                _ => (
                    index + 1,
                    vec!["run when no narrower action-specific plan is available".to_string()],
                    vec![],
                ),
            };

            template["priority"] = json!(priority);
            template["should_run_if"] = json!(should_run_if);
            template["skip_if"] = json!(skip_if);
            let template_id = template["template_id"].as_str().unwrap_or("");
            let (
                plan_stage,
                terminality,
                can_run_in_parallel,
                requires_human_confirmation,
                evidence_written_to_opendog,
                retry_policy,
            ) = match template_id {
                "verification.review_status" | "verification.status" => (
                    "verify",
                    "decision_gate",
                    false,
                    false,
                    false,
                    json!({
                        "allowed": true,
                        "max_attempts": 2,
                        "strategy": "refresh_after_new_evidence",
                        "retry_when": ["verification payload is temporarily unavailable"]
                    }),
                ),
                "verification.rerun" | "verification.execute" => (
                    "verify",
                    "non_terminal",
                    false,
                    true,
                    true,
                    json!({
                        "allowed": true,
                        "max_attempts": 2,
                        "strategy": "rerun_once_after_fix_or_command_adjustment",
                        "retry_when": ["verification command was corrected", "environment issue was resolved"]
                    }),
                ),
                "repo.status" | "repo.diff" | "activity.generate" | "unused.search" | "hot.diff" => (
                    "inspect",
                    "non_terminal",
                    true,
                    false,
                    false,
                    json!({
                        "allowed": true,
                        "max_attempts": 3,
                        "strategy": "rerun_after_workspace_changes",
                        "retry_when": ["repository state changed", "search target changed"]
                    }),
                ),
                "monitor.start" => (
                    "observe",
                    "non_terminal",
                    false,
                    false,
                    false,
                    json!({
                        "allowed": true,
                        "max_attempts": 2,
                        "strategy": "retry_after_process_state_check",
                        "retry_when": ["monitor was not already running", "project registration is intact"]
                    }),
                ),
                "snapshot.baseline" | "snapshot.take" => (
                    "observe",
                    "non_terminal",
                    false,
                    false,
                    true,
                    json!({
                        "allowed": true,
                        "max_attempts": 2,
                        "strategy": "retry_after_filesystem_settles",
                        "retry_when": ["snapshot failed due to transient filesystem state"]
                    }),
                ),
                "stats.inspect" | "stats.refresh" | "stats.hot_files" | "unused.list" => (
                    "analyze",
                    "non_terminal",
                    true,
                    false,
                    false,
                    json!({
                        "allowed": true,
                        "max_attempts": 2,
                        "strategy": "rerun_after_new_activity_or_snapshot",
                        "retry_when": ["new activity data was observed", "snapshot baseline changed"]
                    }),
                ),
                "guidance.refresh" => (
                    "decide",
                    "terminal_on_success",
                    false,
                    false,
                    false,
                    json!({
                        "allowed": true,
                        "max_attempts": 2,
                        "strategy": "refresh_after_preconditions_change",
                        "retry_when": ["upstream evidence changed", "target project changed"]
                    }),
                ),
                _ => (
                    "inspect",
                    "non_terminal",
                    false,
                    false,
                    false,
                    json!({
                        "allowed": false,
                        "max_attempts": 1,
                        "strategy": "manual_review_required",
                        "retry_when": []
                    }),
                ),
            };
            let (expected_output_fields, follow_up_on_success, follow_up_on_failure) =
                match template_id {
                    "verification.review_status" => (
                        vec!["verification.latest_runs", "verification.failing_runs"],
                        vec!["verification.rerun"],
                        vec!["verification.rerun"],
                    ),
                    "verification.rerun" => (
                        vec!["executed.run.status", "executed.run.exit_code"],
                        vec!["verification.review_status", "unused.list"],
                        vec!["verification.review_status", "repo.diff"],
                    ),
                    "repo.status" => (vec!["working_tree_state"], vec!["repo.diff"], vec![]),
                    "repo.diff" => (vec!["diff_summary"], vec!["guidance.refresh"], vec![]),
                    "monitor.start" => (
                        vec!["status", "already_running", "snapshot_taken"],
                        vec!["snapshot.baseline", "stats.inspect"],
                        vec!["guidance.refresh"],
                    ),
                    "snapshot.baseline" | "snapshot.take" => (
                        vec!["total_files", "new_files", "removed_files"],
                        vec!["stats.inspect", "stats.refresh"],
                        vec!["guidance.refresh"],
                    ),
                    "stats.inspect" | "stats.refresh" | "stats.hot_files" => (
                        vec!["summary.total_files", "summary.accessed", "files"],
                        vec!["unused.list", "hot.diff"],
                        vec!["guidance.refresh"],
                    ),
                    "activity.generate" => (
                        vec!["repository_interaction"],
                        vec!["stats.refresh"],
                        vec!["guidance.refresh"],
                    ),
                    "verification.status" => (
                        vec!["verification.latest_runs", "verification.missing_kinds"],
                        vec!["verification.execute"],
                        vec!["guidance.refresh"],
                    ),
                    "verification.execute" => (
                        vec!["executed.run.status", "executed.run.summary"],
                        vec!["verification.status", "stats.hot_files"],
                        vec!["verification.status", "repo.diff"],
                    ),
                    "unused.list" => (
                        vec!["unused_count", "files"],
                        vec!["unused.search"],
                        vec!["guidance.refresh"],
                    ),
                    "unused.search" => (
                        vec!["search_matches"],
                        if cleanup_ready {
                            vec!["guidance.refresh"]
                        } else {
                            vec!["verification.status"]
                        },
                        vec!["guidance.refresh"],
                    ),
                    "hot.diff" => (
                        vec!["diff_summary"],
                        vec!["guidance.refresh"],
                        vec!["guidance.refresh"],
                    ),
                    "guidance.refresh" => (
                        vec![
                            "guidance.recommended_flow",
                            "guidance.layers.execution_strategy",
                        ],
                        vec![],
                        vec![],
                    ),
                    _ => (vec![], vec![], vec![]),
                };
            template["plan_stage"] = json!(plan_stage);
            template["terminality"] = json!(terminality);
            template["can_run_in_parallel"] = json!(can_run_in_parallel);
            template["requires_human_confirmation"] = json!(requires_human_confirmation);
            template["evidence_written_to_opendog"] = json!(evidence_written_to_opendog);
            template["retry_policy"] = retry_policy;
            template["expected_output_fields"] = json!(expected_output_fields);
            template["follow_up_on_success"] = json!(follow_up_on_success);
            template["follow_up_on_failure"] = json!(follow_up_on_failure);
            template
        })
        .collect::<Vec<_>>();

    json!(templates)
}

#[cfg(test)]
mod tests {
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
        assert!(run.iter().any(|r| r.as_str().unwrap().contains("repository operation state")));
        let skip = t["skip_if"].as_array().unwrap();
        assert!(!skip.is_empty());
    }

    #[test]
    fn start_monitor_run_if_and_skip_if() {
        let result = enrich_templates(
            "start_monitor",
            vec![make_template("z")],
            false,
            false,
        );
        let t = &result.as_array().unwrap()[0];
        assert!(t["should_run_if"].as_array().unwrap().len() > 0);
        assert!(t["skip_if"].as_array().unwrap().len() > 0);
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
        let result2 = enrich_templates(
            "review_unused_files",
            vec![make_template("a")],
            true,
            false,
        );
        let skip2 = result2.as_array().unwrap()[0]["skip_if"].as_array().unwrap();
        assert!(skip2.is_empty());
    }

    #[test]
    fn inspect_hot_files_skip_if_depends_on_refactor_ready() {
        // refactor NOT ready
        let result = enrich_templates(
            "inspect_hot_files",
            vec![make_template("b")],
            false,
            false,
        );
        let skip = result.as_array().unwrap()[0]["skip_if"].as_array().unwrap();
        assert!(!skip.is_empty());

        // refactor ready
        let result2 = enrich_templates(
            "inspect_hot_files",
            vec![make_template("b")],
            false,
            true,
        );
        let skip2 = result2.as_array().unwrap()[0]["skip_if"].as_array().unwrap();
        assert!(skip2.is_empty());
    }

    #[test]
    fn unknown_action_has_generic_run_if() {
        let result = enrich_templates("unknown", vec![make_template("c")], false, false);
        let t = &result.as_array().unwrap()[0];
        let run = t["should_run_if"].as_array().unwrap();
        assert!(run.iter().any(|r| r.as_str().unwrap().contains("no narrower")));
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
        for id in &["repo.status", "repo.diff", "activity.generate", "unused.search", "hot.diff"] {
            let result = enrich_templates("stabilize_repository_state", vec![make_template(id)], false, false);
            assert_eq!(result.as_array().unwrap()[0]["plan_stage"], "inspect", "failed for template_id {}", id);
        }
    }

    #[test]
    fn plan_stage_for_monitor_start() {
        let result = enrich_templates("start_monitor", vec![make_template("monitor.start")], false, false);
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
        for id in &["stats.inspect", "stats.refresh", "stats.hot_files", "unused.list"] {
            let result = enrich_templates("take_snapshot", vec![make_template(id)], false, false);
            assert_eq!(result.as_array().unwrap()[0]["plan_stage"], "analyze", "failed for template_id {}", id);
        }
    }

    #[test]
    fn plan_stage_for_guidance_refresh() {
        let result = enrich_templates("unknown", vec![make_template("guidance.refresh")], false, false);
        assert_eq!(result.as_array().unwrap()[0]["plan_stage"], "decide");
    }

    #[test]
    fn plan_stage_for_unknown_template_defaults_to_inspect() {
        let result = enrich_templates("unknown", vec![make_template("some.unknown.id")], false, false);
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
        assert_eq!(result.as_array().unwrap()[0]["terminality"], "decision_gate");
    }

    #[test]
    fn terminality_terminal_on_success_for_guidance_refresh() {
        let result = enrich_templates("unknown", vec![make_template("guidance.refresh")], false, false);
        assert_eq!(result.as_array().unwrap()[0]["terminality"], "terminal_on_success");
    }

    #[test]
    fn terminality_non_terminal_for_most_templates() {
        for id in &["repo.status", "monitor.start", "snapshot.baseline", "stats.inspect", "verification.rerun"] {
            let result = enrich_templates("some_action", vec![make_template(id)], false, false);
            assert_eq!(result.as_array().unwrap()[0]["terminality"], "non_terminal", "failed for {}", id);
        }
    }

    // ── requires_human_confirmation ──────────────────────────────────

    #[test]
    fn human_confirmation_for_verification_rerun() {
        for id in &["verification.rerun", "verification.execute"] {
            let result = enrich_templates("some_action", vec![make_template(id)], false, false);
            assert_eq!(result.as_array().unwrap()[0]["requires_human_confirmation"], true, "expected true for {}", id);
        }
    }

    #[test]
    fn no_human_confirmation_for_read_only_templates() {
        for id in &["repo.status", "stats.inspect", "monitor.start", "guidance.refresh"] {
            let result = enrich_templates("some_action", vec![make_template(id)], false, false);
            assert_eq!(result.as_array().unwrap()[0]["requires_human_confirmation"], false, "expected false for {}", id);
        }
    }

    // ── evidence_written_to_opendog ─────────────────────────────────

    #[test]
    fn evidence_written_for_verification_rerun_and_snapshot() {
        for id in &["verification.rerun", "snapshot.baseline", "snapshot.take", "verification.execute"] {
            let result = enrich_templates("some_action", vec![make_template(id)], false, false);
            assert_eq!(result.as_array().unwrap()[0]["evidence_written_to_opendog"], true, "expected true for {}", id);
        }
    }

    #[test]
    fn no_evidence_written_for_read_templates() {
        for id in &["repo.status", "repo.diff", "stats.inspect", "guidance.refresh", "monitor.start"] {
            let result = enrich_templates("some_action", vec![make_template(id)], false, false);
            assert_eq!(result.as_array().unwrap()[0]["evidence_written_to_opendog"], false, "expected false for {}", id);
        }
    }

    // ── can_run_in_parallel ──────────────────────────────────────────

    #[test]
    fn parallel_for_inspect_templates() {
        for id in &["repo.status", "repo.diff", "activity.generate", "unused.search", "hot.diff", "stats.inspect", "stats.refresh", "stats.hot_files", "unused.list"] {
            let result = enrich_templates("some_action", vec![make_template(id)], false, false);
            assert_eq!(result.as_array().unwrap()[0]["can_run_in_parallel"], true, "expected true for {}", id);
        }
    }

    #[test]
    fn not_parallel_for_verification_templates() {
        for id in &["verification.review_status", "verification.rerun", "monitor.start", "snapshot.baseline", "guidance.refresh"] {
            let result = enrich_templates("some_action", vec![make_template(id)], false, false);
            assert_eq!(result.as_array().unwrap()[0]["can_run_in_parallel"], false, "expected false for {}", id);
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
        let result = enrich_templates("some_action", vec![make_template("totally.unknown")], false, false);
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
        let fields = result.as_array().unwrap()[0]["expected_output_fields"].as_array().unwrap();
        assert!(fields.contains(&json!("verification.latest_runs")));
        assert!(fields.contains(&json!("verification.failing_runs")));
    }

    #[test]
    fn expected_output_fields_for_guidance_refresh() {
        let result = enrich_templates("unknown", vec![make_template("guidance.refresh")], false, false);
        let fields = result.as_array().unwrap()[0]["expected_output_fields"].as_array().unwrap();
        assert!(fields.contains(&json!("guidance.recommended_flow")));
        assert!(fields.iter().any(|f| f.as_str().unwrap().contains("execution_strategy")));
    }

    #[test]
    fn expected_output_fields_empty_for_unknown_template() {
        let result = enrich_templates("unknown", vec![make_template("nonexistent.id")], false, false);
        let fields = result.as_array().unwrap()[0]["expected_output_fields"].as_array().unwrap();
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
        let result = enrich_templates("unknown", vec![make_template("guidance.refresh")], false, false);
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
        let success = result.as_array().unwrap()[0]["follow_up_on_success"].as_array().unwrap();
        assert!(success.contains(&json!("verification.status")));

        // cleanup ready: follow_up_on_success includes guidance.refresh
        let result2 = enrich_templates(
            "review_unused_files",
            vec![make_template("unused.search")],
            true,
            false,
        );
        let success2 = result2.as_array().unwrap()[0]["follow_up_on_success"].as_array().unwrap();
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
}
