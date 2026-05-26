use serde_json::Value;

use super::super::{set_recommended_flow, tool_guidance};

pub(in crate::mcp) fn register_project_guidance() -> Value {
    let mut guidance = tool_guidance(
        "Project registered. Start monitoring before relying on activity-based stats.",
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

#[cfg(test)]
mod tests {
    use super::*;

    // ---- register_project_guidance ----

    #[test]
    fn register_project_guidance_has_schema_version() {
        let guidance = register_project_guidance();
        assert!(guidance["schema_version"].is_string());
        assert!(!guidance["schema_version"].as_str().unwrap().is_empty());
    }

    #[test]
    fn register_project_guidance_has_summary() {
        let guidance = register_project_guidance();
        assert!(guidance["summary"].is_string());
        assert!(guidance["summary"].as_str().unwrap().contains("registered"));
    }

    #[test]
    fn register_project_guidance_has_suggested_commands() {
        let guidance = register_project_guidance();
        let commands = guidance["suggested_commands"].as_array().unwrap();
        assert!(!commands.is_empty());
    }

    #[test]
    fn register_project_guidance_has_next_tools() {
        let guidance = register_project_guidance();
        let tools = guidance["next_tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t == "start_monitor"));
        assert!(tools.iter().any(|t| t == "take_snapshot"));
    }

    #[test]
    fn register_project_guidance_has_recommended_flow() {
        let guidance = register_project_guidance();
        let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
            .as_array()
            .unwrap();
        assert!(!flow.is_empty());
        assert!(flow
            .iter()
            .any(|s| s.as_str().unwrap().contains("registered")));
    }

    #[test]
    fn register_project_guidance_has_when_to_use_shell() {
        let guidance = register_project_guidance();
        assert!(guidance["when_to_use_shell"].is_string());
    }

    // ---- snapshot_guidance ----

    #[test]
    fn snapshot_guidance_zero_files() {
        let guidance = snapshot_guidance(0);
        assert!(guidance["summary"].as_str().unwrap().contains("no files"));
        let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
            .as_array()
            .unwrap();
        assert!(flow
            .iter()
            .any(|s| s.as_str().unwrap().contains("zero files")));
    }

    #[test]
    fn snapshot_guidance_zero_files_next_tools() {
        let guidance = snapshot_guidance(0);
        let tools = guidance["next_tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t == "list_projects"));
        assert!(tools.iter().any(|t| t == "take_snapshot"));
    }

    #[test]
    fn snapshot_guidance_with_files() {
        let guidance = snapshot_guidance(42);
        assert!(guidance["summary"]
            .as_str()
            .unwrap()
            .contains("Snapshot complete"));
    }

    #[test]
    fn snapshot_guidance_with_files_recommends_stats() {
        let guidance = snapshot_guidance(100);
        let tools = guidance["next_tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t == "get_stats"));
        assert!(tools.iter().any(|t| t == "get_unused_files"));
    }

    #[test]
    fn snapshot_guidance_with_files_has_recommended_flow() {
        let guidance = snapshot_guidance(10);
        let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
            .as_array()
            .unwrap();
        assert!(flow
            .iter()
            .any(|s| s.as_str().unwrap().contains("baseline")));
    }

    #[test]
    fn snapshot_guidance_with_files_has_shell_guidance() {
        let guidance = snapshot_guidance(10);
        assert!(guidance["when_to_use_shell"].is_string());
    }

    // ---- start_monitor_guidance ----

    #[test]
    fn start_monitor_guidance_already_running() {
        let guidance = start_monitor_guidance(true, false);
        assert!(guidance["summary"]
            .as_str()
            .unwrap()
            .contains("already active"));
        let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
            .as_array()
            .unwrap();
        assert!(flow
            .iter()
            .any(|s| s.as_str().unwrap().contains("already active")));
    }

    #[test]
    fn start_monitor_guidance_already_running_recommends_stats() {
        let guidance = start_monitor_guidance(true, false);
        let tools = guidance["next_tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t == "get_stats"));
    }

    #[test]
    fn start_monitor_guidance_not_running_with_snapshot() {
        let guidance = start_monitor_guidance(false, true);
        assert!(guidance["summary"]
            .as_str()
            .unwrap()
            .contains("initial snapshot"));
        let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
            .as_array()
            .unwrap();
        assert!(flow
            .iter()
            .any(|s| s.as_str().unwrap().contains("baseline snapshot")));
    }

    #[test]
    fn start_monitor_guidance_not_running_with_snapshot_recommends_activity() {
        let guidance = start_monitor_guidance(false, true);
        let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
            .as_array()
            .unwrap();
        assert!(flow
            .iter()
            .any(|s| s.as_str().unwrap().contains("real project activity")));
    }

    #[test]
    fn start_monitor_guidance_not_running_no_snapshot() {
        let guidance = start_monitor_guidance(false, false);
        assert!(guidance["summary"]
            .as_str()
            .unwrap()
            .contains("Monitoring is active"));
        let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
            .as_array()
            .unwrap();
        assert!(flow
            .iter()
            .any(|s| s.as_str().unwrap().contains("activity-derived")));
    }

    #[test]
    fn start_monitor_guidance_not_running_no_snapshot_recommends_stats() {
        let guidance = start_monitor_guidance(false, false);
        let tools = guidance["next_tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t == "get_stats"));
        assert!(tools.iter().any(|t| t == "get_unused_files"));
    }

    #[test]
    fn start_monitor_guidance_all_paths_have_schema_version() {
        for (running, snapshot) in [(true, true), (true, false), (false, true), (false, false)] {
            let guidance = start_monitor_guidance(running, snapshot);
            assert!(guidance["schema_version"].is_string());
        }
    }
}
