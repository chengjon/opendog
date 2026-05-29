use serde_json::{json, Value};

use super::DecisionAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DecisionRiskTier {
    High,
    Medium,
    Low,
}

impl DecisionRiskTier {
    fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct DecisionRiskContext {
    repo_risk_level: String,
    verification_status: String,
    safe_for_cleanup: Option<bool>,
    safe_for_refactor: Option<bool>,
    cleanup_gate_level: String,
    refactor_gate_level: String,
    cleanup_blockers: Vec<Value>,
    refactor_blockers: Vec<Value>,
    primary_repo_risk_finding: Value,
    repo_risk_findings: Vec<Value>,
    repo_risk_finding_counts: Value,
}

impl DecisionRiskContext {
    fn from_overview(
        matched_overview: &Value,
        verification_status: &str,
        safe_for_cleanup: Option<bool>,
        safe_for_refactor: Option<bool>,
    ) -> Self {
        let repo_risk = &matched_overview["repo_status_risk"];

        Self {
            repo_risk_level: repo_risk["risk_level"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            verification_status: verification_status.to_string(),
            safe_for_cleanup,
            safe_for_refactor,
            cleanup_gate_level: Self::gate_level(matched_overview, "cleanup", safe_for_cleanup),
            refactor_gate_level: Self::gate_level(matched_overview, "refactor", safe_for_refactor),
            cleanup_blockers: matched_overview["cleanup_blockers"]
                .as_array()
                .cloned()
                .unwrap_or_default(),
            refactor_blockers: matched_overview["refactor_blockers"]
                .as_array()
                .cloned()
                .unwrap_or_default(),
            primary_repo_risk_finding: repo_risk["highest_priority_finding"].clone(),
            repo_risk_findings: repo_risk["risk_findings"]
                .as_array()
                .cloned()
                .unwrap_or_default(),
            repo_risk_finding_counts: repo_risk["finding_counts"].clone(),
        }
    }

    fn gate_level(matched_overview: &Value, target: &str, ready: Option<bool>) -> String {
        matched_overview["verification_evidence"]["gate_assessment"][target]["level"]
            .as_str()
            .unwrap_or(if ready.unwrap_or(false) {
                "allow"
            } else {
                "blocked"
            })
            .to_string()
    }

    fn gate_is_cautious(&self, action: DecisionAction) -> bool {
        match action {
            DecisionAction::ReviewUnusedFiles => self.cleanup_gate_level != "allow",
            DecisionAction::InspectHotFiles => self.refactor_gate_level != "allow",
            _ => false,
        }
    }

    #[cfg(test)]
    pub(super) fn for_test(
        repo_risk_level: &str,
        cleanup_gate_level: &str,
        refactor_gate_level: &str,
        verification_status: &str,
    ) -> Self {
        Self {
            repo_risk_level: repo_risk_level.to_string(),
            verification_status: verification_status.to_string(),
            safe_for_cleanup: Some(cleanup_gate_level == "allow"),
            safe_for_refactor: Some(refactor_gate_level == "allow"),
            cleanup_gate_level: cleanup_gate_level.to_string(),
            refactor_gate_level: refactor_gate_level.to_string(),
            cleanup_blockers: Vec::new(),
            refactor_blockers: Vec::new(),
            primary_repo_risk_finding: Value::Null,
            repo_risk_findings: Vec::new(),
            repo_risk_finding_counts: Value::Null,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::mcp::decision_support::profiles) struct DecisionRiskProfile {
    action: DecisionAction,
    context: DecisionRiskContext,
}

impl DecisionRiskProfile {
    pub(in crate::mcp::decision_support::profiles) fn from_overview(
        action: &str,
        matched_overview: &Value,
        verification_status: &str,
        safe_for_cleanup: Option<bool>,
        safe_for_refactor: Option<bool>,
    ) -> Self {
        Self::new(
            action,
            DecisionRiskContext::from_overview(
                matched_overview,
                verification_status,
                safe_for_cleanup,
                safe_for_refactor,
            ),
        )
    }

    #[cfg(test)]
    pub(super) fn from_context(action: &str, context: DecisionRiskContext) -> Self {
        Self::new(action, context)
    }

    fn new(action: &str, context: DecisionRiskContext) -> Self {
        Self {
            action: DecisionAction::from_name(action),
            context,
        }
    }

    #[cfg(test)]
    pub(super) fn risk_tier(&self) -> DecisionRiskTier {
        self.assess_risk_tier()
    }

    fn assess_risk_tier(&self) -> DecisionRiskTier {
        match self.action {
            DecisionAction::ReviewFailingVerification
            | DecisionAction::StabilizeRepositoryState => DecisionRiskTier::High,
            DecisionAction::RunVerificationBeforeHighRiskChanges => DecisionRiskTier::Medium,
            DecisionAction::ReviewUnusedFiles | DecisionAction::InspectHotFiles => {
                if self.context.repo_risk_level == "high"
                    || self.context.verification_status != "available"
                    || self.context.gate_is_cautious(self.action)
                {
                    DecisionRiskTier::Medium
                } else {
                    DecisionRiskTier::Low
                }
            }
            _ => DecisionRiskTier::Low,
        }
    }

    #[cfg(test)]
    pub(super) fn manual_review_required(&self) -> bool {
        self.requires_manual_review()
    }

    fn requires_manual_review(&self) -> bool {
        matches!(
            self.action,
            DecisionAction::ReviewUnusedFiles | DecisionAction::InspectHotFiles
        )
    }

    pub(in crate::mcp::decision_support::profiles) fn to_json(&self) -> Value {
        json!({
            "risk_tier": self.assess_risk_tier().as_str(),
            "repo_risk_level": self.context.repo_risk_level,
            "verification_status": self.context.verification_status,
            "cleanup_ready": self.context.safe_for_cleanup,
            "refactor_ready": self.context.safe_for_refactor,
            "cleanup_gate_level": self.context.cleanup_gate_level,
            "refactor_gate_level": self.context.refactor_gate_level,
            "cleanup_blockers": self.context.cleanup_blockers.clone(),
            "refactor_blockers": self.context.refactor_blockers.clone(),
            "primary_repo_risk_finding": self.context.primary_repo_risk_finding.clone(),
            "repo_risk_findings": self.context.repo_risk_findings.clone(),
            "repo_risk_finding_counts": self.context.repo_risk_finding_counts.clone(),
            "destructive_change_recommended": false,
            "manual_review_required": self.requires_manual_review(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{DecisionRiskContext, DecisionRiskProfile, DecisionRiskTier};

    #[test]
    fn risk_profile_model_marks_stabilization_actions_high() {
        let context = DecisionRiskContext::for_test("low", "allow", "allow", "available");
        let profile = DecisionRiskProfile::from_context("stabilize_repository_state", context);

        assert_eq!(profile.risk_tier(), DecisionRiskTier::High);
        assert!(!profile.manual_review_required());
    }

    #[test]
    fn risk_profile_model_escalates_cleanup_review_when_gate_is_cautious() {
        let context = DecisionRiskContext::for_test("low", "blocked", "allow", "available");
        let profile = DecisionRiskProfile::from_context("review_unused_files", context);

        assert_eq!(profile.risk_tier(), DecisionRiskTier::Medium);
        assert!(profile.manual_review_required());
    }

    #[test]
    fn risk_profile_model_renders_json_contract() {
        let context = DecisionRiskContext::for_test("low", "allow", "allow", "available");
        let profile = DecisionRiskProfile::from_context("inspect_hot_files", context);
        let json = profile.to_json();

        assert_eq!(json["risk_tier"], "low");
        assert_eq!(json["repo_risk_level"], "low");
        assert_eq!(json["verification_status"], "available");
        assert_eq!(json["cleanup_gate_level"], "allow");
        assert_eq!(json["refactor_gate_level"], "allow");
        assert_eq!(json["destructive_change_recommended"], false);
        assert_eq!(json["manual_review_required"], true);
    }
}
