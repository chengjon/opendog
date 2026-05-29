use super::*;
use serde_json::json;

fn default_signals() -> RecommendationSignals {
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

fn mid_operation_repo() -> serde_json::Value {
    json!({
        "risk_level": "low",
        "operation_states": ["merge"],
        "large_diff": false,
    })
}

// --- GateLevel::from_str tests ---

#[test]
fn gate_level_from_str_allow() {
    assert_eq!(GateLevel::from_str("allow"), GateLevel::Allow);
}

#[test]
fn gate_level_from_str_caution() {
    assert_eq!(GateLevel::from_str("caution"), GateLevel::Caution);
}

#[test]
fn gate_level_from_str_blocked_for_unknown() {
    assert_eq!(GateLevel::from_str("blocked"), GateLevel::Blocked);
}

#[test]
fn gate_level_from_str_arbitrary_unknown() {
    assert_eq!(GateLevel::from_str("anything_else"), GateLevel::Blocked);
}

#[test]
fn gate_level_from_str_empty() {
    assert_eq!(GateLevel::from_str(""), GateLevel::Blocked);
}

// --- determine_action_eligibility tests ---

#[test]
fn verification_failing_forces_verification_step() {
    let mut signals = default_signals();
    signals.verification_failing = true;
    let result = determine_action_eligibility(&signals, &low_risk_repo());
    assert_eq!(result.forced_action, Some("review_failing_verification"));
    assert!(!result.cleanup_review_allowed);
    assert!(!result.hotspot_review_allowed);
}

#[test]
fn verification_missing_forces_verification_step() {
    let mut signals = default_signals();
    signals.verification_missing = true;
    let result = determine_action_eligibility(&signals, &low_risk_repo());
    assert_eq!(
        result.forced_action,
        Some("run_verification_before_high_risk_changes")
    );
    assert!(!result.cleanup_review_allowed);
    assert!(!result.hotspot_review_allowed);
}

#[test]
fn verification_stale_forces_verification_step() {
    let mut signals = default_signals();
    signals.verification_stale = true;
    let result = determine_action_eligibility(&signals, &low_risk_repo());
    assert_eq!(
        result.forced_action,
        Some("run_verification_before_high_risk_changes")
    );
    assert!(!result.cleanup_review_allowed);
    assert!(!result.hotspot_review_allowed);
}

#[test]
fn mid_operation_repo_forces_stabilization() {
    let signals = default_signals();
    let result = determine_action_eligibility(&signals, &mid_operation_repo());
    assert_eq!(result.forced_action, Some("stabilize_repository_state"));
    assert!(!result.cleanup_review_allowed);
    assert!(!result.hotspot_review_allowed);
}

#[test]
fn normal_case_allows_cleanup_and_hotspot() {
    let signals = default_signals();
    let result = determine_action_eligibility(&signals, &low_risk_repo());
    assert_eq!(result.forced_action, None);
    assert!(result.cleanup_review_allowed);
    assert!(result.hotspot_review_allowed);
}

#[test]
fn cleanup_blocked_when_no_unused_files() {
    let mut signals = default_signals();
    signals.unused_files = 0;
    let result = determine_action_eligibility(&signals, &low_risk_repo());
    assert_eq!(result.forced_action, None);
    assert!(!result.cleanup_review_allowed);
    assert!(result.hotspot_review_allowed);
}

#[test]
fn cleanup_blocked_when_snapshot_stale() {
    let mut signals = default_signals();
    signals.snapshot_stale = true;
    let result = determine_action_eligibility(&signals, &low_risk_repo());
    assert!(!result.cleanup_review_allowed);
}

#[test]
fn hotspot_blocked_when_activity_stale() {
    let mut signals = default_signals();
    signals.activity_stale = true;
    let result = determine_action_eligibility(&signals, &low_risk_repo());
    assert!(result.cleanup_review_allowed);
    assert!(!result.hotspot_review_allowed);
}

#[test]
fn cleanup_blocked_when_gate_blocked() {
    let mut signals = default_signals();
    signals.cleanup_gate_level = GateLevel::Blocked;
    let result = determine_action_eligibility(&signals, &low_risk_repo());
    assert!(!result.cleanup_review_allowed);
}

#[test]
fn hotspot_blocked_when_gate_blocked() {
    let mut signals = default_signals();
    signals.refactor_gate_level = GateLevel::Blocked;
    let result = determine_action_eligibility(&signals, &low_risk_repo());
    assert!(!result.hotspot_review_allowed);
}

#[test]
fn both_blocked_when_monitoring_inactive() {
    let mut signals = default_signals();
    signals.monitoring_active = false;
    let result = determine_action_eligibility(&signals, &low_risk_repo());
    assert!(!result.cleanup_review_allowed);
    assert!(!result.hotspot_review_allowed);
}

#[test]
fn both_blocked_when_no_snapshot() {
    let mut signals = default_signals();
    signals.snapshot_available = false;
    let result = determine_action_eligibility(&signals, &low_risk_repo());
    assert!(!result.cleanup_review_allowed);
    assert!(!result.hotspot_review_allowed);
}

#[test]
fn hotspot_blocked_when_no_activity() {
    let mut signals = default_signals();
    signals.activity_available = false;
    let result = determine_action_eligibility(&signals, &low_risk_repo());
    assert!(!result.hotspot_review_allowed);
}

#[test]
fn verification_failing_takes_precedence_over_mid_operation() {
    let mut signals = default_signals();
    signals.verification_failing = true;
    let result = determine_action_eligibility(&signals, &mid_operation_repo());
    assert_eq!(result.forced_action, Some("review_failing_verification"));
}

#[test]
fn verification_missing_takes_precedence_over_mid_operation() {
    let mut signals = default_signals();
    signals.verification_missing = true;
    let result = determine_action_eligibility(&signals, &mid_operation_repo());
    assert_eq!(
        result.forced_action,
        Some("run_verification_before_high_risk_changes")
    );
}

#[test]
fn default_eligibility_result_is_all_disabled() {
    let default = EligibilityResult::default();
    assert_eq!(default.forced_action, None);
    assert!(!default.cleanup_review_allowed);
    assert!(!default.hotspot_review_allowed);
}

#[test]
fn repo_with_empty_operation_states_is_normal() {
    let repo = json!({
        "risk_level": "low",
        "operation_states": [],
    });
    let signals = default_signals();
    let result = determine_action_eligibility(&signals, &repo);
    assert_eq!(result.forced_action, None);
}

#[test]
fn repo_without_operation_states_key_is_normal() {
    let repo = json!({
        "risk_level": "low",
    });
    let signals = default_signals();
    let result = determine_action_eligibility(&signals, &repo);
    assert_eq!(result.forced_action, None);
}
