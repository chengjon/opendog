use serde_json::Value;

use super::eligibility::{GateLevel, RecommendationSignals};
use super::scoring::ActionScore;

pub(crate) fn build_reason(
    selected: &ActionScore,
    runner_up: Option<&ActionScore>,
    signals: &RecommendationSignals,
    repo_risk: &Value,
) -> String {
    let dominant_constraint = if signals.cleanup_gate_level == GateLevel::Caution
        || signals.refactor_gate_level == GateLevel::Caution
    {
        "verification evidence"
    } else if repo_risk["risk_level"].as_str().unwrap_or("low") != "low"
        || repo_risk["large_diff"].as_bool().unwrap_or(false)
    {
        "repository state"
    } else if signals.snapshot_stale || signals.activity_stale {
        "observation freshness"
    } else {
        "current evidence"
    };

    let losing_action = runner_up.map(|score| score.action).unwrap_or_else(|| {
        if selected.action == "inspect_hot_files" {
            "review_unused_files"
        } else {
            "inspect_hot_files"
        }
    });
    let losing_label = if losing_action == "inspect_hot_files" {
        "hotspot review"
    } else {
        "cleanup review"
    };
    let winning_label = if selected.action == "inspect_hot_files" {
        "hotspot review"
    } else {
        "cleanup review"
    };

    format!(
        "Current {} makes {} the safer next step, and {} stays behind it for now.",
        dominant_constraint, winning_label, losing_label
    )
}

pub(crate) fn derive_confidence(
    selected: &ActionScore,
    signals: &RecommendationSignals,
    repo_risk: &Value,
) -> &'static str {
    if signals.verification_failing
        || repo_risk["operation_states"]
            .as_array()
            .map(|states| !states.is_empty())
            .unwrap_or(false)
    {
        "high"
    } else if selected.total >= 100
        && signals.cleanup_gate_level == GateLevel::Allow
        && signals.refactor_gate_level == GateLevel::Allow
        && repo_risk["risk_level"].as_str().unwrap_or("low") == "low"
    {
        "high"
    } else {
        "medium"
    }
}
