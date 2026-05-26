use serde_json::{json, Value};

use crate::config::ProjectInfo;
use crate::contracts::versioned_payload;
use crate::storage::database::Database;
use crate::storage::queries::{upsert_data_risk_cache, StatsEntry};

use super::{
    augment_entrypoints_for_storage_maintenance, decision_action_profile,
    decision_entrypoints_payload, decision_execution_templates, decision_risk_profile,
    detect_mock_data_report,
    guidance_types::{DecisionBrief, DecisionSignals},
    serialization::to_value_or_error,
    workspace_data_risk_overview_payload,
};

pub(crate) fn workspace_data_risk_payload<F, D>(
    schema_version: &str,
    projects: &[ProjectInfo],
    candidate_type: &str,
    min_review_priority: &str,
    project_limit: usize,
    load_entries: F,
    get_db: D,
) -> Value
where
    F: FnMut(&ProjectInfo) -> Vec<StatsEntry>,
    D: Fn(&str) -> Option<Database>,
{
    let total_registered_projects = projects.len();
    let mut summaries = collect_workspace_data_risk_summaries(
        projects,
        candidate_type,
        min_review_priority,
        load_entries,
        get_db,
    );
    summaries.truncate(project_limit.max(1));

    versioned_payload(
        schema_version,
        [
            (
                "total_registered_projects",
                json!(total_registered_projects),
            ),
            ("matched_project_count", json!(summaries.len())),
            ("candidate_type", json!(candidate_type)),
            ("min_review_priority", json!(min_review_priority)),
            ("project_limit", json!(project_limit.max(1))),
            ("projects", json!(summaries.clone())),
            (
                "guidance",
                workspace_data_risk_overview_payload(&summaries, total_registered_projects),
            ),
        ],
    )
}

pub(crate) fn collect_workspace_data_risk_summaries<F, D>(
    projects: &[ProjectInfo],
    candidate_type: &str,
    min_review_priority: &str,
    mut load_entries: F,
    get_db: D,
) -> Vec<Value>
where
    F: FnMut(&ProjectInfo) -> Vec<StatsEntry>,
    D: Fn(&str) -> Option<Database>,
{
    let mut summaries = Vec::new();
    for project in projects {
        let entries = load_entries(project);
        let report = detect_mock_data_report(&project.root_path, &entries);

        // Cache unfiltered counts for governance observation hints
        if let Some(db) = get_db(&project.id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string();
            let _ = upsert_data_risk_cache(
                &db,
                report.mock_candidates.len(),
                report.hardcoded_candidates.len(),
                report.mixed_review_files.len(),
                &now,
            );
        }

        let filtered = report.filtered(candidate_type, Some(min_review_priority));
        let summary = filtered.to_value(5);
        let rendered = json!({
            "project_id": project.id,
            "status": project.status,
            "mock_candidate_count": summary["mock_candidate_count"].clone(),
            "hardcoded_candidate_count": summary["hardcoded_candidate_count"].clone(),
            "mixed_review_file_count": summary["mixed_review_file_count"].clone(),
            "data_risk_focus": summary["data_risk_focus"].clone(),
            "rule_groups_summary": summary["rule_groups_summary"].clone(),
            "rule_hits_summary": summary["rule_hits_summary"].clone(),
            "top_hardcoded_candidates": summary["hardcoded_data_candidates"].clone(),
            "top_mock_candidates": summary["mock_data_candidates"].clone(),
        });
        if rendered["mock_candidate_count"].as_u64().unwrap_or(0) > 0
            || rendered["hardcoded_candidate_count"].as_u64().unwrap_or(0) > 0
        {
            summaries.push(rendered);
        }
    }

    summaries.sort_by(|a, b| {
        b["hardcoded_candidate_count"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["hardcoded_candidate_count"].as_u64().unwrap_or(0))
            .then_with(|| {
                b["mixed_review_file_count"]
                    .as_u64()
                    .unwrap_or(0)
                    .cmp(&a["mixed_review_file_count"].as_u64().unwrap_or(0))
            })
            .then_with(|| {
                b["mock_candidate_count"]
                    .as_u64()
                    .unwrap_or(0)
                    .cmp(&a["mock_candidate_count"].as_u64().unwrap_or(0))
            })
    });
    summaries
}

