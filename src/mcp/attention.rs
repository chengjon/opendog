use serde_json::{json, Value};

use super::guidance_types::{AttentionPriorityBasis, AttentionSummary};
use super::serialization::to_value_or_error;

mod portfolio;

#[cfg(test)]
use portfolio::attention_batches_from_queue;
pub(super) use portfolio::{sort_project_recommendations, workspace_portfolio_layer};

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

#[cfg(test)]
mod tests;
