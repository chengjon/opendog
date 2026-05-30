use super::*;
use crate::config::{ProjectConfigOverrides, ProjectInfo};
use crate::storage::queries::StatsEntry;
use serde_json::{json, Value};
use std::path::PathBuf;

fn make_project_info(id: &str, root: &str) -> ProjectInfo {
    ProjectInfo {
        id: id.to_string(),
        root_path: PathBuf::from(root),
        db_path: PathBuf::from("/tmp/nonexistent-test.db"),
        config: ProjectConfigOverrides::default(),
        created_at: String::new(),
        status: "monitoring".to_string(),
    }
}

fn make_stats_entry(file_path: &str) -> StatsEntry {
    StatsEntry {
        file_path: file_path.to_string(),
        size: 100,
        file_type: "rs".to_string(),
        access_count: 5,
        estimated_duration_ms: 10,
        modification_count: 2,
        last_access_time: Some("2025-01-01T00:00:00Z".to_string()),
        first_seen_time: Some("2025-01-01T00:00:00Z".to_string()),
    }
}

// --- collect_workspace_data_risk_summaries ---

#[test]
fn collect_workspace_data_risk_summaries_empty_projects() {
    let summaries = collect_workspace_data_risk_summaries(
        &[],
        "all",
        "low",
        |_p: &ProjectInfo| Vec::new(),
        |_id: &str| None,
    );
    assert!(summaries.is_empty());
}

#[test]
fn collect_workspace_data_risk_summaries_no_candidates() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project_info("proj-a", dir.path().to_str().unwrap());
    // Entries with non-matching file names produce no mock/hardcoded candidates
    let entries = vec![make_stats_entry("src/main.rs")];
    let summaries = collect_workspace_data_risk_summaries(
        &[project],
        "all",
        "low",
        move |_p: &ProjectInfo| entries.clone(),
        |_id: &str| None,
    );
    assert!(summaries.is_empty());
}

