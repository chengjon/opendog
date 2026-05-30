use serde_json::{json, Value};

use crate::config::ProjectInfo;
use crate::contracts::versioned_payload;
use crate::storage::database::Database;
use crate::storage::queries::{upsert_data_risk_cache, StatsEntry};

use super::super::{detect_mock_data_report, workspace_data_risk_overview_payload};

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

        // Cache unfiltered counts for governance observation hints.
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
