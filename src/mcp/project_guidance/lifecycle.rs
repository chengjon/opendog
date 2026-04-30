use serde_json::Value;

use super::super::{set_recommended_flow, tool_guidance};

pub(in crate::mcp) fn create_project_guidance() -> Value {
    let mut guidance = tool_guidance(
        "Project created. Start monitoring before relying on activity-based stats.",
        &[
            "opendog start --id <project>",
            "opendog snapshot --id <project>",
            "rg \"<pattern>\" .",
        ],
        &["start_monitor", "take_snapshot", "list_projects"],
        Some("Use shell commands when you need repository-wide search or project-native tests."),
    );
    set_recommended_flow(
        &mut guidance,
        &[
            "Project is now registered in OPENDOG.",
            "Take a snapshot if no baseline exists yet.",
            "Start monitoring before relying on activity-derived stats.",
            "After some workflow activity, query stats or unused files.",
        ],
    );
    guidance
}

pub(in crate::mcp) fn snapshot_guidance(total_files: usize) -> Value {
    if total_files == 0 {
        let mut guidance = tool_guidance(
            "Snapshot completed but no files were recorded. Check the project root or ignore patterns before relying on stats.",
            &[
                "opendog list",
                "rg --files .",
                "opendog snapshot --id <project>",
            ],
            &["list_projects", "take_snapshot"],
            Some("Use shell file listing to verify the repository actually contains files under the registered root."),
        );
        set_recommended_flow(
            &mut guidance,
            &[
                "Snapshot completed but recorded zero files.",
                "Verify the registered root and ignore patterns with shell listing.",
                "Re-run snapshot only after the baseline path issue is understood.",
            ],
        );
        guidance
    } else {
        let mut guidance = tool_guidance(
            "Snapshot complete. Query stats next if you want usage-based decisions.",
            &[
                "opendog stats --id <project>",
                "opendog unused --id <project>",
                "rg \"<pattern>\" .",
            ],
            &["get_stats", "get_unused_files", "list_projects"],
            Some("Use shell search after snapshot when you need to inspect specific files or symbols."),
        );
        set_recommended_flow(
            &mut guidance,
            &[
                "Snapshot established a project baseline.",
                "Start or continue monitoring so OPENDOG can observe real file activity.",
                "After activity exists, query stats or unused files for review decisions.",
            ],
        );
        guidance
    }
}

pub(in crate::mcp) fn start_monitor_guidance(already_running: bool, snapshot_taken: bool) -> Value {
    if already_running {
        let mut guidance = tool_guidance(
            "Monitoring was already active. Inspect stats or unused files rather than starting again.",
            &[
                "opendog stats --id <project>",
                "opendog unused --id <project>",
                "git status",
            ],
            &["get_stats", "get_unused_files", "list_projects"],
            Some("Use shell commands for repo status or tests; opendog is for activity-derived file guidance."),
        );
        set_recommended_flow(
            &mut guidance,
            &[
                "Monitoring was already active, so do not start a second path.",
                "Let workflow activity accumulate if needed.",
                "Inspect stats or unused files instead of repeating start.",
            ],
        );
        guidance
    } else if snapshot_taken {
        let mut guidance = tool_guidance(
            "Monitoring is active and an initial snapshot was taken automatically. Let some workflow activity happen, then inspect stats.",
            &[
                "opendog stats --id <project>",
                "opendog unused --id <project>",
                "cargo test",
            ],
            &["get_stats", "get_unused_files", "list_projects"],
            Some("Use shell commands to drive real activity, such as tests or builds, before interpreting opendog stats."),
        );
        set_recommended_flow(
            &mut guidance,
            &[
                "Monitoring is active and baseline snapshot exists.",
                "Drive real project activity with edits, tests, or builds.",
                "Query stats or unused files only after enough activity has been observed.",
            ],
        );
        guidance
    } else {
        let mut guidance = tool_guidance(
            "Monitoring is active. Next, inspect stats or unused files after some workflow activity.",
            &[
                "opendog stats --id <project>",
                "opendog unused --id <project>",
                "git status",
            ],
            &["get_stats", "get_unused_files", "list_projects"],
            Some("Use shell commands for repository state or tests; use opendog for activity-based file decisions."),
        );
        set_recommended_flow(
            &mut guidance,
            &[
                "Monitoring is active but activity-derived conclusions still require observed workflow usage.",
                "Generate real project activity if needed.",
                "Inspect stats or unused files after the observation window is meaningful.",
            ],
        );
        guidance
    }
}
