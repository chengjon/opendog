use serde_json::Value;

mod model;

use model::{DecisionActionProfile, DecisionRiskProfile};

pub(in crate::mcp) fn decision_action_profile(action: &str, strategy_mode: &str) -> Value {
    DecisionActionProfile::from_action(action, strategy_mode).to_json()
}

pub(in crate::mcp) fn decision_risk_profile(
    action: &str,
    matched_overview: &Value,
    verification_status: &str,
    safe_for_cleanup: Option<bool>,
    safe_for_refactor: Option<bool>,
) -> Value {
    DecisionRiskProfile::from_overview(
        action,
        matched_overview,
        verification_status,
        safe_for_cleanup,
        safe_for_refactor,
    )
    .to_json()
}

#[cfg(test)]
mod tests;
