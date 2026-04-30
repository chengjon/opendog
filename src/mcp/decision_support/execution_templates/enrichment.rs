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
