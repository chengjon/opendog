use serde_json::{json, Value};

pub(super) fn strategy_profile(
    strategy_mode: &str,
    preferred_primary_tool: &str,
    preferred_secondary_tool: &str,
    evidence_priority: &[&str],
) -> Value {
    json!({
        "strategy_mode": strategy_mode,
        "preferred_primary_tool": preferred_primary_tool,
        "preferred_secondary_tool": preferred_secondary_tool,
        "evidence_priority": evidence_priority,
    })
}

pub(super) fn workspace_strategy_profile(
    project_count: usize,
    monitoring_count: usize,
    has_failing_verification: bool,
    has_mid_operation_repo: bool,
    missing_verification_projects: usize,
) -> Value {
    let (global_strategy_mode, preferred_primary_tool, preferred_secondary_tool, recommended_flow) =
        if has_failing_verification {
            (
                "verify_before_modify",
                "shell",
                "opendog",
                vec![
                    "Inspect recorded failing verification first; do not start broad refactors while test/lint/build evidence is failing or uncertain.".to_string(),
                    "Use `opendog verification --id <project>` or `get_verification_status` to inspect the latest failing verification records.".to_string(),
                    "Only return to activity-based cleanup or refactor work after verification is stable again.".to_string(),
                ],
            )
        } else if has_mid_operation_repo {
            (
                "stabilize_before_modify",
                "shell",
                "opendog",
                vec![
                    "Stabilize repositories that are mid-merge, rebase, cherry-pick, or bisect before making broader code changes.".to_string(),
                    "Use `git status` and `git diff` to understand the in-progress repository operation.".to_string(),
                    "Once repository state is stable, resume OPENDOG-driven cleanup or hotspot review.".to_string(),
                ],
            )
        } else if project_count == 0 {
            (
                "collect_workspace_context",
                "opendog",
                "shell",
                vec![
                    "Register a project first with `create_project` or `opendog create --id <project> --path <root>`.".to_string(),
                    "Start monitoring immediately after creation so opendog can build activity data.".to_string(),
                    "Use shell commands such as `rg` only after you know which project root you want to inspect.".to_string(),
                ],
            )
        } else if monitoring_count == 0 {
            (
                "collect_evidence_first",
                "opendog",
                "shell",
                vec![
                    "Use `opendog list` first to pick a project that should be monitored.".to_string(),
                    "Use `opendog start --id <project>` to ensure monitoring is active; it can take an initial snapshot automatically.".to_string(),
                    "After some workflow activity, use `opendog stats --id <project>` to inspect hotspots.".to_string(),
                ],
            )
        } else {
            (
                if missing_verification_projects > 0 {
                    "verify_before_high_risk_changes"
                } else {
                    "activity_guided_review"
                },
                "opendog",
                "shell",
                vec![
                    "Use `opendog list` first to confirm which projects are already being monitored.".to_string(),
                    "Use `opendog stats --id <project>` or the `get_stats` MCP tool after monitoring to inspect activity hotspots.".to_string(),
                    "Use `opendog unused --id <project>` or the `get_unused_files` MCP tool to review never-accessed files before cleanup.".to_string(),
                ],
            )
        };

    json!({
        "global_strategy_mode": global_strategy_mode,
        "preferred_primary_tool": preferred_primary_tool,
        "preferred_secondary_tool": preferred_secondary_tool,
        "evidence_priority": [
            "verification",
            "repository_risk",
            "activity_signals",
        ],
        "recommended_flow": recommended_flow,
    })
}

pub(super) fn agent_guidance_recommended_flow(
    project_count: usize,
    monitoring_count: usize,
    top_recommendation: Option<&Value>,
    workspace_strategy: &Value,
) -> Value {
    if project_count == 0 {
        return workspace_strategy["recommended_flow"].clone();
    }

    let Some(recommendation) = top_recommendation else {
        return workspace_strategy["recommended_flow"].clone();
    };

    let project_id = recommendation["project_id"].as_str().unwrap_or("<project>");
    let action = recommendation["recommended_next_action"]
        .as_str()
        .unwrap_or_default();

    let steps = match action {
        "review_failing_verification" => vec![
            format!(
                "Start with project '{}' because failing or uncertain verification needs to be stabilized first.",
                project_id
            ),
            format!(
                "Inspect recorded verification with `opendog verification --id {}` or `get_verification_status`.",
                project_id
            ),
            "Use shell diff and project-native verification commands before returning to cleanup or refactor work."
                .to_string(),
        ],
        "stabilize_repository_state" => vec![
            format!(
                "Start with project '{}' because repository state is mid-operation and must be stabilized first.",
                project_id
            ),
            "Use `git status` and `git diff` to understand the in-progress merge, rebase, cherry-pick, or bisect."
                .to_string(),
            "Resume OPENDOG-driven review only after repository state is stable again.".to_string(),
        ],
        "start_monitor" => vec![
            format!(
                "Start with project '{}' because fresh activity evidence cannot exist until monitoring is active.",
                project_id
            ),
            format!("Run `opendog start --id {}` or `start_monitor` first.", project_id),
            format!(
                "After some workflow activity, inspect `opendog stats --id {}` or `get_stats`.",
                project_id
            ),
        ],
        "take_snapshot" => vec![
            format!(
                "Start with project '{}' because no snapshot baseline exists yet.",
                project_id
            ),
            format!("Run `opendog snapshot --id {}` or `take_snapshot` first.", project_id),
            "Only interpret unused-file or hotspot guidance after the baseline is established."
                .to_string(),
        ],
        "generate_activity_then_stats" => vec![
            format!(
                "Start with project '{}' because snapshot data exists but no meaningful file activity has been recorded yet.",
                project_id
            ),
            "Drive real workflow activity with edits, tests, or builds.".to_string(),
            format!(
                "Then inspect `opendog stats --id {}` or `get_stats` for activity-derived guidance.",
                project_id
            ),
        ],
        "run_verification_before_high_risk_changes" => vec![
            format!(
                "Start with project '{}' because activity exists but verification evidence is still missing.",
                project_id
            ),
            "Run and record project-native test, lint, or build commands before high-risk changes."
                .to_string(),
            "Only return to cleanup or refactor work after verification evidence exists.".to_string(),
        ],
        "review_unused_files" => vec![
            format!(
                "Start with project '{}' because unused-file candidates currently deserve review.",
                project_id
            ),
            format!(
                "Inspect `opendog unused --id {}` or `get_unused_files` before proposing cleanup.",
                project_id
            ),
            "Validate candidates with shell search, imports, and tests before deletion.".to_string(),
        ],
        "inspect_hot_files" => vec![
            format!(
                "Start with project '{}' because current evidence points to hotspot-driven review rather than cleanup.",
                project_id
            ),
            format!(
                "Inspect `opendog stats --id {}` or `get_stats` to find the hottest files first.",
                project_id
            ),
            "Use shell diff and symbol search once OPENDOG narrows the review target.".to_string(),
        ],
        _ => {
            if monitoring_count == 0 {
                vec![
                    "No project is currently monitored, so start by enabling observation.".to_string(),
                    "Choose a project from `opendog list` and start monitoring it.".to_string(),
                    "Return to activity-based guidance after some workflow activity exists.".to_string(),
                ]
            } else {
                return workspace_strategy["recommended_flow"].clone();
            }
        }
    };

    json!(steps)
}
