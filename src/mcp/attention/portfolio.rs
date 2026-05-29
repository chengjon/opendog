use serde_json::{json, Value};
use std::collections::BTreeMap;

use super::super::guidance_types::{WorkspacePortfolioLayer, WorkspacePortfolioLayerStatus};
use super::{confidence_priority, enrich_project_overview_with_attention, repo_risk_priority};

fn thin_attention_batch_entry(project: &Value) -> Value {
    json!({
        "project_id": project["project_id"].clone(),
        "recommended_next_action": project["recommended_next_action"].clone(),
        "attention_score": project["attention_score"].clone(),
        "attention_band": project["attention_band"].clone(),
    })
}

pub(super) fn attention_batches_from_queue(
    attention_queue: &[Value],
    project_count: usize,
    status: &str,
) -> Value {
    let immediate = attention_queue
        .first()
        .map(thin_attention_batch_entry)
        .into_iter()
        .collect::<Vec<_>>();
    let next = attention_queue
        .iter()
        .skip(1)
        .take(2)
        .map(thin_attention_batch_entry)
        .collect::<Vec<_>>();
    let later = attention_queue
        .iter()
        .skip(3)
        .map(thin_attention_batch_entry)
        .collect::<Vec<_>>();

    json!({
        "status": status,
        "source": "attention_queue",
        "batched_project_count": attention_queue.len(),
        "unbatched_project_count": project_count.saturating_sub(attention_queue.len()),
        "immediate": immediate,
        "next": next,
        "later": later,
    })
}

pub(crate) fn workspace_portfolio_layer(
    project_overviews: &[Value],
    monitoring_count: usize,
    monitored_projects: &[String],
    priority_candidates: Vec<Value>,
    projects_with_hardcoded_data: usize,
) -> WorkspacePortfolioLayer {
    let status = WorkspacePortfolioLayerStatus::Available;
    let enriched_project_overviews = project_overviews
        .iter()
        .map(enrich_project_overview_with_attention)
        .collect::<Vec<_>>();
    let dirty_projects = project_overviews
        .iter()
        .filter(|p| p["repo_status_risk"]["is_dirty"].as_bool().unwrap_or(false))
        .count();
    let high_risk_projects = project_overviews
        .iter()
        .filter(|p| p["repo_status_risk"]["risk_level"] == "high")
        .count();
    let projects_with_failing_verification = project_overviews
        .iter()
        .filter(|p| {
            p["verification_evidence"]["failing_runs"]
                .as_array()
                .map(|v| !v.is_empty())
                .unwrap_or(false)
        })
        .count();
    let projects_safe_for_cleanup = project_overviews
        .iter()
        .filter(|p| p["safe_for_cleanup"].as_bool().unwrap_or(false))
        .count();
    let projects_safe_for_refactor = project_overviews
        .iter()
        .filter(|p| p["safe_for_refactor"].as_bool().unwrap_or(false))
        .count();
    let projects_in_operation: Vec<Value> = project_overviews
        .iter()
        .filter(|p| {
            p["repo_status_risk"]["operation_states"]
                .as_array()
                .map(|v| !v.is_empty())
                .unwrap_or(false)
        })
        .map(|p| {
            json!({
                "project_id": p["project_id"].clone(),
                "operation_states": p["repo_status_risk"]["operation_states"].clone(),
            })
        })
        .collect();
    let projects_with_hardcoded_candidates = project_overviews
        .iter()
        .filter(|p| {
            p["mock_data_summary"]["hardcoded_candidate_count"]
                .as_u64()
                .unwrap_or(0)
                > 0
        })
        .count();
    let total_mock_candidates = project_overviews
        .iter()
        .map(|p| {
            p["mock_data_summary"]["mock_candidate_count"]
                .as_u64()
                .unwrap_or(0)
        })
        .sum::<u64>();
    let total_hardcoded_candidates = project_overviews
        .iter()
        .map(|p| {
            p["mock_data_summary"]["hardcoded_candidate_count"]
                .as_u64()
                .unwrap_or(0)
        })
        .sum::<u64>();

    let mut attention_queue = enriched_project_overviews.clone();
    attention_queue.sort_by(|a, b| {
        let a_score = a["attention_score"].as_i64().unwrap_or(0);
        let b_score = b["attention_score"].as_i64().unwrap_or(0);
        let a_confidence = confidence_priority(a["strategy_confidence"].as_str().unwrap_or(""));
        let b_confidence = confidence_priority(b["strategy_confidence"].as_str().unwrap_or(""));
        let a_hardcoded = a["mock_data_summary"]["hardcoded_candidate_count"]
            .as_u64()
            .unwrap_or(0);
        let b_hardcoded = b["mock_data_summary"]["hardcoded_candidate_count"]
            .as_u64()
            .unwrap_or(0);
        b_score
            .cmp(&a_score)
            .then_with(|| b_confidence.cmp(&a_confidence))
            .then_with(|| {
                repo_risk_priority(b["repo_status_risk"]["risk_level"].as_str().unwrap_or("")).cmp(
                    &repo_risk_priority(a["repo_status_risk"]["risk_level"].as_str().unwrap_or("")),
                )
            })
            .then_with(|| b_hardcoded.cmp(&a_hardcoded))
            .then_with(|| {
                b["repo_status_risk"]["is_dirty"]
                    .as_bool()
                    .unwrap_or(false)
                    .cmp(&a["repo_status_risk"]["is_dirty"].as_bool().unwrap_or(false))
            })
            .then_with(|| {
                b["verification_evidence"]["failing_runs"]
                    .as_array()
                    .map(|v| !v.is_empty())
                    .unwrap_or(false)
                    .cmp(
                        &a["verification_evidence"]["failing_runs"]
                            .as_array()
                            .map(|v| !v.is_empty())
                            .unwrap_or(false),
                    )
            })
            .then_with(|| {
                b["unused_files"]
                    .as_i64()
                    .unwrap_or(0)
                    .cmp(&a["unused_files"].as_i64().unwrap_or(0))
            })
    });
    attention_queue.truncate(5);

    let attention_batches =
        attention_batches_from_queue(&attention_queue, project_overviews.len(), status.as_str());

    WorkspacePortfolioLayer {
        status,
        project_count: project_overviews.len(),
        monitoring_count,
        monitored_projects: monitored_projects.iter().map(|s| json!(s)).collect(),
        priority_candidates,
        project_overviews: project_overviews.to_vec(),
        priority_model: "action_urgency_plus_evidence_risk".to_string(),
        dirty_projects,
        high_risk_projects,
        projects_with_failing_verification,
        projects_safe_for_cleanup,
        projects_safe_for_refactor,
        projects_with_hardcoded_candidates,
        projects_with_hardcoded_data_candidates: projects_with_hardcoded_data,
        total_mock_candidates,
        total_hardcoded_candidates,
        projects_in_operation,
        attention_queue,
        attention_batches,
    }
}

