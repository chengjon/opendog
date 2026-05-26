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
    assert_eq!(decision["signals"]["attention_score"].as_i64(), Some(85));
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
    let tools = payload["entrypoints"]["next_mcp_tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].as_str(), Some("get_stats"));
}

#[test]
fn empty_entrypoint_tools() {
    let payload = json!({"entrypoints": {"next_mcp_tools": []}});
    let tools = payload["entrypoints"]["next_mcp_tools"].as_array().unwrap();
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
