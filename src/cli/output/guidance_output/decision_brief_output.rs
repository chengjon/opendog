use serde_json::Value;

use super::print_recommended_flow;

pub(super) fn print_decision_brief(payload: &Value) {
    println!(
        "Decision brief — scope={} top={}",
        payload["scope"].as_str().unwrap_or("-"),
        payload["top"].as_u64().unwrap_or(0),
    );

    let decision = &payload["decision"];
    println!(
        "  Next action: {} | primary={} secondary={}",
        decision["recommended_next_action"].as_str().unwrap_or("-"),
        decision["preferred_primary_tool"].as_str().unwrap_or("-"),
        decision["preferred_secondary_tool"].as_str().unwrap_or("-"),
    );
    if let Some(project_id) = decision["target_project_id"].as_str() {
        println!("  Target project: {}", project_id);
    }
    if let Some(summary) = decision["summary"].as_str() {
        println!("  Summary: {}", summary);
    }
    println!(
        "  Action profile: class={} phase={} verification_required={}",
        decision["action_profile"]["action_class"]
            .as_str()
            .unwrap_or("-"),
        decision["action_profile"]["phase"].as_str().unwrap_or("-"),
        decision["action_profile"]["verification_required"]
            .as_bool()
            .unwrap_or(false),
    );
    println!(
        "  Risk profile: tier={} repo_risk={} manual_review={}",
        decision["risk_profile"]["risk_tier"]
            .as_str()
            .unwrap_or("-"),
        decision["risk_profile"]["repo_risk_level"]
            .as_str()
            .unwrap_or("-"),
        decision["risk_profile"]["manual_review_required"]
            .as_bool()
            .unwrap_or(false),
    );
    if let Some(primary_repo_risk) =
        decision["risk_profile"]["primary_repo_risk_finding"].as_object()
    {
        println!(
            "  Repo risk focus: {} [{} / {}]",
            primary_repo_risk
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("-"),
            primary_repo_risk
                .get("severity")
                .and_then(Value::as_str)
                .unwrap_or("-"),
            primary_repo_risk
                .get("priority")
                .and_then(Value::as_str)
                .unwrap_or("-"),
        );
        if let Some(summary) = primary_repo_risk.get("summary").and_then(Value::as_str) {
            println!("  Repo risk reason: {}", summary);
        }
    }
    println!(
        "  Attention: score={} band={}",
        decision["signals"]["attention_score"].as_i64().unwrap_or(0),
        decision["signals"]["attention_band"]
            .as_str()
            .unwrap_or("-"),
    );
    let attention_reasons = decision["signals"]["attention_reasons"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if let Some(primary_attention_reason) = attention_reasons.first() {
        if let Some(text) = primary_attention_reason.as_str() {
            println!("  Attention reason: {}", text);
        }
    }
    if decision["signals"]["storage_maintenance_candidate"]
        .as_bool()
        .unwrap_or(false)
    {
        println!(
            "  Storage maintenance: candidate=true vacuum_candidate={} reclaimable_bytes={}",
            decision["signals"]["storage_vacuum_candidate"]
                .as_bool()
                .unwrap_or(false),
            decision["signals"]["storage_reclaimable_bytes"]
                .as_i64()
                .unwrap_or(0),
        );
    }
    print_recommended_flow(&decision["recommended_flow"]);

    println!();
    println!("Suggested MCP tools:");
    let next_tools = payload["entrypoints"]["next_mcp_tools"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if next_tools.is_empty() {
        println!("  None");
    } else {
        for tool in next_tools {
            if let Some(name) = tool.as_str() {
                println!("  {}", name);
            }
        }
    }

    println!();
    println!("Suggested CLI commands:");
    let next_commands = payload["entrypoints"]["next_cli_commands"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if next_commands.is_empty() {
        println!("  None");
    } else {
        for command in next_commands {
            if let Some(text) = command.as_str() {
                println!("  {}", text);
            }
        }
    }

    let selection_reasons = payload["entrypoints"]["selection_reasons"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !selection_reasons.is_empty() {
        println!();
        println!("Why these entrypoints:");
        for item in selection_reasons.iter().take(3) {
            println!(
                "  {} [{}] {}",
                item["target"].as_str().unwrap_or("-"),
                item["kind"].as_str().unwrap_or("-"),
                item["why"].as_str().unwrap_or("-"),
            );
        }
    }

    let execution_templates = payload["entrypoints"]["execution_templates"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !execution_templates.is_empty() {
        println!();
        println!("Execution templates:");
        for item in execution_templates.iter().take(2) {
            println!(
                "  {} [{}] priority={} stage={}",
                item["template_id"].as_str().unwrap_or("-"),
                item["kind"].as_str().unwrap_or("-"),
                item["priority"].as_u64().unwrap_or(0),
                item["plan_stage"].as_str().unwrap_or("-"),
            );
            if let Some(tool) = item["tool"].as_str() {
                println!("    tool: {}", tool);
            }
            if let Some(command) = item["command_template"].as_str() {
                println!("    command: {}", command);
            }
            if let Some(signal) = item["success_signal"].as_str() {
                println!("    success: {}", signal);
            }
            println!(
                "    parallel={} human_confirmation={} evidence_to_opendog={}",
                item["can_run_in_parallel"].as_bool().unwrap_or(false),
                item["requires_human_confirmation"]
                    .as_bool()
                    .unwrap_or(false),
                item["evidence_written_to_opendog"]
                    .as_bool()
                    .unwrap_or(false),
            );
            if let Some(terminality) = item["terminality"].as_str() {
                println!("    terminality: {}", terminality);
            }
            if let Some(defaults) = item["default_values"].as_object() {
                if !defaults.is_empty() {
                    println!(
                        "    defaults: {}",
                        serde_json::Value::Object(defaults.clone())
                    );
                }
            }
            if let Some(hints) = item["placeholder_hints"].as_array() {
                if !hints.is_empty() {
                    println!(
                        "    placeholders: {}",
                        hints[0]["placeholder"].as_str().unwrap_or("-")
                    );
                }
            }
            if let Some(conditions) = item["should_run_if"].as_array() {
                if !conditions.is_empty() {
                    println!(
                        "    should_run_if: {}",
                        conditions[0].as_str().unwrap_or("-")
                    );
                }
            }
            if let Some(conditions) = item["skip_if"].as_array() {
                if !conditions.is_empty() {
                    println!("    skip_if: {}", conditions[0].as_str().unwrap_or("-"));
                }
            }
            if let Some(fields) = item["expected_output_fields"].as_array() {
                if !fields.is_empty() {
                    println!("    expected_output: {}", fields[0].as_str().unwrap_or("-"));
                }
            }
            if let Some(followups) = item["follow_up_on_success"].as_array() {
                if !followups.is_empty() {
                    println!("    on_success: {}", followups[0].as_str().unwrap_or("-"));
                }
            }
            if let Some(followups) = item["follow_up_on_failure"].as_array() {
                if !followups.is_empty() {
                    println!("    on_failure: {}", followups[0].as_str().unwrap_or("-"));
                }
            }
            if item["retry_policy"].is_object() {
                println!("    retry_policy: {}", item["retry_policy"]);
            }
        }
    }

    let layers = &payload["layers"];
    let observation = &layers["workspace_observation"];
    println!();
    println!(
        "Workspace observation: status={} analysis_state={}",
        observation["status"].as_str().unwrap_or("-"),
        observation["analysis_state"].as_str().unwrap_or("-"),
    );
    println!(
        "  snapshot_missing={} snapshot_stale={} activity_missing={} activity_stale={} verification_missing={} verification_stale={}",
        observation["projects_missing_snapshot"].as_u64().unwrap_or(0),
        observation["projects_with_stale_snapshot"]
            .as_u64()
            .unwrap_or(0),
        observation["projects_missing_activity"].as_u64().unwrap_or(0),
        observation["projects_with_stale_activity"]
            .as_u64()
            .unwrap_or(0),
        observation["projects_missing_verification"]
            .as_u64()
            .unwrap_or(0),
        observation["projects_with_stale_verification"]
            .as_u64()
            .unwrap_or(0),
    );
    println!(
        "  monitoring={} hardcoded_projects={} total_hardcoded={}",
        layers["multi_project_portfolio"]["monitoring_count"]
            .as_u64()
            .unwrap_or(0),
        observation["projects_with_hardcoded_candidates"]
            .as_u64()
            .unwrap_or(0),
        observation["total_hardcoded_candidates"]
            .as_u64()
            .unwrap_or(0),
    );

    let strategy = &layers["execution_strategy"];
    println!(
        "Execution strategy: mode={} evidence_priority={}",
        strategy["global_strategy_mode"].as_str().unwrap_or("-"),
        strategy["evidence_priority"].as_str().unwrap_or("-"),
    );

    let matched_overview = layers["multi_project_portfolio"]["project_overviews"]
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .find(|item| item["project_id"].as_str() == decision["target_project_id"].as_str())
        });
    let repo_risk = matched_overview
        .map(|project| &project["repo_status_risk"])
        .unwrap_or(&layers["repo_status_risk"]);
    println!(
        "Repo risk: status={} level={} dirty={}",
        repo_risk["status"].as_str().unwrap_or("-"),
        repo_risk["risk_level"].as_str().unwrap_or("-"),
        repo_risk["is_dirty"].as_bool().unwrap_or(false),
    );
    println!(
        "Repo findings: total={} high={} medium={} low={}",
        decision["risk_profile"]["repo_risk_finding_counts"]["total"]
            .as_u64()
            .unwrap_or(0),
        decision["risk_profile"]["repo_risk_finding_counts"]["high"]
            .as_u64()
            .unwrap_or(0),
        decision["risk_profile"]["repo_risk_finding_counts"]["medium"]
            .as_u64()
            .unwrap_or(0),
        decision["risk_profile"]["repo_risk_finding_counts"]["low"]
            .as_u64()
            .unwrap_or(0),
    );
    let top_changed_directories = repo_risk["top_changed_directories"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !top_changed_directories.is_empty() {
        let hotspots = top_changed_directories
            .iter()
            .take(3)
            .map(|item| {
                format!(
                    "{}({})",
                    item["directory"].as_str().unwrap_or("-"),
                    item["changed_files"].as_u64().unwrap_or(0)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        println!("Repo hotspots: {}", hotspots);
    }
    let operation_states = repo_risk["operation_states"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !operation_states.is_empty() {
        println!(
            "Repo operations: {}",
            operation_states
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let verification = &layers["verification_evidence"];
    println!(
        "Verification evidence: recorded={} missing={} failing={} stale={} confidence={}",
        verification["projects_with_recorded_verification"]
            .as_u64()
            .unwrap_or(0),
        verification["projects_missing_verification"]
            .as_u64()
            .unwrap_or(0),
        verification["projects_with_failing_verification"]
            .as_u64()
            .unwrap_or(0),
        verification["projects_with_stale_verification"]
            .as_u64()
            .unwrap_or(0),
        verification["confidence"].as_str().unwrap_or("-"),
    );
    if let Some(project) = matched_overview {
        println!(
            "  Target verification: project={} status={} freshness={} cleanup_safe={} refactor_safe={}",
            project["project_id"].as_str().unwrap_or("-"),
            project["verification_evidence"]["status"]
                .as_str()
                .unwrap_or("-"),
            project["observation"]["freshness"]["verification"]["status"]
                .as_str()
                .unwrap_or("-"),
            project["safe_for_cleanup"].as_bool().unwrap_or(false),
            project["safe_for_refactor"].as_bool().unwrap_or(false),
        );
        println!(
            "  Target observation: project={} coverage={} snapshot={} activity={} verification={}",
            project["project_id"].as_str().unwrap_or("-"),
            project["observation"]["coverage_state"]
                .as_str()
                .unwrap_or("-"),
            project["observation"]["freshness"]["snapshot"]["status"]
                .as_str()
                .unwrap_or("-"),
            project["observation"]["freshness"]["activity"]["status"]
                .as_str()
                .unwrap_or("-"),
            project["observation"]["freshness"]["verification"]["status"]
                .as_str()
                .unwrap_or("-"),
        );
    }

    let toolchain = &layers["project_toolchain"];
    if let Some(project) = matched_overview {
        println!(
            "Toolchain: project={} type={} confidence={}",
            project["project_id"].as_str().unwrap_or("-"),
            project["project_toolchain"]["project_type"]
                .as_str()
                .unwrap_or("-"),
            project["project_toolchain"]["confidence"]
                .as_str()
                .unwrap_or("-"),
        );
    } else {
        println!(
            "Toolchain: status={} known_types={} unknown_projects={}",
            toolchain["status"].as_str().unwrap_or("-"),
            toolchain["known_project_types"].as_u64().unwrap_or(0),
            toolchain["projects_without_detected_toolchain"]
                .as_u64()
                .unwrap_or(0),
        );
    }

    let signals = &decision["signals"];
    println!(
        "Signals: repo_risk={} dirty={} hardcoded={} mock={}",
        signals["repo_risk_level"].as_str().unwrap_or("-"),
        signals["repo_is_dirty"].as_bool().unwrap_or(false),
        signals["hardcoded_candidate_count"].as_u64().unwrap_or(0),
        signals["mock_candidate_count"].as_u64().unwrap_or(0),
    );
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn decision_brief_header_format() {
        let payload = json!({"scope": "workspace", "top": 3});
        let line = format!(
            "Decision brief — scope={} top={}",
            payload["scope"].as_str().unwrap_or("-"),
            payload["top"].as_u64().unwrap_or(0)
        );
        assert_eq!(line, "Decision brief — scope=workspace top=3");
    }

    #[test]
    fn decision_next_action_format() {
        let decision = json!({
            "recommended_next_action": "run_verification",
            "preferred_primary_tool": "opendog",
            "preferred_secondary_tool": "shell"
        });
        let line = format!(
            "  Next action: {} | primary={} secondary={}",
            decision["recommended_next_action"].as_str().unwrap_or("-"),
            decision["preferred_primary_tool"].as_str().unwrap_or("-"),
            decision["preferred_secondary_tool"].as_str().unwrap_or("-")
        );
        assert!(line.contains("run_verification"));
        assert!(line.contains("opendog"));
        assert!(line.contains("shell"));
    }

    #[test]
    fn target_project_id_present() {
        let decision = json!({"target_project_id": "proj1"});
        let has_target = decision["target_project_id"].as_str().is_some();
        assert!(has_target);
        assert_eq!(decision["target_project_id"].as_str(), Some("proj1"));
    }

    #[test]
    fn target_project_id_absent() {
        let decision = json!({"target_project_id": null});
        let has_target = decision["target_project_id"].as_str().is_some();
        assert!(!has_target);
    }

    #[test]
    fn action_profile_format() {
        let decision = json!({
            "action_profile": {
                "action_class": "verification",
                "phase": "pre_edit",
                "verification_required": true
            }
        });
        let ap = &decision["action_profile"];
        assert_eq!(ap["action_class"].as_str(), Some("verification"));
        assert!(ap["verification_required"].as_bool().unwrap());
    }

    #[test]
    fn risk_profile_format() {
        let decision = json!({
            "risk_profile": {
                "risk_tier": "high",
                "repo_risk_level": "critical",
                "manual_review_required": true
            }
        });
        let rp = &decision["risk_profile"];
        assert_eq!(rp["risk_tier"].as_str(), Some("high"));
        assert!(rp["manual_review_required"].as_bool().unwrap());
    }

    #[test]
    fn repo_risk_finding_present() {
        let decision = json!({
            "risk_profile": {
                "primary_repo_risk_finding": {
                    "kind": "uncommitted_changes",
                    "severity": "high",
                    "priority": "P0",
                    "summary": "Working tree has uncommitted changes"
                }
            }
        });
        let finding = &decision["risk_profile"]["primary_repo_risk_finding"];
        assert!(finding.is_object());
        assert_eq!(finding["kind"].as_str(), Some("uncommitted_changes"));
        assert_eq!(finding["severity"].as_str(), Some("high"));
    }

    #[test]
    fn attention_signals_format() {
        let decision = json!({
            "signals": {
                "attention_score": 85,
                "attention_band": "critical",
                "attention_reasons": ["failing verification", "stale snapshot"]
            }
        });
        assert_eq!(
            decision["signals"]["attention_score"].as_i64(),
            Some(85)
        );
        assert_eq!(
            decision["signals"]["attention_band"].as_str(),
            Some("critical")
        );
    }

    #[test]
    fn storage_maintenance_candidate_true() {
        let decision = json!({
            "signals": {
                "storage_maintenance_candidate": true,
                "storage_vacuum_candidate": false,
                "storage_reclaimable_bytes": 4096
            }
        });
        assert!(decision["signals"]["storage_maintenance_candidate"]
            .as_bool()
            .unwrap());
    }

    #[test]
    fn storage_maintenance_candidate_false_skipped() {
        let decision = json!({"signals": {"storage_maintenance_candidate": false}});
        assert!(!decision["signals"]["storage_maintenance_candidate"]
            .as_bool()
            .unwrap());
    }

    #[test]
    fn entrypoint_tools_list() {
        let payload = json!({
            "entrypoints": {
                "next_mcp_tools": ["get_stats", "get_verification_status"],
                "next_cli_commands": ["opendog stats --id proj"]
            }
        });
        let tools = payload["entrypoints"]["next_mcp_tools"]
            .as_array()
            .unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].as_str(), Some("get_stats"));
    }

    #[test]
    fn empty_entrypoint_tools() {
        let payload = json!({"entrypoints": {"next_mcp_tools": []}});
        let tools = payload["entrypoints"]["next_mcp_tools"]
            .as_array()
            .unwrap();
        assert!(tools.is_empty());
    }

    #[test]
    fn selection_reasons_format() {
        let payload = json!({
            "entrypoints": {
                "selection_reasons": [
                    {"target": "get_stats", "kind": "mcp", "why": "check activity"}
                ]
            }
        });
        let reasons = payload["entrypoints"]["selection_reasons"]
            .as_array()
            .unwrap();
        assert_eq!(reasons[0]["target"].as_str(), Some("get_stats"));
    }

    #[test]
    fn repo_risk_finding_counts() {
        let decision = json!({
            "risk_profile": {
                "repo_risk_finding_counts": {"total": 5, "high": 1, "medium": 2, "low": 2}
            }
        });
        let counts = &decision["risk_profile"]["repo_risk_finding_counts"];
        assert_eq!(counts["total"].as_u64(), Some(5));
    }

    #[test]
    fn top_changed_directories_format() {
        let repo_risk = json!({
            "top_changed_directories": [
                {"directory": "src/", "changed_files": 10},
                {"directory": "tests/", "changed_files": 3}
            ]
        });
        let dirs = repo_risk["top_changed_directories"].as_array().unwrap();
        let hotspots: Vec<String> = dirs
            .iter()
            .take(3)
            .map(|item| {
                format!(
                    "{}({})",
                    item["directory"].as_str().unwrap_or("-"),
                    item["changed_files"].as_u64().unwrap_or(0)
                )
            })
            .collect();
        assert_eq!(hotspots[0], "src/(10)");
        assert_eq!(hotspots[1], "tests/(3)");
    }

    #[test]
    fn operation_states_format() {
        let repo_risk = json!({"operation_states": ["merge", "rebase"]});
        let ops: Vec<&str> = repo_risk["operation_states"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        assert_eq!(ops, vec!["merge", "rebase"]);
    }

    #[test]
    fn matched_overview_safe_flags() {
        let project = json!({
            "project_id": "proj1",
            "safe_for_cleanup": true,
            "safe_for_refactor": false
        });
        assert!(project["safe_for_cleanup"].as_bool().unwrap());
        assert!(!project["safe_for_refactor"].as_bool().unwrap());
    }
}
