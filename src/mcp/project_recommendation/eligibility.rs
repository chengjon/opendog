use serde_json::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GateLevel {
    Allow,
    Caution,
    Blocked,
}

impl GateLevel {
    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "allow" => Self::Allow,
            "caution" => Self::Caution,
            _ => Self::Blocked,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RecommendationSignals {
    pub(crate) cleanup_gate_level: GateLevel,
    pub(crate) refactor_gate_level: GateLevel,
    pub(crate) monitoring_active: bool,
    pub(crate) snapshot_available: bool,
    pub(crate) activity_available: bool,
    pub(crate) snapshot_stale: bool,
    pub(crate) activity_stale: bool,
    pub(crate) verification_missing: bool,
    pub(crate) verification_stale: bool,
    pub(crate) verification_failing: bool,
    pub(crate) unused_files: i64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EligibilityResult {
    pub(crate) forced_action: Option<&'static str>,
    pub(crate) cleanup_review_allowed: bool,
    pub(crate) hotspot_review_allowed: bool,
}

pub(crate) fn determine_action_eligibility(
    signals: &RecommendationSignals,
    repo_risk: &Value,
) -> EligibilityResult {
    if signals.verification_failing {
        return EligibilityResult {
            forced_action: Some("review_failing_verification"),
            cleanup_review_allowed: false,
            hotspot_review_allowed: false,
        };
    }
    if signals.verification_missing || signals.verification_stale {
        return EligibilityResult {
            forced_action: Some("run_verification_before_high_risk_changes"),
            cleanup_review_allowed: false,
            hotspot_review_allowed: false,
        };
    }
    if repo_risk["operation_states"]
        .as_array()
        .map(|states| !states.is_empty())
        .unwrap_or(false)
    {
        return EligibilityResult {
            forced_action: Some("stabilize_repository_state"),
            cleanup_review_allowed: false,
            hotspot_review_allowed: false,
        };
    }

    EligibilityResult {
        forced_action: None,
        cleanup_review_allowed: signals.monitoring_active
            && signals.snapshot_available
            && !signals.snapshot_stale
            && signals.unused_files > 0
            && signals.cleanup_gate_level != GateLevel::Blocked,
        hotspot_review_allowed: signals.monitoring_active
            && signals.snapshot_available
            && signals.activity_available
            && !signals.activity_stale
            && signals.refactor_gate_level != GateLevel::Blocked,
    }
}