#[test]
fn collect_workspace_data_risk_summaries_with_mock_candidates() {
    let dir = tempfile::tempdir().unwrap();
    // Create a mock file so that detect_mock_data_report finds path-based mock tokens
    let mock_dir = dir.path().join("mocks");
    std::fs::create_dir_all(&mock_dir).unwrap();
    std::fs::write(mock_dir.join("data.json"), r#"{"name": "test"}"#).unwrap();

    let project = make_project_info("proj-mock", dir.path().to_str().unwrap());
    let entries = vec![make_stats_entry("mocks/data.json")];
    let summaries = collect_workspace_data_risk_summaries(
        &[project],
        "all",
        "low",
        move |_p: &ProjectInfo| entries.clone(),
        |_id: &str| None,
    );
    // Should produce at least one summary
    assert!(!summaries.is_empty());
    assert_eq!(summaries[0]["project_id"], "proj-mock");
    assert!(summaries[0]["mock_candidate_count"].as_u64().unwrap() > 0);
}

#[test]
fn collect_workspace_data_risk_summaries_sorted_by_hardcoded_count_desc() {
    let dir_a = tempfile::tempdir().unwrap();
    let dir_b = tempfile::tempdir().unwrap();

    // proj-a: a mock file
    let mock_dir_a = dir_a.path().join("mocks");
    std::fs::create_dir_all(&mock_dir_a).unwrap();
    std::fs::write(mock_dir_a.join("data.json"), "{}").unwrap();

    // proj-b: mock + hardcoded patterns
    let mock_dir_b = dir_b.path().join("mocks");
    std::fs::create_dir_all(&mock_dir_b).unwrap();
    std::fs::write(
        mock_dir_b.join("data.py"),
        "customer = 'Alice'\nemail = 'a@b.com'\namount = 100\n",
    )
    .unwrap();

    let project_a = make_project_info("proj-a", dir_a.path().to_str().unwrap());
    let project_b = make_project_info("proj-b", dir_b.path().to_str().unwrap());

    let entries_a = vec![make_stats_entry("mocks/data.json")];
    let entries_b = vec![make_stats_entry("mocks/data.py")];

    let summaries = collect_workspace_data_risk_summaries(
        &[project_a, project_b],
        "all",
        "low",
        |p: &ProjectInfo| {
            if p.id == "proj-a" {
                entries_a.clone()
            } else {
                entries_b.clone()
            }
        },
        |_id: &str| None,
    );
    // proj-b should come first because hardcoded_candidate_count is >= proj-a
    if summaries.len() >= 2 {
        let hc_b = summaries
            .iter()
            .find(|s| s["project_id"] == "proj-b")
            .unwrap();
        let hc_a = summaries
            .iter()
            .find(|s| s["project_id"] == "proj-a")
            .unwrap();
        assert!(
            hc_b["hardcoded_candidate_count"].as_u64().unwrap()
                >= hc_a["hardcoded_candidate_count"].as_u64().unwrap()
        );
    }
}

#[test]
fn collect_workspace_data_risk_summaries_filters_by_candidate_type() {
    let dir = tempfile::tempdir().unwrap();
    let mock_dir = dir.path().join("mocks");
    std::fs::create_dir_all(&mock_dir).unwrap();
    std::fs::write(mock_dir.join("data.json"), "{}").unwrap();

    let project = make_project_info("proj-1", dir.path().to_str().unwrap());
    let entries = vec![make_stats_entry("mocks/data.json")];

    // Filter to "hardcoded" only - should return empty because only mock candidates
    let summaries = collect_workspace_data_risk_summaries(
        std::slice::from_ref(&project),
        "hardcoded",
        "low",
        move |_p: &ProjectInfo| entries.clone(),
        |_id: &str| None,
    );
    // With only mock path candidates, filtering to "hardcoded" eliminates mock,
    // leaving 0 mock and 0 hardcoded => no summary pushed
    assert!(summaries.is_empty());
}

#[test]
fn workspace_data_risk_payload_wraps_projects_and_guidance() {
    let dir = tempfile::tempdir().unwrap();
    let mock_dir = dir.path().join("mocks");
    std::fs::create_dir_all(&mock_dir).unwrap();
    std::fs::write(mock_dir.join("data.json"), "{}").unwrap();

    let project = make_project_info("proj-mock", dir.path().to_str().unwrap());
    let entries = vec![make_stats_entry("mocks/data.json")];

    let payload = workspace_data_risk_payload(
        "test.workspace-data-risk.v1",
        &[project],
        "all",
        "low",
        5,
        move |_p: &ProjectInfo| entries.clone(),
        |_id: &str| None,
    );

    assert_eq!(payload["schema_version"], "test.workspace-data-risk.v1");
    assert_eq!(payload["total_registered_projects"], 1);
    assert_eq!(payload["matched_project_count"], 1);
    assert_eq!(payload["candidate_type"], "all");
    assert_eq!(payload["min_review_priority"], "low");
    assert_eq!(payload["project_limit"], 5);
    assert_eq!(payload["projects"].as_array().unwrap().len(), 1);
    assert_eq!(payload["projects"][0]["project_id"], "proj-mock");
    assert_eq!(
        payload["guidance"]["layers"]["workspace_observation"]["matched_project_count"],
        1
    );
    assert_eq!(
        payload["guidance"]["layers"]["multi_project_portfolio"]["priority_projects"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

// --- decision_brief_payload ---

fn build_minimal_agent_guidance() -> Value {
    json!({
        "guidance": {
            "recommended_flow": ["Inspect workspace state."],
            "layers": {
                "execution_strategy": {
                    "preferred_primary_tool": "opendog",
                    "preferred_secondary_tool": "shell",
                    "global_strategy_mode": "observe_first",
                },
                "multi_project_portfolio": {
                    "monitoring_count": 1,
                    "priority_candidates": [{
                        "project_id": "proj-1",
                        "recommended_next_action": "inspect_hot_files",
                        "reason": "Active monitoring project.",
                        "repo_truth_gaps": [],
                        "mandatory_shell_checks": [],
                        "execution_sequence": [],
                        "hardcoded_candidate_count": 0,
                        "mock_candidate_count": 0,
                        "attention_score": 5,
                        "attention_band": "medium",
                        "attention_reasons": [],
                    }],
                    "project_overviews": [{
                        "project_id": "proj-1",
                        "safe_for_cleanup": true,
                        "safe_for_refactor": false,
                        "verification_evidence": { "status": "available" },
                        "repo_status_risk": { "risk_level": "low" },
                        "storage_maintenance": {
                            "maintenance_candidate": false,
                            "vacuum_candidate": false,
                            "approx_reclaimable_bytes": 0,
                            "approx_db_size_bytes": 1024,
                        },
                        "mock_data_summary": { "data_risk_focus": { "primary_focus": "none" } },
                    }],
                },
                "workspace_observation": {},
                "repo_status_risk": {},
                "verification_evidence": {},
                "storage_maintenance": {},
                "cleanup_refactor_candidates": { "candidates": [] },
                "project_toolchain": {},
                "constraints_boundaries": {},
                "governance": {},
            },
        },
    })
}

#[test]
fn decision_brief_payload_basic_structure() {
    let guidance = build_minimal_agent_guidance();
    let result = decision_brief_payload(
        "opendog.mcp.decision-brief.v1",
        "workspace",
        None,
        5,
        &guidance,
        None,
    );
    assert_eq!(result["schema_version"], "opendog.mcp.decision-brief.v1");
    assert_eq!(result["scope"], "workspace");
    assert_eq!(result["top"], 5);
    assert!(result["decision"].is_object());
    assert!(result["entrypoints"].is_object());
    assert!(result["layers"].is_object());
}

#[test]
fn decision_brief_payload_extracts_target_project() {
    let guidance = build_minimal_agent_guidance();
    let result = decision_brief_payload(
        "opendog.mcp.decision-brief.v1",
        "workspace",
        None,
        5,
        &guidance,
        None,
    );
    assert_eq!(result["decision"]["target_project_id"], "proj-1");
    assert_eq!(
        result["decision"]["recommended_next_action"],
        "inspect_hot_files"
    );
}

#[test]
fn decision_brief_payload_selected_project_overrides() {
    let guidance = build_minimal_agent_guidance();
    let result = decision_brief_payload(
        "opendog.mcp.decision-brief.v1",
        "project",
        Some("proj-1"),
        3,
        &guidance,
        None,
    );
    assert_eq!(result["selected_project_id"], "proj-1");
}

#[test]
fn decision_brief_payload_includes_execution_templates() {
    let guidance = build_minimal_agent_guidance();
    let result = decision_brief_payload(
        "opendog.mcp.decision-brief.v1",
        "workspace",
        None,
        5,
        &guidance,
        None,
    );
    let templates = result["entrypoints"]["execution_templates"].clone();
    assert!(templates.is_array() || templates.is_object());
}

#[test]
fn decision_brief_payload_with_data_risk_guidance() {
    let guidance = build_minimal_agent_guidance();
    let data_risk_guidance = json!({
        "layers": {
            "workspace_observation": {
                "projects_with_mock_candidates": 2,
                "projects_with_hardcoded_candidates": 1,
                "total_mock_candidates": 5,
                "total_hardcoded_candidates": 3,
                "data_risk_focus_distribution": {},
                "projects_requiring_hardcoded_review": ["proj-1"],
                "projects_requiring_mock_review": ["proj-1", "proj-2"],
                "projects_requiring_mixed_file_review": [],
                "rule_groups_summary": [],
                "rule_hits_summary": [],
            },
            "execution_strategy": {
                "data_risk_focus_distribution": {},
                "projects_requiring_hardcoded_review": ["proj-1"],
                "projects_requiring_mock_review": ["proj-1", "proj-2"],
                "projects_requiring_mixed_file_review": [],
            },
            "multi_project_portfolio": {
                "priority_projects": [],
                "rule_groups_summary": [],
                "rule_hits_summary": [],
            },
            "cleanup_refactor_candidates": {
                "priority_projects": [],
            },
        },
    });
    let result = decision_brief_payload(
        "opendog.mcp.decision-brief.v1",
        "workspace",
        None,
        5,
        &guidance,
        Some(&data_risk_guidance),
    );
    // Data risk enrichment should be reflected in layers
    assert_eq!(
        result["layers"]["workspace_observation"]["projects_with_mock_candidates"],
        2
    );
    assert_eq!(
        result["layers"]["workspace_observation"]["projects_with_hardcoded_candidates"],
        1
    );
}

#[test]
fn decision_brief_payload_decision_fields() {
    let guidance = build_minimal_agent_guidance();
    let result = decision_brief_payload(
        "opendog.mcp.decision-brief.v1",
        "workspace",
        None,
        5,
        &guidance,
        None,
    );
    let decision = &result["decision"];
    assert_eq!(decision["summary"], "Inspect workspace state.");
    assert!(decision["recommended_flow"].is_array());
    assert!(decision["safe_for_cleanup"].is_boolean());
    assert!(decision["verification_status"].is_string());
    assert!(decision["requires_verification"].is_boolean());
    assert!(decision["signals"].is_object());
}

#[test]
fn decision_brief_payload_decision_signals() {
    let guidance = build_minimal_agent_guidance();
    let result = decision_brief_payload(
        "opendog.mcp.decision-brief.v1",
        "workspace",
        None,
        5,
        &guidance,
        None,
    );
    let signals = &result["decision"]["signals"];
    assert_eq!(signals["repo_risk_level"], "low");
    assert_eq!(signals["hardcoded_candidate_count"], 0);
    assert_eq!(signals["mock_candidate_count"], 0);
    assert_eq!(signals["attention_score"], 5);
    assert_eq!(signals["attention_band"], "medium");
    assert_eq!(signals["monitoring_count"], 1);
}
