use super::{DecisionEntrypointsPlan, EntrypointReasonKind, EntrypointSelectionReason};

pub(super) fn plan_for_action(action: &str, project_id: Option<&str>) -> DecisionEntrypointsPlan {
    let project_flag = project_id
        .map(|id| format!(" --id {id}"))
        .unwrap_or_default();

    match action {
        "review_failing_verification" => DecisionEntrypointsPlan::new(
            &[
                "get_verification_status",
                "run_verification_command",
                "get_data_risk_candidates",
            ],
            vec![
                format!("opendog verification{project_flag}"),
                format!("opendog run-verification{project_flag} --kind test --command '<cmd>'"),
                format!("opendog data-risk{project_flag} --json"),
            ],
            vec![
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "get_verification_status",
                    "inspect persisted evidence before new edits",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "run_verification_command",
                    "refresh failing or missing verification evidence",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::CliOrShell,
                    "git diff",
                    "compare current repository state against the failing evidence",
                ),
            ],
        ),
        "stabilize_repository_state" => DecisionEntrypointsPlan::new(
            &["get_guidance", "get_verification_status"],
            vec![
                "git status".to_string(),
                "git diff".to_string(),
                format!("opendog verification{project_flag}"),
            ],
            vec![
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::ShellCommand,
                    "git status",
                    "repository truth is needed before activity-based guidance",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::ShellCommand,
                    "git diff",
                    "inspect the unstable working state directly",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "get_verification_status",
                    "avoid cleanup or refactor while evidence is stale or failing",
                ),
            ],
        ),
        "start_monitor" => DecisionEntrypointsPlan::new(
            &["start_monitor", "take_snapshot", "get_stats"],
            vec![
                format!("opendog start{project_flag}"),
                format!("opendog snapshot{project_flag}"),
                format!("opendog stats{project_flag}"),
            ],
            vec![
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "start_monitor",
                    "fresh activity evidence does not exist yet",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "take_snapshot",
                    "monitoring is more useful when a file baseline exists",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "get_stats",
                    "only meaningful after monitoring and baseline data exist",
                ),
            ],
        ),
        "take_snapshot" => DecisionEntrypointsPlan::new(
            &["take_snapshot", "get_stats", "get_unused_files"],
            vec![
                format!("opendog snapshot{project_flag}"),
                format!("opendog stats{project_flag}"),
                format!("opendog unused{project_flag}"),
            ],
            vec![
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "take_snapshot",
                    "file inventory is still missing",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "get_stats",
                    "activity rankings depend on an established snapshot",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "get_unused_files",
                    "cleanup review depends on the snapshot baseline",
                ),
            ],
        ),
        "generate_activity_then_stats" => DecisionEntrypointsPlan::new(
            &["get_stats", "get_guidance"],
            vec![
                "git status".to_string(),
                format!("opendog stats{project_flag}"),
            ],
            vec![
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::ShellCommand,
                    "git status",
                    "real workflow activity usually comes from edits, builds, or tests",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "get_stats",
                    "wait until enough activity exists before trusting hotspot signals",
                ),
            ],
        ),
        "run_verification_before_high_risk_changes" => DecisionEntrypointsPlan::new(
            &[
                "get_verification_status",
                "run_verification_command",
                "get_data_risk_candidates",
            ],
            vec![
                format!("opendog verification{project_flag}"),
                format!("opendog run-verification{project_flag} --kind test --command '<cmd>'"),
                format!("opendog data-risk{project_flag} --json"),
            ],
            vec![
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "get_verification_status",
                    "determine exactly which evidence is still missing",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "run_verification_command",
                    "record fresh build/test/lint evidence before risky changes",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "get_data_risk_candidates",
                    "check whether suspicious data files raise cleanup or refactor risk",
                ),
            ],
        ),
        "review_unused_files" => DecisionEntrypointsPlan::new(
            &[
                "get_unused_files",
                "get_verification_status",
                "get_data_risk_candidates",
            ],
            vec![
                format!("opendog unused{project_flag}"),
                format!("opendog verification{project_flag}"),
                "rg \"<pattern>\" .".to_string(),
            ],
            vec![
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "get_unused_files",
                    "OPENDOG has already ranked cleanup candidates",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "get_verification_status",
                    "cleanup decisions still need evidence gates",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::ShellCommand,
                    "rg \"<pattern>\" .",
                    "confirm whether unused candidates are referenced indirectly",
                ),
            ],
        ),
        "inspect_hot_files" => DecisionEntrypointsPlan::new(
            &[
                "get_stats",
                "get_verification_status",
                "get_data_risk_candidates",
            ],
            vec![
                format!("opendog stats{project_flag}"),
                "git diff".to_string(),
                "rg \"<pattern>\" .".to_string(),
            ],
            vec![
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "get_stats",
                    "activity-based ranking should narrow the first inspection target",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::ShellCommand,
                    "git diff",
                    "hot files still need source-level repository context",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::ShellCommand,
                    "rg \"<pattern>\" .",
                    "follow activity signals with code search before edits",
                ),
            ],
        ),
        _ => DecisionEntrypointsPlan::new(
            &[
                "get_guidance",
                "list_projects",
                "get_workspace_data_risk_overview",
            ],
            vec![
                "opendog agent-guidance".to_string(),
                "opendog list".to_string(),
                "opendog workspace-data-risk --json".to_string(),
            ],
            vec![
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "get_guidance",
                    "refresh project-level next-action advice",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "list_projects",
                    "reconfirm project availability and monitoring state",
                ),
                EntrypointSelectionReason::new(
                    EntrypointReasonKind::McpTool,
                    "get_workspace_data_risk_overview",
                    "cross-project prioritization may still be unresolved",
                ),
            ],
        ),
    }
}
