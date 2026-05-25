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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn allow_signals() -> RecommendationSignals {
        RecommendationSignals {
            cleanup_gate_level: GateLevel::Allow,
            refactor_gate_level: GateLevel::Allow,
            monitoring_active: true,
            snapshot_available: true,
            activity_available: true,
            snapshot_stale: false,
            activity_stale: false,
            verification_missing: false,
            verification_stale: false,
            verification_failing: false,
            unused_files: 10,
        }
    }

    fn full_eligibility() -> EligibilityResult {
        EligibilityResult {
            forced_action: None,
            cleanup_review_allowed: true,
            hotspot_review_allowed: true,
        }
    }

    fn low_risk_repo() -> serde_json::Value {
        json!({
            "risk_level": "low",
            "large_diff": false,
        })
    }

    #[test]
    fn both_allowed_base_scores_are_100() {
        let signals = allow_signals();
        let eligibility = full_eligibility();
        let scores = score_review_actions(&signals, &low_risk_repo(), &eligibility);
        assert_eq!(scores.len(), 2);
        assert!(scores.iter().all(|s| s.total == 100));
    }

    #[test]
    fn cleanup_score_penalty_for_caution_gate() {
        let mut signals = allow_signals();
        signals.cleanup_gate_level = GateLevel::Caution;
        let scores = score_review_actions(&signals, &low_risk_repo(), &full_eligibility());
        let cleanup = scores.iter().find(|s| s.action == "review_unused_files").unwrap();
        assert_eq!(cleanup.total, 80);
    }

    #[test]
    fn cleanup_score_penalty_for_stale_snapshot() {
        let mut signals = allow_signals();
        signals.snapshot_stale = true;
        let scores = score_review_actions(&signals, &low_risk_repo(), &full_eligibility());
        let cleanup = scores.iter().find(|s| s.action == "review_unused_files").unwrap();
        assert_eq!(cleanup.total, 60);
    }

    #[test]
    fn cleanup_combined_penalties() {
        let mut signals = allow_signals();
        signals.cleanup_gate_level = GateLevel::Caution;
        signals.snapshot_stale = true;
        let scores = score_review_actions(&signals, &low_risk_repo(), &full_eligibility());
        let cleanup = scores.iter().find(|s| s.action == "review_unused_files").unwrap();
        assert_eq!(cleanup.total, 40);
    }

    #[test]
    fn hotspot_score_penalty_for_caution_gate() {
        let mut signals = allow_signals();
        signals.refactor_gate_level = GateLevel::Caution;
        let scores = score_review_actions(&signals, &low_risk_repo(), &full_eligibility());
        let hotspot = scores.iter().find(|s| s.action == "inspect_hot_files").unwrap();
        assert_eq!(hotspot.total, 75);
    }

    #[test]
    fn hotspot_score_penalty_for_stale_activity() {
        let mut signals = allow_signals();
        signals.activity_stale = true;
        let scores = score_review_actions(&signals, &low_risk_repo(), &full_eligibility());
        let hotspot = scores.iter().find(|s| s.action == "inspect_hot_files").unwrap();
        assert_eq!(hotspot.total, 60);
    }

    #[test]
    fn hotspot_score_penalty_for_large_diff() {
        let repo = json!({
            "risk_level": "low",
            "large_diff": true,
        });
        let signals = allow_signals();
        let scores = score_review_actions(&signals, &repo, &full_eligibility());
        let hotspot = scores.iter().find(|s| s.action == "inspect_hot_files").unwrap();
        assert_eq!(hotspot.total, 70);
    }

    #[test]
    fn hotspot_score_penalty_for_high_risk() {
        let repo = json!({
            "risk_level": "high",
            "large_diff": false,
        });
        let signals = allow_signals();
        let scores = score_review_actions(&signals, &repo, &full_eligibility());
        let hotspot = scores.iter().find(|s| s.action == "inspect_hot_files").unwrap();
        assert_eq!(hotspot.total, 90);
    }

    #[test]
    fn hotspot_combined_penalties() {
        let mut signals = allow_signals();
        signals.refactor_gate_level = GateLevel::Caution;
        signals.activity_stale = true;
        let repo = json!({
            "risk_level": "high",
            "large_diff": true,
        });
        let scores = score_review_actions(&signals, &repo, &full_eligibility());
        let hotspot = scores.iter().find(|s| s.action == "inspect_hot_files").unwrap();
        // 100 - 25 (caution) - 40 (stale) - 30 (large diff) - 10 (high risk) = -5
        assert_eq!(hotspot.total, -5);
    }

    #[test]
    fn scores_are_sorted_descending() {
        let mut signals = allow_signals();
        signals.cleanup_gate_level = GateLevel::Caution;
        let scores = score_review_actions(&signals, &low_risk_repo(), &full_eligibility());
        assert_eq!(scores.len(), 2);
        assert!(scores[0].total >= scores[1].total);
    }

    #[test]
    fn no_cleanup_when_not_allowed() {
        let eligibility = EligibilityResult {
            forced_action: None,
            cleanup_review_allowed: false,
            hotspot_review_allowed: true,
        };
        let signals = allow_signals();
        let scores = score_review_actions(&signals, &low_risk_repo(), &eligibility);
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].action, "inspect_hot_files");
    }

    #[test]
    fn no_hotspot_when_not_allowed() {
        let eligibility = EligibilityResult {
            forced_action: None,
            cleanup_review_allowed: true,
            hotspot_review_allowed: false,
        };
        let signals = allow_signals();
        let scores = score_review_actions(&signals, &low_risk_repo(), &eligibility);
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].action, "review_unused_files");
    }

    #[test]
    fn empty_scores_when_neither_allowed() {
        let eligibility = EligibilityResult {
            forced_action: None,
            cleanup_review_allowed: false,
            hotspot_review_allowed: false,
        };
        let signals = allow_signals();
        let scores = score_review_actions(&signals, &low_risk_repo(), &eligibility);
        assert!(scores.is_empty());
    }
}
