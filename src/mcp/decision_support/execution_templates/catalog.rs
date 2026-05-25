use serde_json::{json, Value};

pub(super) fn base_templates(
    action: &str,
    project_id_value: &str,
    verification_status: &str,
    repo_risk_level: &str,
    cleanup_ready: bool,
    refactor_ready: bool,
    project_placeholder_hint: &Value,
) -> Vec<Value> {
    match action {
        "review_failing_verification" => vec![
            json!({
                "template_id": "verification.review_status",
                "kind": "mcp_tool",
                "tool": "get_verification_status",
                "args_template": { "id": project_id_value },
                "preconditions": ["project must exist in OPENDOG"],
                "blocking_conditions": [],
                "success_signal": "latest verification runs are loaded for inspection",
                "parameter_schema": {
                    "id": { "type": "string", "required": true, "source": "project_id" }
                },
                "default_values": {},
                "placeholder_hints": project_placeholder_hint.clone()
            }),
            json!({
                "template_id": "verification.rerun",
                "kind": "cli_command",
                "command_template": format!("opendog run-verification --id {} --kind test --command '<project-test-command>' --json", project_id_value),
                "preconditions": ["choose a real project-native test, lint, or build command"],
                "blocking_conditions": [],
                "success_signal": "a fresh verification result is recorded in OPENDOG",
                "parameter_schema": {
                    "id": { "type": "string", "required": true, "source": "project_id" },
                    "kind": { "type": "enum", "required": true, "allowed_values": ["test", "lint", "build"] },
                    "command": { "type": "string", "required": true, "must_replace_placeholder": true }
                },
                "default_values": { "kind": "test", "json": true },
                "placeholder_hints": json!([
                    {
                        "field": "command",
                        "placeholder": "<project-test-command>",
                        "description": "replace with a real test, lint, or build command such as cargo test or pytest"
                    }
                ])
            }),
        ],
        "stabilize_repository_state" => vec![
            json!({
                "template_id": "repo.status",
                "kind": "shell_command",
                "command_template": "git status",
                "preconditions": ["run inside the target repository root"],
                "blocking_conditions": [],
                "success_signal": "current repository operation state is visible",
                "parameter_schema": {},
                "default_values": {},
                "placeholder_hints": []
            }),
            json!({
                "template_id": "repo.diff",
                "kind": "shell_command",
                "command_template": "git diff",
                "preconditions": ["working tree must be accessible"],
                "blocking_conditions": [],
                "success_signal": "the unstable change set is visible for manual review",
                "parameter_schema": {},
                "default_values": {},
                "placeholder_hints": []
            }),
        ],
        "start_monitor" => vec![
            json!({
                "template_id": "monitor.start",
                "kind": "mcp_tool",
                "tool": "start_monitor",
                "args_template": { "id": project_id_value },
                "preconditions": ["project must already be registered in OPENDOG"],
                "blocking_conditions": [],
                "success_signal": "project status becomes monitoring",
                "parameter_schema": {
                    "id": { "type": "string", "required": true, "source": "project_id" }
                },
                "default_values": {},
                "placeholder_hints": project_placeholder_hint.clone()
            }),
            json!({
                "template_id": "snapshot.baseline",
                "kind": "cli_command",
                "command_template": format!("opendog snapshot --id {}", project_id_value),
                "preconditions": ["use when no snapshot baseline exists or the baseline is stale"],
                "blocking_conditions": [],
                "success_signal": "snapshot baseline exists for the project",
                "parameter_schema": {
                    "id": { "type": "string", "required": true, "source": "project_id" }
                },
                "default_values": {},
                "placeholder_hints": project_placeholder_hint.clone()
            }),
        ],
        "take_snapshot" => vec![
            json!({
                "template_id": "snapshot.take",
                "kind": "mcp_tool",
                "tool": "take_snapshot",
                "args_template": { "id": project_id_value },
                "preconditions": ["project must already be registered in OPENDOG"],
                "blocking_conditions": [],
                "success_signal": "snapshot completes and file inventory exists",
                "parameter_schema": {
                    "id": { "type": "string", "required": true, "source": "project_id" }
                },
                "default_values": {},
                "placeholder_hints": project_placeholder_hint.clone()
            }),
            json!({
                "template_id": "stats.inspect",
                "kind": "cli_command",
                "command_template": format!("opendog stats --id {}", project_id_value),
                "preconditions": ["use after snapshot when you want activity-ranked files"],
                "blocking_conditions": [],
                "success_signal": "stats payload is available for hotspot review",
                "parameter_schema": {
                    "id": { "type": "string", "required": true, "source": "project_id" }
                },
                "default_values": {},
                "placeholder_hints": project_placeholder_hint.clone()
            }),
        ],
        "generate_activity_then_stats" => vec![
            json!({
                "template_id": "activity.generate",
                "kind": "shell_command",
                "command_template": "git status",
                "preconditions": ["run some real edit, build, test, or search workflow before relying on OPENDOG activity"],
                "blocking_conditions": [],
                "success_signal": "real project interaction has occurred",
                "parameter_schema": {},
                "default_values": {},
                "placeholder_hints": []
            }),
            json!({
                "template_id": "stats.refresh",
                "kind": "mcp_tool",
                "tool": "get_stats",
                "args_template": { "id": project_id_value },
                "preconditions": ["some observation window must have passed after activity"],
                "blocking_conditions": [],
                "success_signal": "accessed files are no longer zero",
                "parameter_schema": {
                    "id": { "type": "string", "required": true, "source": "project_id" }
                },
                "default_values": {},
                "placeholder_hints": project_placeholder_hint.clone()
            }),
        ],
        "run_verification_before_high_risk_changes" => vec![
            json!({
                "template_id": "verification.status",
                "kind": "mcp_tool",
                "tool": "get_verification_status",
                "args_template": { "id": project_id_value },
                "preconditions": ["project must exist in OPENDOG"],
                "blocking_conditions": [],
                "success_signal": "missing or stale verification evidence is identified",
                "parameter_schema": {
                    "id": { "type": "string", "required": true, "source": "project_id" }
                },
                "default_values": {},
                "placeholder_hints": project_placeholder_hint.clone()
            }),
            json!({
                "template_id": "verification.execute",
                "kind": "mcp_tool",
                "tool": "run_verification_command",
                "args_template": {
                    "id": project_id_value,
                    "kind": "test",
                    "command": "<project-test-command>",
                    "source": "mcp"
                },
                "preconditions": ["replace <project-test-command> with a real test, lint, or build command"],
                "blocking_conditions": [],
                "success_signal": "verification evidence is recorded and status becomes available",
                "parameter_schema": {
                    "id": { "type": "string", "required": true, "source": "project_id" },
                    "kind": { "type": "enum", "required": true, "allowed_values": ["test", "lint", "build"] },
                    "command": { "type": "string", "required": true, "must_replace_placeholder": true },
                    "source": { "type": "string", "required": false }
                },
                "default_values": { "kind": "test", "source": "mcp" },
                "placeholder_hints": json!([
                    {
                        "field": "command",
                        "placeholder": "<project-test-command>",
                        "description": "replace with a real test, lint, or build command"
                    }
                ])
            }),
        ],
        "review_unused_files" => vec![
            json!({
                "template_id": "unused.list",
                "kind": "mcp_tool",
                "tool": "get_unused_files",
                "args_template": { "id": project_id_value },
                "preconditions": [
                    "snapshot baseline must exist",
                    "some activity should have been observed"
                ],
                "blocking_conditions": if cleanup_ready { json!([]) } else { json!(["cleanup is not yet ready; review only, do not delete"]) },
                "success_signal": "unused candidates are listed for manual inspection",
                "parameter_schema": {
                    "id": { "type": "string", "required": true, "source": "project_id" }
                },
                "default_values": {},
                "placeholder_hints": project_placeholder_hint.clone()
            }),
            json!({
                "template_id": "unused.search",
                "kind": "shell_command",
                "command_template": "rg \"<symbol-or-path-fragment>\" .",
                "preconditions": ["replace <symbol-or-path-fragment> with a candidate path or import symbol"],
                "blocking_conditions": if cleanup_ready { json!([]) } else { json!(["cleanup blockers still exist; verify references before any edit"]) },
                "success_signal": "candidate usage is manually confirmed or ruled out",
                "parameter_schema": {
                    "pattern": { "type": "string", "required": true, "must_replace_placeholder": true }
                },
                "default_values": {},
                "placeholder_hints": json!([
                    {
                        "field": "pattern",
                        "placeholder": "<symbol-or-path-fragment>",
                        "description": "replace with a candidate file path fragment, symbol name, or import string"
                    }
                ])
            }),
        ],
        "inspect_hot_files" => vec![
            json!({
                "template_id": "stats.hot_files",
                "kind": "mcp_tool",
                "tool": "get_stats",
                "args_template": { "id": project_id_value },
                "preconditions": ["snapshot baseline and some activity data should exist"],
                "blocking_conditions": if refactor_ready { json!([]) } else { json!(["refactor is not yet ready; inspect only"]) },
                "success_signal": "hot files are ranked for targeted review",
                "parameter_schema": {
                    "id": { "type": "string", "required": true, "source": "project_id" }
                },
                "default_values": {},
                "placeholder_hints": project_placeholder_hint.clone()
            }),
            json!({
                "template_id": "hot.diff",
                "kind": "shell_command",
                "command_template": "git diff",
                "preconditions": ["run inside the target repository root"],
                "blocking_conditions": if repo_risk_level == "high" { json!(["repository risk is high; avoid broad modifications"]) } else { json!([]) },
                "success_signal": "hot-file changes are visible in repository context",
                "parameter_schema": {},
                "default_values": {},
                "placeholder_hints": []
            }),
        ],
        _ => vec![json!({
            "template_id": "guidance.refresh",
            "kind": "mcp_tool",
            "tool": "get_guidance",
            "args_template": { "detail": "summary" },
            "preconditions": [],
            "blocking_conditions": [],
            "success_signal": format!("guidance is refreshed; current verification status is {}", verification_status),
            "parameter_schema": {
                "detail": { "type": "enum", "required": false, "allowed_values": ["summary", "decision"] }
            },
            "default_values": { "detail": "summary" },
            "placeholder_hints": []
        })],
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(template_ids(&t), vec!["verification.review_status", "verification.rerun"]);
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
        assert_eq!(template_ids(&t), vec!["verification.status", "verification.execute"]);
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
                assert!(t["template_id"].is_string(), "missing template_id for action {}", action);
                assert!(t["preconditions"].is_array(), "missing preconditions for action {}", action);
                assert!(t["blocking_conditions"].is_array(), "missing blocking_conditions for action {}", action);
                assert!(t["success_signal"].is_string(), "missing success_signal for action {}", action);
                assert!(t["parameter_schema"].is_object(), "missing parameter_schema for action {}", action);
                assert!(t["default_values"].is_object(), "missing default_values for action {}", action);
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
            assert!(!bc.is_empty(), "expected blocking_conditions when cleanup_ready=false");
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
            assert!(bc.is_empty(), "expected empty blocking_conditions when cleanup_ready=true");
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
        let templates = base_templates(
            "whatever",
            "p",
            "missing",
            "low",
            false,
            false,
            &json!([]),
        );
        let sig = templates[0]["success_signal"].as_str().unwrap();
        assert!(sig.contains("missing"), "expected verification status in success_signal");
    }
}
