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
            "tool": "get_agent_guidance",
            "args_template": {},
            "preconditions": [],
            "blocking_conditions": [],
            "success_signal": format!("guidance is refreshed; current verification status is {}", verification_status),
            "parameter_schema": {},
            "default_values": {},
            "placeholder_hints": []
        })],
    }
}
