use super::*;

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
