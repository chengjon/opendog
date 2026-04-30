use serde_json::{json, Value};
use std::collections::BTreeMap;

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

fn project_attention_summary(overview: &Value) -> Value {
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

    json!({
        "attention_score": score,
        "attention_band": attention_band(score),
        "attention_reasons": reasons,
        "evidence_quality": evidence_quality,
        "priority_basis": {
            "recommended_next_action": action,
            "recommended_action_base": attention_action_base(action),
            "repo_risk_level": repo_risk_level,
            "repo_in_operation": repo_in_operation,
            "repo_is_dirty": repo_is_dirty,
            "verification_status": verification_status,
            "has_failing_verification": has_failing_verification,
            "coverage_state": coverage_state,
            "snapshot_freshness": snapshot_status,
            "activity_freshness": activity_status,
            "verification_freshness": verification_freshness,
            "hardcoded_candidate_count": hardcoded_candidate_count,
            "mock_candidate_count": mock_candidate_count,
            "safe_for_cleanup": safe_for_cleanup,
            "safe_for_refactor": safe_for_refactor
        }
    })
}

pub(super) fn enrich_project_overview_with_attention(overview: &Value) -> Value {
    let mut enriched = overview.clone();
    let attention = project_attention_summary(&enriched);
    enriched["attention_score"] = attention["attention_score"].clone();
    enriched["attention_band"] = attention["attention_band"].clone();
    enriched["attention_reasons"] = attention["attention_reasons"].clone();
    enriched["evidence_quality"] = attention["evidence_quality"].clone();
    enriched["priority_basis"] = attention["priority_basis"].clone();
    enriched
}

pub(super) fn workspace_portfolio_layer(project_overviews: &[Value]) -> Value {
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

    json!({
        "status": "available",
        "project_count": project_overviews.len(),
        "priority_model": "action_urgency_plus_evidence_risk",
        "dirty_projects": dirty_projects,
        "high_risk_projects": high_risk_projects,
        "projects_with_failing_verification": projects_with_failing_verification,
        "projects_safe_for_cleanup": projects_safe_for_cleanup,
        "projects_safe_for_refactor": projects_safe_for_refactor,
        "projects_with_hardcoded_candidates": projects_with_hardcoded_candidates,
        "total_mock_candidates": total_mock_candidates,
        "total_hardcoded_candidates": total_hardcoded_candidates,
        "projects_in_operation": projects_in_operation,
        "attention_queue": attention_queue,
    })
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
