use serde_json::{json, Value};
use std::collections::BTreeMap;

use super::guidance_types::{
    AttentionPriorityBasis, AttentionSummary, WorkspacePortfolioLayer,
    WorkspacePortfolioLayerStatus,
};
use super::serialization::to_value_or_error;

fn repo_risk_priority(score: &str) -> i32 {
    match score {
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

fn confidence_priority(score: &str) -> i32 {
    match score {
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

fn attention_action_base(action: &str) -> i64 {
    match action {
        "stabilize_repository_state" => 100,
        "review_failing_verification" => 95,
        "run_verification_before_high_risk_changes" => 75,
        "take_snapshot" => 65,
        "start_monitor" => 60,
        "generate_activity_then_stats" => 55,
        "review_unused_files" => 40,
        "inspect_hot_files" => 30,
        _ => 20,
    }
}

fn attention_band(score: i64) -> &'static str {
    match score {
        120.. => "critical",
        80..=119 => "high",
        45..=79 => "medium",
        _ => "low",
    }
}

fn freshness_attention_score(status: &str, missing_weight: i64, stale_weight: i64) -> i64 {
    match status {
        "missing" => missing_weight,
        "stale" | "unknown" => stale_weight,
        _ => 0,
    }
}

fn project_attention_summary(overview: &Value) -> AttentionSummary {
    let action = overview["recommended_next_action"]
        .as_str()
        .unwrap_or("inspect_workspace_state");
    let repo_risk_level = overview["repo_status_risk"]["risk_level"]
        .as_str()
        .unwrap_or("unknown");
    let repo_in_operation = overview["repo_status_risk"]["operation_states"]
        .as_array()
        .map(|states| !states.is_empty())
        .unwrap_or(false);
    let repo_is_dirty = overview["repo_status_risk"]["is_dirty"]
        .as_bool()
        .unwrap_or(false);
    let verification_status = overview["verification_evidence"]["status"]
        .as_str()
        .unwrap_or("not_recorded");
    let has_failing_verification = overview["verification_evidence"]["failing_runs"]
        .as_array()
        .map(|runs| !runs.is_empty())
        .unwrap_or(false);
    let snapshot_status = overview["observation"]["freshness"]["snapshot"]["status"]
        .as_str()
        .unwrap_or("unknown");
    let activity_status = overview["observation"]["freshness"]["activity"]["status"]
        .as_str()
        .unwrap_or("unknown");
    let verification_freshness = overview["observation"]["freshness"]["verification"]["status"]
        .as_str()
        .unwrap_or("unknown");
    let coverage_state = overview["observation"]["coverage_state"]
        .as_str()
        .unwrap_or("unknown");
    let hardcoded_candidate_count = overview["mock_data_summary"]["hardcoded_candidate_count"]
        .as_u64()
        .unwrap_or(0);
    let mock_candidate_count = overview["mock_data_summary"]["mock_candidate_count"]
        .as_u64()
        .unwrap_or(0);
    let safe_for_cleanup = overview["safe_for_cleanup"].as_bool().unwrap_or(false);
    let safe_for_refactor = overview["safe_for_refactor"].as_bool().unwrap_or(false);

    let mut score = attention_action_base(action);
    score += match repo_risk_level {
        "high" => 18,
        "medium" => 8,
        "low" => 0,
        _ => 4,
    };
    if repo_in_operation {
        score += 30;
    }
    if repo_is_dirty {
        score += 6;
    }
    if has_failing_verification {
        score += 25;
    }
    if verification_status == "not_recorded" {
        score += 18;
    }
    score += freshness_attention_score(snapshot_status, 14, 9);
    score += freshness_attention_score(activity_status, 12, 8);
    score += freshness_attention_score(verification_freshness, 18, 12);
    score += std::cmp::min(hardcoded_candidate_count, 3) as i64 * 5;
    score += std::cmp::min(mock_candidate_count, 3) as i64 * 2;
    if !safe_for_cleanup {
        score += 6;
    }
    if !safe_for_refactor {
        score += 6;
    }

    let mut reasons = Vec::new();
    if repo_in_operation {
        reasons.push(
            "Repository is mid-operation, so stabilization outranks cleanup or refactor review."
                .to_string(),
        );
    }
    if has_failing_verification {
        reasons.push("Recorded verification is currently failing.".to_string());
    } else if verification_status == "not_recorded" {
        reasons.push("Verification evidence is missing for risky follow-up work.".to_string());
    } else if matches!(verification_freshness, "stale" | "unknown") {
        reasons.push("Recorded verification evidence is stale.".to_string());
    }
    if snapshot_status == "missing" {
        reasons.push("Snapshot baseline is missing.".to_string());
    } else if matches!(snapshot_status, "stale" | "unknown") {
        reasons.push("Snapshot baseline is stale.".to_string());
    }
    if activity_status == "missing" {
        reasons.push("Activity evidence is missing.".to_string());
    } else if matches!(activity_status, "stale" | "unknown") {
        reasons.push("Activity evidence is stale.".to_string());
    }
    if hardcoded_candidate_count > 0 {
        reasons.push("Hardcoded-data candidates require manual review.".to_string());
    } else if mock_candidate_count > 0 {
        reasons.push("Mock-style data candidates still need classification review.".to_string());
    }
    if repo_is_dirty && repo_risk_level != "low" {
        reasons.push("Working tree is dirty and repository risk is elevated.".to_string());
    } else if repo_risk_level == "high" {
        reasons.push("Repository risk is high.".to_string());
    }
    if !safe_for_cleanup || !safe_for_refactor {
        reasons.push(
            "Cleanup or refactor work is currently blocked by repository or verification state."
                .to_string(),
        );
    }
    if reasons.is_empty() {
        reasons.push("Current evidence supports routine review sequencing.".to_string());
    }

    let evidence_quality = if repo_in_operation || has_failing_verification {
        "blocked"
    } else if matches!(
        coverage_state,
        "missing_snapshot" | "missing_activity" | "missing_verification"
    ) {
        "missing"
    } else if coverage_state == "stale_evidence"
        || matches!(snapshot_status, "stale" | "unknown")
        || matches!(activity_status, "stale" | "unknown")
        || matches!(verification_freshness, "stale" | "unknown")
    {
        "stale"
    } else {
        "ready"
    };

    AttentionSummary {
        attention_score: score,
        attention_band: attention_band(score).to_string(),
        attention_reasons: reasons,
        evidence_quality: evidence_quality.to_string(),
        priority_basis: AttentionPriorityBasis {
            recommended_next_action: action.to_string(),
            recommended_action_base: attention_action_base(action),
            repo_risk_level: repo_risk_level.to_string(),
            repo_in_operation,
            repo_is_dirty,
            verification_status: verification_status.to_string(),
            has_failing_verification,
            coverage_state: coverage_state.to_string(),
            snapshot_freshness: snapshot_status.to_string(),
            activity_freshness: activity_status.to_string(),
            verification_freshness: verification_freshness.to_string(),
            hardcoded_candidate_count,
            mock_candidate_count,
            safe_for_cleanup,
            safe_for_refactor,
        },
    }
}

fn thin_attention_batch_entry(project: &Value) -> Value {
    json!({
        "project_id": project["project_id"].clone(),
        "recommended_next_action": project["recommended_next_action"].clone(),
        "attention_score": project["attention_score"].clone(),
        "attention_band": project["attention_band"].clone(),
    })
}

fn attention_batches_from_queue(
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

pub(super) fn enrich_project_overview_with_attention(overview: &Value) -> Value {
    let mut enriched = overview.clone();
    let attention = project_attention_summary(&enriched);
    enriched["attention_score"] = json!(attention.attention_score);
    enriched["attention_band"] = json!(attention.attention_band);
    enriched["attention_reasons"] = json!(attention.attention_reasons);
    enriched["evidence_quality"] = json!(attention.evidence_quality);
    enriched["priority_basis"] =
        to_value_or_error("AttentionPriorityBasis", attention.priority_basis);
    enriched
}

pub(super) fn workspace_portfolio_layer(
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

pub(super) fn sort_project_recommendations(
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- attention_action_base ---

    #[test]
    fn attention_action_base_known_actions() {
        assert_eq!(attention_action_base("stabilize_repository_state"), 100);
        assert_eq!(attention_action_base("review_failing_verification"), 95);
        assert_eq!(
            attention_action_base("run_verification_before_high_risk_changes"),
            75
        );
        assert_eq!(attention_action_base("take_snapshot"), 65);
        assert_eq!(attention_action_base("start_monitor"), 60);
        assert_eq!(attention_action_base("generate_activity_then_stats"), 55);
        assert_eq!(attention_action_base("review_unused_files"), 40);
        assert_eq!(attention_action_base("inspect_hot_files"), 30);
    }

    #[test]
    fn attention_action_base_unknown_action_returns_default() {
        assert_eq!(attention_action_base("nonexistent_action"), 20);
        assert_eq!(attention_action_base(""), 20);
        assert_eq!(attention_action_base("inspect_workspace_state"), 20);
    }

    // --- attention_band ---

    #[test]
    fn attention_band_critical() {
        assert_eq!(attention_band(120), "critical");
        assert_eq!(attention_band(200), "critical");
        assert_eq!(attention_band(i64::MAX), "critical");
    }

    #[test]
    fn attention_band_high() {
        assert_eq!(attention_band(80), "high");
        assert_eq!(attention_band(100), "high");
        assert_eq!(attention_band(119), "high");
    }

    #[test]
    fn attention_band_medium() {
        assert_eq!(attention_band(45), "medium");
        assert_eq!(attention_band(60), "medium");
        assert_eq!(attention_band(79), "medium");
    }

    #[test]
    fn attention_band_low() {
        assert_eq!(attention_band(0), "low");
        assert_eq!(attention_band(44), "low");
        assert_eq!(attention_band(-10), "low");
        assert_eq!(attention_band(i64::MIN), "low");
    }

    // --- freshness_attention_score ---

    #[test]
    fn freshness_attention_score_missing() {
        assert_eq!(freshness_attention_score("missing", 14, 9), 14);
        assert_eq!(freshness_attention_score("missing", 20, 10), 20);
    }

    #[test]
    fn freshness_attention_score_stale_and_unknown() {
        assert_eq!(freshness_attention_score("stale", 14, 9), 9);
        assert_eq!(freshness_attention_score("unknown", 14, 9), 9);
    }

    #[test]
    fn freshness_attention_score_fresh_returns_zero() {
        assert_eq!(freshness_attention_score("fresh", 14, 9), 0);
        assert_eq!(freshness_attention_score("anything_else", 14, 9), 0);
        assert_eq!(freshness_attention_score("", 14, 9), 0);
    }

    // --- repo_risk_priority ---

    #[test]
    fn repo_risk_priority_all_levels() {
        assert_eq!(repo_risk_priority("high"), 3);
        assert_eq!(repo_risk_priority("medium"), 2);
        assert_eq!(repo_risk_priority("low"), 1);
        assert_eq!(repo_risk_priority("unknown"), 0);
        assert_eq!(repo_risk_priority(""), 0);
    }

    // --- confidence_priority ---

    #[test]
    fn confidence_priority_all_levels() {
        assert_eq!(confidence_priority("high"), 3);
        assert_eq!(confidence_priority("medium"), 2);
        assert_eq!(confidence_priority("low"), 1);
        assert_eq!(confidence_priority("unknown"), 0);
        assert_eq!(confidence_priority(""), 0);
    }

    // --- attention_batches_from_queue ---

    #[test]
    fn attention_batches_empty_queue() {
        let result = attention_batches_from_queue(&[], 3, "available");
        assert_eq!(result["status"], "available");
        assert_eq!(result["source"], "attention_queue");
        assert_eq!(result["batched_project_count"], 0);
        assert_eq!(result["unbatched_project_count"], 3);
        assert_eq!(result["immediate"].as_array().unwrap().len(), 0);
        assert_eq!(result["next"].as_array().unwrap().len(), 0);
        assert_eq!(result["later"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn attention_batches_single_item_goes_to_immediate() {
        let queue = vec![json!({
            "project_id": "proj1",
            "recommended_next_action": "take_snapshot",
            "attention_score": 80,
            "attention_band": "high",
        })];
        let result = attention_batches_from_queue(&queue, 1, "available");
        let immediate = result["immediate"].as_array().unwrap();
        let next = result["next"].as_array().unwrap();
        let later = result["later"].as_array().unwrap();
        assert_eq!(immediate.len(), 1);
        assert_eq!(immediate[0]["project_id"], "proj1");
        assert_eq!(next.len(), 0);
        assert_eq!(later.len(), 0);
    }

    #[test]
    fn attention_batches_splits_into_immediate_next_later() {
        let queue: Vec<Value> = (0..6)
            .map(|i| {
                json!({
                    "project_id": format!("proj{i}"),
                    "recommended_next_action": "inspect_hot_files",
                    "attention_score": 100 - i as i64 * 10,
                    "attention_band": "high",
                })
            })
            .collect();
        let result = attention_batches_from_queue(&queue, 10, "available");
        let immediate = result["immediate"].as_array().unwrap();
        let next = result["next"].as_array().unwrap();
        let later = result["later"].as_array().unwrap();
        // immediate = first 1
        assert_eq!(immediate.len(), 1);
        assert_eq!(immediate[0]["project_id"], "proj0");
        // next = skip 1, take 2
        assert_eq!(next.len(), 2);
        assert_eq!(next[0]["project_id"], "proj1");
        assert_eq!(next[1]["project_id"], "proj2");
        // later = skip 3, rest
        assert_eq!(later.len(), 3);
        assert_eq!(later[0]["project_id"], "proj3");
        // unbatched = total(10) - queue(6) = 4
        assert_eq!(result["unbatched_project_count"], 4);
    }

    #[test]
    fn attention_batches_thin_entries_only_have_four_fields() {
        let queue = vec![json!({
            "project_id": "proj1",
            "recommended_next_action": "take_snapshot",
            "attention_score": 80,
            "attention_band": "high",
            "extra_field": "should_not_appear",
        })];
        let result = attention_batches_from_queue(&queue, 1, "available");
        let entry = &result["immediate"].as_array().unwrap()[0];
        // Only 4 thin fields should be present
        assert!(entry.get("extra_field").is_none());
        assert!(entry.get("project_id").is_some());
        assert!(entry.get("recommended_next_action").is_some());
        assert!(entry.get("attention_score").is_some());
        assert!(entry.get("attention_band").is_some());
    }

    // --- project_attention_summary ---

    fn minimal_overview() -> Value {
        json!({
            "recommended_next_action": "inspect_hot_files",
            "repo_status_risk": {
                "risk_level": "low",
                "operation_states": [],
                "is_dirty": false,
            },
            "verification_evidence": {
                "status": "recorded",
                "failing_runs": [],
            },
            "observation": {
                "freshness": {
                    "snapshot": { "status": "fresh" },
                    "activity": { "status": "fresh" },
                    "verification": { "status": "fresh" },
                },
                "coverage_state": "active",
            },
            "mock_data_summary": {
                "hardcoded_candidate_count": 0,
                "mock_candidate_count": 0,
            },
            "safe_for_cleanup": true,
            "safe_for_refactor": true,
        })
    }

    #[test]
    fn project_attention_summary_minimal_clean_state() {
        let overview = minimal_overview();
        let summary = project_attention_summary(&overview);
        // Base for "inspect_hot_files" = 30, repo_risk low = +0, all fresh = +0
        // hardcoded=0, mock=0, safe_cleanup=true, safe_refactor=true => +0
        assert_eq!(summary.attention_score, 30);
        assert_eq!(summary.attention_band, "low");
        assert_eq!(summary.evidence_quality, "ready");
        assert_eq!(
            summary.priority_basis.recommended_next_action,
            "inspect_hot_files"
        );
        assert_eq!(summary.priority_basis.recommended_action_base, 30);
        assert_eq!(summary.priority_basis.repo_risk_level, "low");
        assert!(!summary.priority_basis.repo_in_operation);
        assert!(!summary.priority_basis.repo_is_dirty);
        assert!(summary.priority_basis.safe_for_cleanup);
        assert!(summary.priority_basis.safe_for_refactor);
        // Routine reason
        assert_eq!(
            summary.attention_reasons,
            vec!["Current evidence supports routine review sequencing.".to_string()]
        );
    }

    #[test]
    fn project_attention_summary_high_repo_risk_and_dirty() {
        let overview = json!({
            "recommended_next_action": "stabilize_repository_state",
            "repo_status_risk": {
                "risk_level": "high",
                "operation_states": [],
                "is_dirty": true,
            },
            "verification_evidence": {
                "status": "recorded",
                "failing_runs": [],
            },
            "observation": {
                "freshness": {
                    "snapshot": { "status": "fresh" },
                    "activity": { "status": "fresh" },
                    "verification": { "status": "fresh" },
                },
                "coverage_state": "active",
            },
            "mock_data_summary": {
                "hardcoded_candidate_count": 0,
                "mock_candidate_count": 0,
            },
            "safe_for_cleanup": true,
            "safe_for_refactor": true,
        });
        let summary = project_attention_summary(&overview);
        // Base 100 + high_risk 18 + dirty 6 = 124
        assert_eq!(summary.attention_score, 124);
        assert_eq!(summary.attention_band, "critical");
        // evidence_quality is "ready" since no blockers
        assert_eq!(summary.evidence_quality, "ready");
    }

    #[test]
    fn project_attention_summary_repo_in_operation_adds_30() {
        let mut overview = minimal_overview();
        overview["repo_status_risk"]["operation_states"] = json!(["merge"]);
        let summary = project_attention_summary(&overview);
        // Base 30 + operation 30 = 60
        assert_eq!(summary.attention_score, 60);
        assert!(summary
            .attention_reasons
            .iter()
            .any(|r| r.contains("mid-operation")));
        assert_eq!(summary.evidence_quality, "blocked");
    }

    #[test]
    fn project_attention_summary_failing_verification_adds_25() {
        let mut overview = minimal_overview();
        overview["verification_evidence"]["failing_runs"] =
            json!([{"command": "cargo test", "status": "failed"}]);
        let summary = project_attention_summary(&overview);
        // Base 30 + failing 25 = 55
        assert_eq!(summary.attention_score, 55);
        assert!(summary
            .attention_reasons
            .iter()
            .any(|r| r.contains("failing")));
        assert_eq!(summary.evidence_quality, "blocked");
    }

    #[test]
    fn project_attention_summary_missing_snapshot_adds_14() {
        let mut overview = minimal_overview();
        overview["observation"]["freshness"]["snapshot"]["status"] = json!("missing");
        let summary = project_attention_summary(&overview);
        // Base 30 + missing_snapshot 14 = 44
        assert_eq!(summary.attention_score, 44);
        assert!(summary
            .attention_reasons
            .iter()
            .any(|r| r.contains("Snapshot baseline is missing")));
    }

    #[test]
    fn project_attention_summary_stale_activity_adds_8() {
        let mut overview = minimal_overview();
        overview["observation"]["freshness"]["activity"]["status"] = json!("stale");
        let summary = project_attention_summary(&overview);
        // Base 30 + stale_activity 8 = 38
        assert_eq!(summary.attention_score, 38);
        assert!(summary
            .attention_reasons
            .iter()
            .any(|r| r.contains("Activity evidence is stale")));
    }

    #[test]
    fn project_attention_summary_not_recorded_verification_adds_18_plus_freshness_18() {
        let mut overview = minimal_overview();
        overview["verification_evidence"]["status"] = json!("not_recorded");
        overview["observation"]["freshness"]["verification"]["status"] = json!("missing");
        let summary = project_attention_summary(&overview);
        // Base 30 + not_recorded 18 + missing_verification_freshness 18 = 66
        assert_eq!(summary.attention_score, 66);
    }

    #[test]
    fn project_attention_summary_hardcoded_candidates_capped_at_3() {
        let mut overview = minimal_overview();
        overview["mock_data_summary"]["hardcoded_candidate_count"] = json!(10);
        let summary = project_attention_summary(&overview);
        // Base 30 + min(10,3)*5 = 30 + 15 = 45
        assert_eq!(summary.attention_score, 45);
        assert!(summary
            .attention_reasons
            .iter()
            .any(|r| r.contains("Hardcoded-data candidates")));
    }

    #[test]
    fn project_attention_summary_mock_candidates_capped_at_3() {
        let mut overview = minimal_overview();
        overview["mock_data_summary"]["mock_candidate_count"] = json!(5);
        let summary = project_attention_summary(&overview);
        // Base 30 + min(5,3)*2 = 30 + 6 = 36
        assert_eq!(summary.attention_score, 36);
        assert!(summary
            .attention_reasons
            .iter()
            .any(|r| r.contains("Mock-style data candidates")));
    }

    #[test]
    fn project_attention_summary_not_safe_for_cleanup_adds_6() {
        let mut overview = minimal_overview();
        overview["safe_for_cleanup"] = json!(false);
        let summary = project_attention_summary(&overview);
        // Base 30 + not_safe_cleanup 6 = 36
        assert_eq!(summary.attention_score, 36);
    }

    #[test]
    fn project_attention_summary_not_safe_for_refactor_adds_6() {
        let mut overview = minimal_overview();
        overview["safe_for_refactor"] = json!(false);
        let summary = project_attention_summary(&overview);
        // Base 30 + not_safe_refactor 6 = 36
        assert_eq!(summary.attention_score, 36);
    }

    #[test]
    fn project_attention_summary_evidence_quality_missing_snapshot() {
        let mut overview = minimal_overview();
        overview["observation"]["coverage_state"] = json!("missing_snapshot");
        let summary = project_attention_summary(&overview);
        assert_eq!(summary.evidence_quality, "missing");
    }

    #[test]
    fn project_attention_summary_evidence_quality_stale() {
        let mut overview = minimal_overview();
        overview["observation"]["freshness"]["snapshot"]["status"] = json!("stale");
        let summary = project_attention_summary(&overview);
        assert_eq!(summary.evidence_quality, "stale");
    }

    #[test]
    fn project_attention_summary_missing_key_fields_uses_defaults() {
        let overview = json!({});
        let summary = project_attention_summary(&overview);
        // Default action "inspect_workspace_state" => base 20
        // Default repo_risk_level "unknown" => +4
        // Default verification_status "not_recorded" => +18
        // Default snapshot "unknown" => freshness_attention_score("unknown", 14, 9) = 9
        // Default activity "unknown" => freshness_attention_score("unknown", 12, 8) = 8
        // Default verification freshness "unknown" => freshness_attention_score("unknown", 18, 12) = 12
        // safe_for_cleanup default false => +6
        // safe_for_refactor default false => +6
        // Total: 20 + 4 + 18 + 9 + 8 + 12 + 6 + 6 = 83
        assert_eq!(summary.attention_score, 83);
        assert_eq!(
            summary.priority_basis.recommended_next_action,
            "inspect_workspace_state"
        );
        assert_eq!(summary.priority_basis.repo_risk_level, "unknown");
        assert!(!summary.priority_basis.safe_for_cleanup);
        assert!(!summary.priority_basis.safe_for_refactor);
    }

    // --- workspace_portfolio_layer ---

    #[test]
    fn workspace_portfolio_layer_sorts_by_attention_and_truncates_to_five() {
        let overviews: Vec<Value> = (0..7)
            .map(|i| {
                let mut ov = minimal_overview();
                ov["project_id"] = json!(format!("proj{i}"));
                // Higher index = higher score via action base differences
                ov["recommended_next_action"] = if i < 3 {
                    json!("inspect_hot_files") // base 30
                } else if i < 5 {
                    json!("stabilize_repository_state") // base 100
                } else {
                    json!("review_failing_verification") // base 95
                };
                ov
            })
            .collect();
        let result = workspace_portfolio_layer(
            &overviews,
            2,
            &["proj0".to_string(), "proj1".to_string()],
            vec![],
            0,
        );
        assert_eq!(result.project_count, 7);
        assert_eq!(result.monitoring_count, 2);
        // attention_queue is truncated to 5
        assert_eq!(result.attention_queue.len(), 5);
        // All 7 project overviews are preserved (not truncated)
        assert_eq!(result.project_overviews.len(), 7);
    }

    #[test]
    fn workspace_portfolio_layer_empty_projects() {
        let result = workspace_portfolio_layer(&[], 0, &[], vec![], 0);
        assert_eq!(result.project_count, 0);
        assert_eq!(result.attention_queue.len(), 0);
        assert_eq!(result.dirty_projects, 0);
        assert_eq!(result.high_risk_projects, 0);
    }

    #[test]
    fn workspace_portfolio_layer_counts_dirty_and_high_risk() {
        let overviews = vec![
            {
                let mut ov = minimal_overview();
                ov["project_id"] = json!("dirty_high");
                ov["repo_status_risk"]["is_dirty"] = json!(true);
                ov["repo_status_risk"]["risk_level"] = json!("high");
                ov
            },
            {
                let mut ov = minimal_overview();
                ov["project_id"] = json!("clean_low");
                ov
            },
        ];
        let result = workspace_portfolio_layer(&overviews, 1, &[], vec![], 0);
        assert_eq!(result.dirty_projects, 1);
        assert_eq!(result.high_risk_projects, 1);
    }

    #[test]
    fn workspace_portfolio_layer_counts_hardcoded_candidates() {
        let overviews = vec![{
            let mut ov = minimal_overview();
            ov["project_id"] = json!("proj1");
            ov["mock_data_summary"]["hardcoded_candidate_count"] = json!(5);
            ov["mock_data_summary"]["mock_candidate_count"] = json!(3);
            ov
        }];
        let result = workspace_portfolio_layer(&overviews, 0, &[], vec![], 2);
        assert_eq!(result.projects_with_hardcoded_candidates, 1);
        assert_eq!(result.total_hardcoded_candidates, 5);
        assert_eq!(result.total_mock_candidates, 3);
        assert_eq!(result.projects_with_hardcoded_data_candidates, 2);
    }

    // --- sort_project_recommendations ---

    #[test]
    fn sort_project_recommendations_joins_and_sorts_by_attention() {
        let recommendations = vec![
            json!({
                "project_id": "low_prio",
                "confidence": "low",
            }),
            json!({
                "project_id": "high_prio",
                "confidence": "high",
            }),
        ];
        let overviews = vec![
            {
                let mut ov = minimal_overview();
                ov["project_id"] = json!("low_prio");
                // Base 30 for inspect_hot_files
                ov
            },
            {
                let mut ov = minimal_overview();
                ov["project_id"] = json!("high_prio");
                ov["recommended_next_action"] = json!("stabilize_repository_state");
                // Base 100
                ov
            },
        ];
        let sorted = sort_project_recommendations(&recommendations, &overviews);
        assert_eq!(sorted.len(), 2);
        // high_prio should be first (higher attention score)
        assert_eq!(sorted[0]["project_id"], "high_prio");
        assert_eq!(sorted[1]["project_id"], "low_prio");
        // Should have attention_score injected
        assert!(sorted[0].get("attention_score").is_some());
        assert!(sorted[0].get("attention_band").is_some());
        assert!(sorted[0].get("attention_reasons").is_some());
    }

    #[test]
    fn sort_project_recommendations_skips_overviews_without_matching_recommendation() {
        let recommendations = vec![json!({
            "project_id": "only_one",
            "confidence": "medium",
        })];
        let overviews = vec![
            {
                let mut ov = minimal_overview();
                ov["project_id"] = json!("only_one");
                ov
            },
            {
                let mut ov = minimal_overview();
                ov["project_id"] = json!("no_match");
                ov
            },
        ];
        let sorted = sort_project_recommendations(&recommendations, &overviews);
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0]["project_id"], "only_one");
    }

    #[test]
    fn sort_project_recommendations_empty_inputs() {
        let sorted = sort_project_recommendations(&[], &[]);
        assert!(sorted.is_empty());
    }
}
