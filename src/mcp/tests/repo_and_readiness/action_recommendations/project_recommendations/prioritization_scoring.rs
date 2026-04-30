use super::*;
use crate::mcp::project_recommendation::eligibility::{
    determine_action_eligibility, GateLevel, RecommendationSignals,
};
use crate::mcp::project_recommendation::scoring::score_review_actions;

fn base_signals() -> RecommendationSignals {
    RecommendationSignals {
        cleanup_gate_level: GateLevel::Allow,
        refactor_gate_level: GateLevel::Allow,
        safe_for_cleanup: true,
        safe_for_refactor: true,
        cleanup_reason: "cleanup-ready".to_string(),
        refactor_reason: "refactor-ready".to_string(),
        monitoring_active: true,
        snapshot_available: true,
        activity_available: true,
        snapshot_stale: false,
        activity_stale: false,
        verification_missing: false,
        verification_stale: false,
        verification_failing: false,
        unused_files: 4,
    }
}

#[test]
fn determine_action_eligibility_blocks_hotspot_review_when_refactor_gate_is_blocked() {
    let mut signals = base_signals();
    signals.cleanup_gate_level = GateLevel::Caution;
    signals.refactor_gate_level = GateLevel::Blocked;
    signals.safe_for_refactor = false;
    signals.refactor_reason = "build evidence is missing".to_string();

    let eligibility = determine_action_eligibility(
        &signals,
        &json!({
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": false,
            "risk_level": "low"
        }),
    );

    assert!(eligibility.cleanup_review_allowed);
    assert!(!eligibility.hotspot_review_allowed);
    assert_eq!(eligibility.forced_action, None);
}

#[test]
fn score_review_actions_penalizes_hotspot_review_more_than_unused_review_for_large_diff() {
    let signals = base_signals();
    let eligibility = determine_action_eligibility(
        &signals,
        &json!({
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": true,
            "changed_file_count": 18,
            "risk_level": "high"
        }),
    );

    let scores = score_review_actions(
        &signals,
        &json!({
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": true,
            "changed_file_count": 18,
            "risk_level": "high"
        }),
        &eligibility,
    );

    assert_eq!(scores[0].action, "review_unused_files");
    assert!(scores[0].total > scores[1].total);
}
