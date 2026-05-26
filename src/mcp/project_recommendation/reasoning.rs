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
        "unused-file review"
    };
    let winning_label = if selected.action == "inspect_hot_files" {
        "hotspot review"
    } else {
        "unused-file review"
    };

    if dominant_constraint == "verification evidence"
        && (signals.cleanup_gate_level == GateLevel::Caution
            || signals.refactor_gate_level == GateLevel::Caution)
    {
        format!(
            "Current verification evidence is cautious, so {} is the safer next step, and {} stays behind it for now.",
            winning_label, losing_label
        )
    } else {
        format!(
            "Current {} makes {} the safer next step, and {} stays behind it for now.",
            dominant_constraint, winning_label, losing_label
        )
    }
}

pub(crate) fn derive_confidence(
    selected: &ActionScore,
    signals: &RecommendationSignals,
    repo_risk: &Value,
) -> &'static str {
    let repo_is_mid_operation = repo_risk["operation_states"]
        .as_array()
        .map(|states| !states.is_empty())
        .unwrap_or(false);
    let is_strong_ready_signal = selected.total >= 100
        && signals.cleanup_gate_level == GateLevel::Allow
        && signals.refactor_gate_level == GateLevel::Allow
        && repo_risk["risk_level"].as_str().unwrap_or("low") == "low";

    if signals.verification_failing || repo_is_mid_operation || is_strong_ready_signal {
        "high"
    } else {
        "medium"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_score(action: &'static str, total: i32) -> ActionScore {
        ActionScore { action, total }
    }

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

    fn low_risk_repo() -> serde_json::Value {
        json!({
            "risk_level": "low",
            "operation_states": [],
            "large_diff": false,
        })
    }

    // --- build_reason tests ---

    #[test]
    fn reason_normal_case_uses_current_evidence() {
        let selected = make_score("review_unused_files", 100);
        let runner_up = make_score("inspect_hot_files", 90);
        let signals = allow_signals();
        let reason = build_reason(&selected, Some(&runner_up), &signals, &low_risk_repo());
        assert!(reason.contains("unused-file review"));
        assert!(reason.contains("hotspot review"));
        assert!(reason.contains("current evidence"));
    }

    #[test]
    fn reason_with_caution_cleanup_gate() {
        let mut signals = allow_signals();
        signals.cleanup_gate_level = GateLevel::Caution;
        let selected = make_score("review_unused_files", 80);
        let runner_up = make_score("inspect_hot_files", 90);
        let reason = build_reason(&selected, Some(&runner_up), &signals, &low_risk_repo());
        assert!(reason.contains("verification evidence is cautious"));
        assert!(reason.contains("unused-file review"));
        assert!(reason.contains("hotspot review"));
    }

    #[test]
    fn reason_with_caution_refactor_gate() {
        let mut signals = allow_signals();
        signals.refactor_gate_level = GateLevel::Caution;
        let selected = make_score("inspect_hot_files", 100);
        let runner_up = make_score("review_unused_files", 80);
        let reason = build_reason(&selected, Some(&runner_up), &signals, &low_risk_repo());
        assert!(reason.contains("verification evidence is cautious"));
    }

    #[test]
    fn reason_with_high_repo_risk() {
        let repo = json!({
            "risk_level": "high",
            "operation_states": [],
            "large_diff": false,
        });
        let selected = make_score("review_unused_files", 100);
        let signals = allow_signals();
        let reason = build_reason(&selected, None, &signals, &repo);
        assert!(reason.contains("repository state"));
    }

    #[test]
    fn reason_with_large_diff() {
        let repo = json!({
            "risk_level": "low",
            "operation_states": [],
            "large_diff": true,
        });
        let selected = make_score("inspect_hot_files", 100);
        let signals = allow_signals();
        let reason = build_reason(&selected, None, &signals, &repo);
        assert!(reason.contains("repository state"));
    }

    #[test]
    fn reason_with_stale_snapshot() {
        let mut signals = allow_signals();
        signals.snapshot_stale = true;
        let selected = make_score("review_unused_files", 60);
        let reason = build_reason(&selected, None, &signals, &low_risk_repo());
        assert!(reason.contains("observation freshness"));
    }

    #[test]
    fn reason_with_stale_activity() {
        let mut signals = allow_signals();
        signals.activity_stale = true;
        let selected = make_score("inspect_hot_files", 60);
        let reason = build_reason(&selected, None, &signals, &low_risk_repo());
        assert!(reason.contains("observation freshness"));
    }

    #[test]
    fn reason_without_runner_up_uses_opposite_action() {
        let selected = make_score("inspect_hot_files", 100);
        let signals = allow_signals();
        let reason = build_reason(&selected, None, &signals, &low_risk_repo());
        assert!(reason.contains("hotspot review"));
        assert!(reason.contains("unused-file review"));
    }

    #[test]
    fn reason_cleanup_selected_without_runner_up() {
        let selected = make_score("review_unused_files", 100);
        let signals = allow_signals();
        let reason = build_reason(&selected, None, &signals, &low_risk_repo());
        assert!(reason.contains("unused-file review"));
        assert!(reason.contains("hotspot review"));
    }

    #[test]
    fn reason_winner_and_runner_up_labels_correct() {
        let selected = make_score("inspect_hot_files", 100);
        let runner_up = make_score("review_unused_files", 80);
        let signals = allow_signals();
        let reason = build_reason(&selected, Some(&runner_up), &signals, &low_risk_repo());
        // Winner is hotspot, loser is unused-file
        assert!(reason.contains("hotspot review the safer next step"));
        assert!(reason.contains("unused-file review stays behind"));
    }

    // --- derive_confidence tests ---

    #[test]
    fn confidence_high_when_verification_failing() {
        let mut signals = allow_signals();
        signals.verification_failing = true;
        let score = make_score("review_unused_files", 100);
        assert_eq!(
            derive_confidence(&score, &signals, &low_risk_repo()),
            "high"
        );
    }

    #[test]
    fn confidence_high_when_repo_mid_operation() {
        let repo = json!({
            "risk_level": "low",
            "operation_states": ["merge"],
            "large_diff": false,
        });
        let score = make_score("review_unused_files", 100);
        assert_eq!(derive_confidence(&score, &allow_signals(), &repo), "high");
    }

    #[test]
    fn confidence_high_when_strong_ready_signal() {
        let score = make_score("review_unused_files", 100);
        assert_eq!(
            derive_confidence(&score, &allow_signals(), &low_risk_repo()),
            "high"
        );
    }

    #[test]
    fn confidence_medium_when_score_below_100() {
        let score = make_score("review_unused_files", 80);
        let repo = json!({
            "risk_level": "low",
            "operation_states": [],
            "large_diff": false,
        });
        assert_eq!(derive_confidence(&score, &allow_signals(), &repo), "medium");
    }

    #[test]
    fn confidence_medium_when_caution_gate() {
        let mut signals = allow_signals();
        signals.cleanup_gate_level = GateLevel::Caution;
        let score = make_score("review_unused_files", 100);
        assert_eq!(
            derive_confidence(&score, &signals, &low_risk_repo()),
            "medium"
        );
    }

    #[test]
    fn confidence_medium_when_non_low_repo_risk() {
        let repo = json!({
            "risk_level": "medium",
            "operation_states": [],
            "large_diff": false,
        });
        let score = make_score("review_unused_files", 100);
        assert_eq!(derive_confidence(&score, &allow_signals(), &repo), "medium");
    }

    #[test]
    fn confidence_medium_for_typical_mixed_case() {
        let mut signals = allow_signals();
        signals.snapshot_stale = true;
        let score = make_score("review_unused_files", 60);
        assert_eq!(
            derive_confidence(&score, &signals, &low_risk_repo()),
            "medium"
        );
    }

    #[test]
    fn confidence_high_strong_signal_overrides_medium_risk() {
        // Score >= 100, all gates allow, but risk is not low => medium, not strong
        let repo = json!({
            "risk_level": "medium",
            "operation_states": [],
            "large_diff": false,
        });
        let score = make_score("review_unused_files", 100);
        // strong_ready_signal requires risk == "low", so this is medium
        assert_eq!(derive_confidence(&score, &allow_signals(), &repo), "medium");
    }
}
