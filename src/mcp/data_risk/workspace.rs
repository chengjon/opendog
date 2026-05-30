mod aggregation;
mod priority;

use serde_json::{json, Value};

use super::super::{set_recommended_flow, tool_guidance};
use aggregation::{
    aggregate_workspace_data_risk_focus, aggregate_workspace_rule_groups,
    aggregate_workspace_rule_hits,
};
use priority::enrich_workspace_priority_project;

#[cfg(test)]
use priority::{workspace_dominant_rule_group, workspace_priority_reason};

pub(crate) fn workspace_data_risk_overview_payload(
    project_summaries: &[Value],
    total_registered_projects: usize,
) -> Value {
    let rule_hits_summary = aggregate_workspace_rule_hits(project_summaries);
    let rule_groups_summary = aggregate_workspace_rule_groups(project_summaries);
    let data_risk_focus_summary = aggregate_workspace_data_risk_focus(project_summaries);
    let matched_projects = project_summaries.len();
    let projects_with_hardcoded = project_summaries
        .iter()
        .filter(|summary| summary["hardcoded_candidate_count"].as_u64().unwrap_or(0) > 0)
        .count();
    let projects_with_mock = project_summaries
        .iter()
        .filter(|summary| summary["mock_candidate_count"].as_u64().unwrap_or(0) > 0)
        .count();
    let total_hardcoded_candidates = project_summaries
        .iter()
        .map(|summary| summary["hardcoded_candidate_count"].as_u64().unwrap_or(0))
        .sum::<u64>();
    let total_mock_candidates = project_summaries
        .iter()
        .map(|summary| summary["mock_candidate_count"].as_u64().unwrap_or(0))
        .sum::<u64>();

    let mut priority_projects = project_summaries
        .iter()
        .map(enrich_workspace_priority_project)
        .collect::<Vec<_>>();
    priority_projects.sort_by(|a, b| {
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
    priority_projects.truncate(10);

    let mut guidance = tool_guidance(
        if projects_with_hardcoded > 0 {
            "Workspace data-risk overview loaded. Review projects with hardcoded-data candidates before broad cleanup or refactor work."
        } else if projects_with_mock > 0 {
            "Workspace data-risk overview loaded. Mock-style candidates exist; confirm they are test-only before cleanup."
        } else {
            "Workspace data-risk overview loaded. No current mock or hardcoded-data candidates were detected."
        },
        &[
            "opendog get-data-risk-candidates --id <project>",
            "rg \"mock|fixture|fake|stub|sample|demo|seed\" .",
            "rg \"customer|invoice|email|address|payment|tenant\" .",
        ],
        &["get_data_risk_candidates", "get_guidance", "list_projects"],
        Some("Use shell commands to inspect candidate files directly after OPENDOG identifies which projects deserve manual review."),
    );
    if projects_with_hardcoded > 0 {
        set_recommended_flow(
            &mut guidance,
            &[
                "Start with the highest-priority project in the workspace queue.",
                "Inspect that project's hardcoded-data candidates before broad cleanup or refactor.",
                "Use project-level guidance and verification status before making edits.",
                "Repeat for the next project only after the first review path is understood.",
            ],
        );
    } else if projects_with_mock > 0 {
        set_recommended_flow(
            &mut guidance,
            &[
                "Start with the highest-priority project in the workspace queue.",
                "Confirm whether mock-style candidates are test-only artifacts.",
                "Escalate to project-level data-risk review if any runtime/shared path looks suspicious.",
            ],
        );
    } else {
        set_recommended_flow(
            &mut guidance,
            &[
                "No current workspace data-risk candidates were detected.",
                "Use agent guidance or verification status to choose the next project action.",
                "Return to workspace-level review when priorities shift across projects.",
            ],
        );
    }
    guidance["layers"]["workspace_observation"] = json!({
        "status": "available",
        "total_registered_projects": total_registered_projects,
        "matched_project_count": matched_projects,
        "projects_with_mock_candidates": projects_with_mock,
        "projects_with_hardcoded_candidates": projects_with_hardcoded,
        "total_mock_candidates": total_mock_candidates,
        "total_hardcoded_candidates": total_hardcoded_candidates,
        "data_risk_focus_distribution": data_risk_focus_summary["distribution"].clone(),
        "projects_requiring_hardcoded_review":
            data_risk_focus_summary["projects_requiring_hardcoded_review"].clone(),
        "projects_requiring_mock_review":
            data_risk_focus_summary["projects_requiring_mock_review"].clone(),
        "projects_requiring_mixed_file_review":
            data_risk_focus_summary["projects_requiring_mixed_file_review"].clone(),
        "rule_groups_summary": rule_groups_summary,
        "rule_hits_summary": rule_hits_summary,
    });
    guidance["layers"]["multi_project_portfolio"] = json!({
        "status": "available",
        "total_registered_projects": total_registered_projects,
        "matched_project_count": matched_projects,
        "projects_with_mock_candidates": projects_with_mock,
        "projects_with_hardcoded_candidates": projects_with_hardcoded,
        "total_mock_candidates": total_mock_candidates,
        "total_hardcoded_candidates": total_hardcoded_candidates,
        "rule_groups_summary": rule_groups_summary,
        "rule_hits_summary": rule_hits_summary,
        "priority_projects": priority_projects,
    });
    guidance["layers"]["execution_strategy"]["projects_with_hardcoded_data_candidates"] =
        json!(projects_with_hardcoded);
    guidance["layers"]["execution_strategy"]["review_mock_data_before_cleanup"] =
        json!(projects_with_hardcoded > 0);
    guidance["layers"]["execution_strategy"]["data_risk_focus_distribution"] =
        data_risk_focus_summary["distribution"].clone();
    guidance["layers"]["execution_strategy"]["projects_requiring_hardcoded_review"] =
        data_risk_focus_summary["projects_requiring_hardcoded_review"].clone();
    guidance["layers"]["execution_strategy"]["projects_requiring_mock_review"] =
        data_risk_focus_summary["projects_requiring_mock_review"].clone();
    guidance["layers"]["execution_strategy"]["projects_requiring_mixed_file_review"] =
        data_risk_focus_summary["projects_requiring_mixed_file_review"].clone();
    guidance["layers"]["cleanup_refactor_candidates"] = json!({
        "status": "available",
        "priority_projects": priority_projects,
    });
    guidance
}

#[cfg(test)]
mod tests;
