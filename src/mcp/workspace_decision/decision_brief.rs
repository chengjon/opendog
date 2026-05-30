use serde_json::{json, Value};

use crate::contracts::versioned_payload;

use super::super::{
    augment_entrypoints_for_storage_maintenance, decision_action_profile,
    decision_entrypoints_payload, decision_execution_templates, decision_risk_profile,
    guidance_types::{DecisionBrief, DecisionSignals},
    serialization::to_value_or_error,
};

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
