use serde_json::Value;

mod model;

use model::DecisionEntrypointsPlan;

pub(in crate::mcp) fn decision_entrypoints_payload(
    action: &str,
    project_id: Option<&str>,
    preferred_primary_tool: &str,
    preferred_secondary_tool: &str,
) -> Value {
    DecisionEntrypointsPlan::from_action(action, project_id)
        .to_json(preferred_primary_tool, preferred_secondary_tool)
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
        assert_eq!(
            p["tool_selection_policy"]["preferred_secondary_tool"],
            "cli"
        );
        let fallback = p["tool_selection_policy"]["fallback_order"]
            .as_array()
            .unwrap();
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
        assert_eq!(
            tools(&p),
            vec!["start_monitor", "take_snapshot", "get_stats"]
        );
        let cmds = commands(&p);
        assert!(cmds.iter().all(|c| c.contains("--id proj")));
    }

    #[test]
    fn take_snapshot_tools() {
        let p = extract_payload("take_snapshot", Some("proj"));
        assert_eq!(
            tools(&p),
            vec!["take_snapshot", "get_stats", "get_unused_files"]
        );
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
            vec![
                "get_unused_files",
                "get_verification_status",
                "get_data_risk_candidates"
            ]
        );
    }

    #[test]
    fn inspect_hot_files_tools() {
        let p = extract_payload("inspect_hot_files", Some("p"));
        assert_eq!(
            tools(&p),
            vec![
                "get_stats",
                "get_verification_status",
                "get_data_risk_candidates"
            ]
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
            assert!(!cmd.contains("--id"), "unexpected --id in command: {}", cmd);
        }
    }

    // ── selection_reasons structure ──────────────────────────────────

    #[test]
    fn selection_reasons_have_kind_target_why() {
        let p = extract_payload("review_failing_verification", Some("p"));
        for reason in reasons(&p) {
            assert!(
                reason["kind"].is_string(),
                "reason missing kind: {:?}",
                reason
            );
            assert!(
                reason["target"].is_string(),
                "reason missing target: {:?}",
                reason
            );
            assert!(
                reason["why"].is_string(),
                "reason missing why: {:?}",
                reason
            );
        }
    }
}
