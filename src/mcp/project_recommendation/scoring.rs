use serde_json::Value;

use super::eligibility::{EligibilityResult, GateLevel, RecommendationSignals};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActionScore {
    pub(crate) action: &'static str,
    pub(crate) total: i32,
}

pub(crate) fn score_review_actions(
    signals: &RecommendationSignals,
    repo_risk: &Value,
    eligibility: &EligibilityResult,
) -> Vec<ActionScore> {
    let mut scores = Vec::new();

    if eligibility.cleanup_review_allowed {
        let mut total = 100;
        if signals.cleanup_gate_level == GateLevel::Caution {
            total -= 20;
        }
        if signals.snapshot_stale {
            total -= 40;
        }
        scores.push(ActionScore {
            action: "review_unused_files",
            total,
        });
    }

    if eligibility.hotspot_review_allowed {
        let mut total = 100;
        if signals.refactor_gate_level == GateLevel::Caution {
            total -= 25;
        }
        if signals.activity_stale {
            total -= 40;
        }
        if repo_risk["large_diff"].as_bool().unwrap_or(false) {
            total -= 30;
        }
        if repo_risk["risk_level"].as_str().unwrap_or("low") == "high" {
            total -= 10;
        }
        scores.push(ActionScore {
            action: "inspect_hot_files",
            total,
        });
    }

    scores.sort_by(|a, b| b.total.cmp(&a.total));
    scores
}
