use serde_json::{json, Value};

pub(in crate::mcp) fn decision_entrypoints_payload(
    action: &str,
    project_id: Option<&str>,
    preferred_primary_tool: &str,
    preferred_secondary_tool: &str,
) -> Value {
    let project_flag = project_id
        .map(|id| format!(" --id {}", id))
        .unwrap_or_default();
    let (next_mcp_tools, next_cli_commands, selection_reasons) = match action {
        "review_failing_verification" => (
            vec![
                "get_verification_status".to_string(),
                "run_verification_command".to_string(),
                "get_data_risk_candidates".to_string(),
            ],
            vec![
                format!("opendog verification{}", project_flag),
                format!(
                    "opendog run-verification{} --kind test --command '<cmd>'",
                    project_flag
                ),
                format!("opendog data-risk{} --json", project_flag),
            ],
            vec![
                json!({"kind":"mcp_tool","target":"get_verification_status","why":"inspect persisted evidence before new edits"}),
                json!({"kind":"mcp_tool","target":"run_verification_command","why":"refresh failing or missing verification evidence"}),
                json!({"kind":"cli_or_shell","target":"git diff","why":"compare current repository state against the failing evidence"}),
            ],
        ),
        "stabilize_repository_state" => (
            vec![
                "get_guidance".to_string(),
                "get_verification_status".to_string(),
            ],
            vec![
                "git status".to_string(),
                "git diff".to_string(),
                format!("opendog verification{}", project_flag),
            ],
            vec![
                json!({"kind":"shell_command","target":"git status","why":"repository truth is needed before activity-based guidance"}),
                json!({"kind":"shell_command","target":"git diff","why":"inspect the unstable working state directly"}),
                json!({"kind":"mcp_tool","target":"get_verification_status","why":"avoid cleanup or refactor while evidence is stale or failing"}),
            ],
        ),
        "start_monitor" => (
            vec![
                "start_monitor".to_string(),
                "take_snapshot".to_string(),
                "get_stats".to_string(),
            ],
            vec![
                format!("opendog start{}", project_flag),
                format!("opendog snapshot{}", project_flag),
                format!("opendog stats{}", project_flag),
            ],
            vec![
                json!({"kind":"mcp_tool","target":"start_monitor","why":"fresh activity evidence does not exist yet"}),
                json!({"kind":"mcp_tool","target":"take_snapshot","why":"monitoring is more useful when a file baseline exists"}),
                json!({"kind":"mcp_tool","target":"get_stats","why":"only meaningful after monitoring and baseline data exist"}),
            ],
        ),
        "take_snapshot" => (
            vec![
                "take_snapshot".to_string(),
                "get_stats".to_string(),
                "get_unused_files".to_string(),
            ],
            vec![
                format!("opendog snapshot{}", project_flag),
                format!("opendog stats{}", project_flag),
                format!("opendog unused{}", project_flag),
            ],
            vec![
                json!({"kind":"mcp_tool","target":"take_snapshot","why":"file inventory is still missing"}),
                json!({"kind":"mcp_tool","target":"get_stats","why":"activity rankings depend on an established snapshot"}),
                json!({"kind":"mcp_tool","target":"get_unused_files","why":"cleanup review depends on the snapshot baseline"}),
            ],
        ),
        "generate_activity_then_stats" => (
            vec!["get_stats".to_string(), "get_guidance".to_string()],
            vec![
                "git status".to_string(),
                format!("opendog stats{}", project_flag),
            ],
            vec![
                json!({"kind":"shell_command","target":"git status","why":"real workflow activity usually comes from edits, builds, or tests"}),
                json!({"kind":"mcp_tool","target":"get_stats","why":"wait until enough activity exists before trusting hotspot signals"}),
            ],
        ),
        "run_verification_before_high_risk_changes" => (
            vec![
                "get_verification_status".to_string(),
                "run_verification_command".to_string(),
                "get_data_risk_candidates".to_string(),
            ],
            vec![
                format!("opendog verification{}", project_flag),
                format!(
                    "opendog run-verification{} --kind test --command '<cmd>'",
                    project_flag
                ),
                format!("opendog data-risk{} --json", project_flag),
            ],
            vec![
                json!({"kind":"mcp_tool","target":"get_verification_status","why":"determine exactly which evidence is still missing"}),
                json!({"kind":"mcp_tool","target":"run_verification_command","why":"record fresh build/test/lint evidence before risky changes"}),
                json!({"kind":"mcp_tool","target":"get_data_risk_candidates","why":"check whether suspicious data files raise cleanup or refactor risk"}),
            ],
        ),
        "review_unused_files" => (
            vec![
                "get_unused_files".to_string(),
                "get_verification_status".to_string(),
                "get_data_risk_candidates".to_string(),
            ],
            vec![
                format!("opendog unused{}", project_flag),
                format!("opendog verification{}", project_flag),
                "rg \"<pattern>\" .".to_string(),
            ],
            vec![
                json!({"kind":"mcp_tool","target":"get_unused_files","why":"OPENDOG has already ranked cleanup candidates"}),
                json!({"kind":"mcp_tool","target":"get_verification_status","why":"cleanup decisions still need evidence gates"}),
                json!({"kind":"shell_command","target":"rg \"<pattern>\" .","why":"confirm whether unused candidates are referenced indirectly"}),
            ],
        ),
        "inspect_hot_files" => (
            vec![
                "get_stats".to_string(),
                "get_verification_status".to_string(),
                "get_data_risk_candidates".to_string(),
            ],
            vec![
                format!("opendog stats{}", project_flag),
                "git diff".to_string(),
                "rg \"<pattern>\" .".to_string(),
            ],
            vec![
                json!({"kind":"mcp_tool","target":"get_stats","why":"activity-based ranking should narrow the first inspection target"}),
                json!({"kind":"shell_command","target":"git diff","why":"hot files still need source-level repository context"}),
                json!({"kind":"shell_command","target":"rg \"<pattern>\" .","why":"follow activity signals with code search before edits"}),
            ],
        ),
        _ => (
            vec![
                "get_guidance".to_string(),
                "list_projects".to_string(),
                "get_workspace_data_risk_overview".to_string(),
            ],
            vec![
                "opendog agent-guidance".to_string(),
                "opendog list".to_string(),
                "opendog workspace-data-risk --json".to_string(),
            ],
            vec![
                json!({"kind":"mcp_tool","target":"get_guidance","why":"refresh project-level next-action advice"}),
                json!({"kind":"mcp_tool","target":"list_projects","why":"reconfirm project availability and monitoring state"}),
                json!({"kind":"mcp_tool","target":"get_workspace_data_risk_overview","why":"cross-project prioritization may still be unresolved"}),
            ],
        ),
    };

    json!({
        "next_mcp_tools": next_mcp_tools,
        "next_cli_commands": next_cli_commands,
        "selection_reasons": selection_reasons,
        "tool_selection_policy": {
            "preferred_primary_tool": preferred_primary_tool,
            "preferred_secondary_tool": preferred_secondary_tool,
            "fallback_order": [preferred_primary_tool, preferred_secondary_tool, "shell"],
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract_payload(action: &str, project_id: Option<&str>) -> Value {
        decision_entrypoints_payload(action, project_id, "mcp", "cli")
    }

    fn tools(v: &Value) -> Vec<String> {
        v["next_mcp_tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap().to_string())
            .collect()
    }

    fn commands(v: &Value) -> Vec<String> {
        v["next_cli_commands"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap().to_string())
            .collect()
    }

    fn reasons(v: &Value) -> &Vec<Value> {
        v["selection_reasons"].as_array().unwrap()
    }

    // ── tool_selection_policy always present ─────────────────────────

    #[test]
    fn policy_fields_echo_preferred_tools() {
        let p = extract_payload("start_monitor", Some("proj1"));
        assert_eq!(p["tool_selection_policy"]["preferred_primary_tool"], "mcp");
        assert_eq!(p["tool_selection_policy"]["preferred_secondary_tool"], "cli");
        let fallback = p["tool_selection_policy"]["fallback_order"].as_array().unwrap();
        assert_eq!(fallback[0], "mcp");
        assert_eq!(fallback[1], "cli");
        assert_eq!(fallback[2], "shell");
    }

    // ── per-action tool lists ────────────────────────────────────────

    #[test]
    fn review_failing_verification_tools() {
        let p = extract_payload("review_failing_verification", Some("p1"));
        assert_eq!(
            tools(&p),
            vec![
                "get_verification_status",
                "run_verification_command",
                "get_data_risk_candidates",
            ]
        );
        let cmds = commands(&p);
        assert!(cmds.iter().any(|c| c.contains("verification")));
        assert!(cmds.iter().any(|c| c.contains("data-risk")));
        assert_eq!(reasons(&p).len(), 3);
    }

    #[test]
    fn stabilize_repository_tools() {
        let p = extract_payload("stabilize_repository_state", Some("p1"));
        assert!(tools(&p).contains(&"get_guidance".to_string()));
        assert!(tools(&p).contains(&"get_verification_status".to_string()));
        assert!(commands(&p).contains(&"git status".to_string()));
        assert!(commands(&p).contains(&"git diff".to_string()));
    }

    #[test]
    fn start_monitor_tools() {
        let p = extract_payload("start_monitor", Some("proj"));
        assert_eq!(tools(&p), vec!["start_monitor", "take_snapshot", "get_stats"]);
        let cmds = commands(&p);
        assert!(cmds.iter().all(|c| c.contains("--id proj")));
    }

    #[test]
    fn take_snapshot_tools() {
        let p = extract_payload("take_snapshot", Some("proj"));
        assert_eq!(tools(&p), vec!["take_snapshot", "get_stats", "get_unused_files"]);
    }

    #[test]
    fn generate_activity_then_stats_tools() {
        let p = extract_payload("generate_activity_then_stats", Some("x"));
        assert_eq!(tools(&p), vec!["get_stats", "get_guidance"]);
        assert!(commands(&p).contains(&"git status".to_string()));
    }

    #[test]
    fn run_verification_before_high_risk_tools() {
        let p = extract_payload("run_verification_before_high_risk_changes", Some("p"));
        assert_eq!(
            tools(&p),
            vec![
                "get_verification_status",
                "run_verification_command",
                "get_data_risk_candidates",
            ]
        );
    }

    #[test]
    fn review_unused_files_tools() {
        let p = extract_payload("review_unused_files", Some("p"));
        assert_eq!(
            tools(&p),
            vec!["get_unused_files", "get_verification_status", "get_data_risk_candidates"]
        );
    }

    #[test]
    fn inspect_hot_files_tools() {
        let p = extract_payload("inspect_hot_files", Some("p"));
        assert_eq!(
            tools(&p),
            vec!["get_stats", "get_verification_status", "get_data_risk_candidates"]
        );
    }

    // ── unknown action fallback ──────────────────────────────────────

    #[test]
    fn unknown_action_falls_back_to_triage() {
        let p = extract_payload("nonexistent_action", Some("p"));
        assert!(tools(&p).contains(&"get_guidance".to_string()));
        assert!(tools(&p).contains(&"list_projects".to_string()));
        assert!(commands(&p).contains(&"opendog agent-guidance".to_string()));
    }

    // ── project_id handling ──────────────────────────────────────────

    #[test]
    fn project_id_injected_into_cli_commands() {
        let p = extract_payload("start_monitor", Some("myproj"));
        for cmd in commands(&p) {
            assert!(
                cmd.contains("--id myproj"),
                "command missing --id myproj: {}",
                cmd
            );
        }
    }

    #[test]
    fn no_project_id_means_no_flag() {
        let p = extract_payload("start_monitor", None);
        for cmd in commands(&p) {
            assert!(
                !cmd.contains("--id"),
                "unexpected --id in command: {}",
                cmd
            );
        }
    }

    // ── selection_reasons structure ──────────────────────────────────

    #[test]
    fn selection_reasons_have_kind_target_why() {
        let p = extract_payload("review_failing_verification", Some("p"));
        for reason in reasons(&p) {
            assert!(reason["kind"].is_string(), "reason missing kind: {:?}", reason);
            assert!(reason["target"].is_string(), "reason missing target: {:?}", reason);
            assert!(reason["why"].is_string(), "reason missing why: {:?}", reason);
        }
    }
}