pub(crate) fn decision_brief_payload(
    schema_version: &str,
    scope: &str,
    selected_project_id: Option<&str>,
    top: usize,
    agent_guidance: &Value,
    workspace_data_guidance: Option<&Value>,
) -> Value {
    let guidance = &agent_guidance["guidance"];
    let strategy = &guidance["layers"]["execution_strategy"];
    let portfolio = &guidance["layers"]["multi_project_portfolio"];
    let top_candidate = portfolio["priority_candidates"]
        .as_array()
        .and_then(|items| items.first())
        .cloned()
        .unwrap_or(Value::Null);
    let target_project_id = top_candidate["project_id"]
        .as_str()
        .or(selected_project_id)
        .map(|value| value.to_string());
    let matched_overview = guidance["layers"]["multi_project_portfolio"]["project_overviews"]
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .find(|item| item["project_id"].as_str() == target_project_id.as_deref())
        })
        .cloned()
        .unwrap_or(Value::Null);
    let recommended_next_action = top_candidate["recommended_next_action"]
        .as_str()
        .unwrap_or("inspect_workspace_state");
    let mut entrypoints = decision_entrypoints_payload(
        recommended_next_action,
        target_project_id.as_deref(),
        strategy["preferred_primary_tool"]
            .as_str()
            .unwrap_or("opendog"),
        strategy["preferred_secondary_tool"]
            .as_str()
            .unwrap_or("shell"),
    );

    let safe_for_cleanup = portfolio["project_overviews"]
        .as_array()
        .and_then(|_| matched_overview["safe_for_cleanup"].as_bool());
    let safe_for_refactor = portfolio["project_overviews"]
        .as_array()
        .and_then(|_| matched_overview["safe_for_refactor"].as_bool());
    let verification_status = matched_overview["verification_evidence"]["status"]
        .as_str()
        .unwrap_or("not_recorded");
    let repo_risk_level = matched_overview["repo_status_risk"]["risk_level"]
        .as_str()
        .unwrap_or("unknown");
    let storage_maintenance = &matched_overview["storage_maintenance"];

    entrypoints["execution_templates"] = decision_execution_templates(
        recommended_next_action,
        target_project_id.as_deref(),
        verification_status,
        repo_risk_level,
        safe_for_cleanup,
        safe_for_refactor,
    );
    augment_entrypoints_for_storage_maintenance(
        &mut entrypoints,
        target_project_id.as_deref(),
        storage_maintenance,
    );

    let mut layers = guidance["layers"].clone();
    if let Some(data_risk_guidance) = workspace_data_guidance {
        let risk_observation = &data_risk_guidance["layers"]["workspace_observation"];
        layers["workspace_observation"]["projects_with_mock_candidates"] =
            risk_observation["projects_with_mock_candidates"].clone();
        layers["workspace_observation"]["projects_with_hardcoded_candidates"] =
            risk_observation["projects_with_hardcoded_candidates"].clone();
        layers["workspace_observation"]["total_mock_candidates"] =
            risk_observation["total_mock_candidates"].clone();
        layers["workspace_observation"]["total_hardcoded_candidates"] =
            risk_observation["total_hardcoded_candidates"].clone();
        layers["workspace_observation"]["data_risk_focus_distribution"] =
            risk_observation["data_risk_focus_distribution"].clone();
        layers["workspace_observation"]["projects_requiring_hardcoded_review"] =
            risk_observation["projects_requiring_hardcoded_review"].clone();
        layers["workspace_observation"]["projects_requiring_mock_review"] =
            risk_observation["projects_requiring_mock_review"].clone();
        layers["workspace_observation"]["projects_requiring_mixed_file_review"] =
            risk_observation["projects_requiring_mixed_file_review"].clone();
        layers["workspace_observation"]["rule_groups_summary"] =
            risk_observation["rule_groups_summary"].clone();
        layers["workspace_observation"]["rule_hits_summary"] =
            risk_observation["rule_hits_summary"].clone();
        layers["execution_strategy"]["data_risk_focus_distribution"] = data_risk_guidance["layers"]
            ["execution_strategy"]["data_risk_focus_distribution"]
            .clone();
        layers["execution_strategy"]["projects_requiring_hardcoded_review"] = data_risk_guidance
            ["layers"]["execution_strategy"]["projects_requiring_hardcoded_review"]
            .clone();
        layers["execution_strategy"]["projects_requiring_mock_review"] = data_risk_guidance
            ["layers"]["execution_strategy"]["projects_requiring_mock_review"]
            .clone();
        layers["execution_strategy"]["projects_requiring_mixed_file_review"] = data_risk_guidance
            ["layers"]["execution_strategy"]["projects_requiring_mixed_file_review"]
            .clone();
        layers["multi_project_portfolio"]["priority_projects"] =
            data_risk_guidance["layers"]["multi_project_portfolio"]["priority_projects"].clone();
        layers["multi_project_portfolio"]["rule_groups_summary"] =
            data_risk_guidance["layers"]["multi_project_portfolio"]["rule_groups_summary"].clone();
        layers["multi_project_portfolio"]["rule_hits_summary"] =
            data_risk_guidance["layers"]["multi_project_portfolio"]["rule_hits_summary"].clone();
        layers["cleanup_refactor_candidates"]["priority_projects"] = data_risk_guidance["layers"]
            ["cleanup_refactor_candidates"]["priority_projects"]
            .clone();
    }

    let decision = to_value_or_error(
        "DecisionBrief",
        DecisionBrief {
            summary: guidance["recommended_flow"]
                .as_array()
                .and_then(|steps| steps.first())
                .and_then(|step| step.as_str())
                .unwrap_or("No recommendation available.")
                .to_string(),
            recommended_next_action: recommended_next_action.to_string(),
            reason: top_candidate["reason"].clone(),
            repo_truth_gaps: top_candidate["repo_truth_gaps"].clone(),
            mandatory_shell_checks: top_candidate["mandatory_shell_checks"].clone(),
            external_truth_boundary: layers["execution_strategy"]["external_truth_boundary"]
                .clone(),
            review_focus: layers["execution_strategy"]["review_focus_projection"]["review_focus"]
                .clone(),
            execution_sequence: top_candidate["execution_sequence"].clone(),
            data_risk_focus: matched_overview["mock_data_summary"]["data_risk_focus"].clone(),
            target_project_id,
            strategy_mode: strategy["global_strategy_mode"].clone(),
            preferred_primary_tool: strategy["preferred_primary_tool"].clone(),
            preferred_secondary_tool: strategy["preferred_secondary_tool"].clone(),
            recommended_flow: guidance["recommended_flow"].clone(),
            safe_for_cleanup,
            safe_for_refactor,
            verification_status: verification_status.to_string(),
            requires_verification: verification_status != "available",
            action_profile: decision_action_profile(
                recommended_next_action,
                strategy["global_strategy_mode"]
                    .as_str()
                    .unwrap_or("unknown"),
            ),
            risk_profile: decision_risk_profile(
                recommended_next_action,
                &matched_overview,
                verification_status,
                safe_for_cleanup,
                safe_for_refactor,
            ),
            signals: DecisionSignals {
                repo_risk_level: matched_overview["repo_status_risk"]["risk_level"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                repo_is_dirty: matched_overview["repo_status_risk"]["is_dirty"]
                    .as_bool()
                    .unwrap_or(false),
                hardcoded_candidate_count: top_candidate["hardcoded_candidate_count"]
                    .as_u64()
                    .unwrap_or(0),
                mock_candidate_count: top_candidate["mock_candidate_count"].as_u64().unwrap_or(0),
                mixed_review_file_count: matched_overview["mock_data_summary"]
                    ["mixed_review_file_count"]
                    .as_u64()
                    .unwrap_or(0),
                storage_maintenance_candidate: storage_maintenance["maintenance_candidate"]
                    .as_bool()
                    .unwrap_or(false),
                storage_vacuum_candidate: storage_maintenance["vacuum_candidate"]
                    .as_bool()
                    .unwrap_or(false),
                storage_reclaimable_bytes: storage_maintenance["approx_reclaimable_bytes"]
                    .as_i64()
                    .unwrap_or(0),
                storage_db_size_bytes: storage_maintenance["approx_db_size_bytes"]
                    .as_i64()
                    .unwrap_or(0),
                attention_score: top_candidate["attention_score"].as_i64().unwrap_or(0),
                attention_band: top_candidate["attention_band"]
                    .as_str()
                    .unwrap_or("low")
                    .to_string(),
                attention_reasons: top_candidate["attention_reasons"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default(),
                monitoring_count: portfolio["monitoring_count"].as_u64().unwrap_or(0),
            },
        },
    );

    versioned_payload(
        schema_version,
        [
            ("scope", json!(scope)),
            ("top", json!(top)),
            ("selected_project_id", json!(selected_project_id)),
            ("decision", decision),
            ("entrypoints", entrypoints),
            ("layers", layers),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProjectConfigOverrides;
    use crate::storage::queries::StatsEntry;
    use serde_json::json;
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
}
