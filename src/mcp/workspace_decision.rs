use serde_json::{json, Value};

use crate::config::ProjectInfo;
use crate::contracts::versioned_payload;
use crate::storage::queries::StatsEntry;

use super::{
    augment_entrypoints_for_storage_maintenance, decision_action_profile,
    decision_entrypoints_payload, decision_execution_templates, decision_risk_profile,
    detect_mock_data_report, workspace_data_risk_overview_payload,
};

pub(crate) fn workspace_data_risk_payload<F>(
    schema_version: &str,
    projects: &[ProjectInfo],
    candidate_type: &str,
    min_review_priority: &str,
    project_limit: usize,
    load_entries: F,
) -> Value
where
    F: FnMut(&ProjectInfo) -> Vec<StatsEntry>,
{
    let total_registered_projects = projects.len();
    let mut summaries = collect_workspace_data_risk_summaries(
        projects,
        candidate_type,
        min_review_priority,
        load_entries,
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

pub(crate) fn collect_workspace_data_risk_summaries<F>(
    projects: &[ProjectInfo],
    candidate_type: &str,
    min_review_priority: &str,
    mut load_entries: F,
) -> Vec<Value>
where
    F: FnMut(&ProjectInfo) -> Vec<StatsEntry>,
{
    let mut summaries = Vec::new();
    for project in projects {
        let entries = load_entries(project);
        let report = detect_mock_data_report(&project.root_path, &entries);
        let filtered = report.filtered(candidate_type, Some(min_review_priority));
        let summary = filtered.to_value(5);
        let rendered = json!({
            "project_id": project.id,
            "status": project.status,
            "mock_candidate_count": summary["mock_candidate_count"].clone(),
            "hardcoded_candidate_count": summary["hardcoded_candidate_count"].clone(),
            "mixed_review_file_count": summary["mixed_review_file_count"].clone(),
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
        layers["workspace_observation"]["rule_groups_summary"] =
            risk_observation["rule_groups_summary"].clone();
        layers["workspace_observation"]["rule_hits_summary"] =
            risk_observation["rule_hits_summary"].clone();
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

    versioned_payload(
        schema_version,
        [
            ("scope", json!(scope)),
            ("top", json!(top)),
            ("selected_project_id", json!(selected_project_id)),
            (
                "decision",
                json!({
                    "summary": guidance["recommended_flow"]
                        .as_array()
                        .and_then(|steps| steps.first())
                        .and_then(|step| step.as_str())
                        .unwrap_or("No recommendation available."),
                    "recommended_next_action": recommended_next_action,
                    "reason": top_candidate["reason"].clone(),
                    "repo_truth_gaps": top_candidate["repo_truth_gaps"].clone(),
                    "mandatory_shell_checks": top_candidate["mandatory_shell_checks"].clone(),
                    "target_project_id": target_project_id,
                    "strategy_mode": strategy["global_strategy_mode"].clone(),
                    "preferred_primary_tool": strategy["preferred_primary_tool"].clone(),
                    "preferred_secondary_tool": strategy["preferred_secondary_tool"].clone(),
                    "recommended_flow": guidance["recommended_flow"].clone(),
                    "safe_for_cleanup": safe_for_cleanup,
                    "safe_for_refactor": safe_for_refactor,
                    "verification_status": verification_status,
                    "requires_verification": verification_status != "available",
                    "action_profile": decision_action_profile(
                        recommended_next_action,
                        strategy["global_strategy_mode"].as_str().unwrap_or("unknown"),
                    ),
                    "risk_profile": decision_risk_profile(
                        recommended_next_action,
                        &matched_overview,
                        verification_status,
                        safe_for_cleanup,
                        safe_for_refactor,
                    ),
                    "signals": {
                        "repo_risk_level": matched_overview["repo_status_risk"]["risk_level"]
                            .as_str()
                            .unwrap_or("unknown"),
                        "repo_is_dirty": matched_overview["repo_status_risk"]["is_dirty"]
                            .as_bool()
                            .unwrap_or(false),
                        "hardcoded_candidate_count": top_candidate["hardcoded_candidate_count"]
                            .as_u64()
                            .unwrap_or(0),
                        "mock_candidate_count": top_candidate["mock_candidate_count"]
                            .as_u64()
                            .unwrap_or(0),
                        "storage_maintenance_candidate": storage_maintenance["maintenance_candidate"]
                            .as_bool()
                            .unwrap_or(false),
                        "storage_vacuum_candidate": storage_maintenance["vacuum_candidate"]
                            .as_bool()
                            .unwrap_or(false),
                        "storage_reclaimable_bytes": storage_maintenance["approx_reclaimable_bytes"]
                            .as_i64()
                            .unwrap_or(0),
                        "storage_db_size_bytes": storage_maintenance["approx_db_size_bytes"]
                            .as_i64()
                            .unwrap_or(0),
                        "attention_score": top_candidate["attention_score"]
                            .as_i64()
                            .unwrap_or(0),
                        "attention_band": top_candidate["attention_band"]
                            .as_str()
                            .unwrap_or("low"),
                        "attention_reasons": top_candidate["attention_reasons"]
                            .as_array()
                            .cloned()
                            .unwrap_or_default(),
                        "monitoring_count": portfolio["monitoring_count"].as_u64().unwrap_or(0),
                    },
                }),
            ),
            ("entrypoints", entrypoints),
            ("layers", layers),
        ],
    )
}