pub(crate) fn sort_project_recommendations(
    project_recommendations: &[Value],
    project_overviews: &[Value],
) -> Vec<Value> {
    let mut by_project: BTreeMap<String, Value> = BTreeMap::new();
    for recommendation in project_recommendations {
        if let Some(project_id) = recommendation["project_id"].as_str() {
            by_project.insert(project_id.to_string(), recommendation.clone());
        }
    }

    let mut sorted = project_overviews
        .iter()
        .filter_map(|overview| {
            let project_id = overview["project_id"].as_str()?;
            let enriched_overview = enrich_project_overview_with_attention(overview);
            let mut recommendation = by_project.get(project_id)?.clone();
            recommendation["hardcoded_candidate_count"] =
                enriched_overview["mock_data_summary"]["hardcoded_candidate_count"].clone();
            recommendation["mock_candidate_count"] =
                enriched_overview["mock_data_summary"]["mock_candidate_count"].clone();
            recommendation["attention_score"] = enriched_overview["attention_score"].clone();
            recommendation["attention_band"] = enriched_overview["attention_band"].clone();
            recommendation["attention_reasons"] = enriched_overview["attention_reasons"].clone();
            recommendation["evidence_quality"] = enriched_overview["evidence_quality"].clone();
            recommendation["priority_basis"] = enriched_overview["priority_basis"].clone();
            Some((project_id.to_string(), recommendation))
        })
        .collect::<Vec<_>>();

    sorted.sort_by(|(_, a), (_, b)| {
        let a_score = a["attention_score"].as_i64().unwrap_or(0);
        let b_score = b["attention_score"].as_i64().unwrap_or(0);
        let a_hardcoded = a["hardcoded_candidate_count"].as_u64().unwrap_or(0);
        let b_hardcoded = b["hardcoded_candidate_count"].as_u64().unwrap_or(0);
        let a_confidence = confidence_priority(a["confidence"].as_str().unwrap_or(""));
        let b_confidence = confidence_priority(b["confidence"].as_str().unwrap_or(""));
        b_score
            .cmp(&a_score)
            .then_with(|| b_confidence.cmp(&a_confidence))
            .then_with(|| b_hardcoded.cmp(&a_hardcoded))
            .then_with(|| {
                b["mock_candidate_count"]
                    .as_u64()
                    .unwrap_or(0)
                    .cmp(&a["mock_candidate_count"].as_u64().unwrap_or(0))
            })
    });

    sorted
        .into_iter()
        .map(|(_, recommendation)| recommendation)
        .collect()
}
