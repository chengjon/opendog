use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecisionAction {
    ReviewFailingVerification,
    StabilizeRepositoryState,
    EvidenceCollection,
    RunVerificationBeforeHighRiskChanges,
    ReviewUnusedFiles,
    InspectHotFiles,
    WorkspaceTriage,
}

impl DecisionAction {
    fn from_name(action: &str) -> Self {
        match action {
            "review_failing_verification" => Self::ReviewFailingVerification,
            "stabilize_repository_state" => Self::StabilizeRepositoryState,
            "start_monitor" | "take_snapshot" | "generate_activity_then_stats" => {
                Self::EvidenceCollection
            }
            "run_verification_before_high_risk_changes" => {
                Self::RunVerificationBeforeHighRiskChanges
            }
            "review_unused_files" => Self::ReviewUnusedFiles,
            "inspect_hot_files" => Self::InspectHotFiles,
            _ => Self::WorkspaceTriage,
        }
    }

    fn action_class(self) -> DecisionActionClass {
        match self {
            Self::ReviewFailingVerification => DecisionActionClass::VerificationRecovery,
            Self::StabilizeRepositoryState => DecisionActionClass::RepositoryStabilization,
            Self::EvidenceCollection => DecisionActionClass::EvidenceCollection,
            Self::RunVerificationBeforeHighRiskChanges => {
                DecisionActionClass::VerificationCollection
            }
            Self::ReviewUnusedFiles => DecisionActionClass::CleanupReview,
            Self::InspectHotFiles => DecisionActionClass::RefactorReview,
            Self::WorkspaceTriage => DecisionActionClass::WorkspaceTriage,
        }
    }

    fn phase(self) -> DecisionActionPhase {
        match self {
            Self::ReviewFailingVerification | Self::StabilizeRepositoryState => {
                DecisionActionPhase::Stabilize
            }
            Self::EvidenceCollection => DecisionActionPhase::Observe,
            Self::RunVerificationBeforeHighRiskChanges => DecisionActionPhase::Verify,
            Self::ReviewUnusedFiles | Self::InspectHotFiles => DecisionActionPhase::Review,
            Self::WorkspaceTriage => DecisionActionPhase::Triage,
        }
    }

    fn mutability_scope(self) -> DecisionMutabilityScope {
        match self {
            Self::ReviewFailingVerification
            | Self::StabilizeRepositoryState
            | Self::RunVerificationBeforeHighRiskChanges => DecisionMutabilityScope::ReadMostly,
            Self::EvidenceCollection => DecisionMutabilityScope::NonCodeStateChange,
            Self::ReviewUnusedFiles | Self::InspectHotFiles => {
                DecisionMutabilityScope::ReviewBeforeModify
            }
            Self::WorkspaceTriage => DecisionMutabilityScope::ReadOnly,
        }
    }

    fn verification_required(self) -> bool {
        matches!(
            self,
            Self::ReviewFailingVerification
                | Self::StabilizeRepositoryState
                | Self::RunVerificationBeforeHighRiskChanges
        )
    }

    fn primary_goal(self) -> &'static str {
        match self {
            Self::ReviewFailingVerification => {
                "stabilize failing or uncertain evidence before broader edits"
            }
            Self::StabilizeRepositoryState => {
                "resolve in-progress repository state before broader changes"
            }
            Self::EvidenceCollection => "collect missing activity or inventory evidence",
            Self::RunVerificationBeforeHighRiskChanges => {
                "record test/lint/build evidence before risky work"
            }
            Self::ReviewUnusedFiles => "inspect unused-file candidates before cleanup",
            Self::InspectHotFiles => "inspect activity hotspots before targeted refactor",
            Self::WorkspaceTriage => "choose the next project or tool path",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DecisionActionClass {
    VerificationRecovery,
    RepositoryStabilization,
    EvidenceCollection,
    VerificationCollection,
    CleanupReview,
    RefactorReview,
    WorkspaceTriage,
}

impl DecisionActionClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::VerificationRecovery => "verification_recovery",
            Self::RepositoryStabilization => "repository_stabilization",
            Self::EvidenceCollection => "evidence_collection",
            Self::VerificationCollection => "verification_collection",
            Self::CleanupReview => "cleanup_review",
            Self::RefactorReview => "refactor_review",
            Self::WorkspaceTriage => "workspace_triage",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DecisionActionPhase {
    Stabilize,
    Observe,
    Verify,
    Review,
    Triage,
}

impl DecisionActionPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Stabilize => "stabilize",
            Self::Observe => "observe",
            Self::Verify => "verify",
            Self::Review => "review",
            Self::Triage => "triage",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DecisionMutabilityScope {
    ReadMostly,
    NonCodeStateChange,
    ReviewBeforeModify,
    ReadOnly,
}

impl DecisionMutabilityScope {
    fn as_str(self) -> &'static str {
        match self {
            Self::ReadMostly => "read_mostly",
            Self::NonCodeStateChange => "non_code_state_change",
            Self::ReviewBeforeModify => "review_before_modify",
            Self::ReadOnly => "read_only",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DecisionActionProfile {
    action: DecisionAction,
    strategy_mode: String,
}

impl DecisionActionProfile {
    pub(super) fn from_action(action: &str, strategy_mode: &str) -> Self {
        Self {
            action: DecisionAction::from_name(action),
            strategy_mode: strategy_mode.to_string(),
        }
    }

    #[cfg(test)]
    pub(super) fn action_class(&self) -> DecisionActionClass {
        self.action.action_class()
    }

    #[cfg(test)]
    pub(super) fn phase(&self) -> DecisionActionPhase {
        self.action.phase()
    }

    #[cfg(test)]
    pub(super) fn mutability_scope(&self) -> DecisionMutabilityScope {
        self.action.mutability_scope()
    }

    #[cfg(test)]
    pub(super) fn verification_required(&self) -> bool {
        self.action.verification_required()
    }

    #[cfg(test)]
    pub(super) fn primary_goal(&self) -> &'static str {
        self.action.primary_goal()
    }

    pub(super) fn to_json(&self) -> Value {
        json!({
            "action_class": self.action.action_class().as_str(),
            "phase": self.action.phase().as_str(),
            "mutability_scope": self.action.mutability_scope().as_str(),
            "verification_required": self.action.verification_required(),
            "strategy_mode": self.strategy_mode,
            "primary_goal": self.action.primary_goal(),
        })
    }
}

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
pub(super) struct DecisionRiskProfile {
    action: DecisionAction,
    context: DecisionRiskContext,
}

impl DecisionRiskProfile {
    pub(super) fn from_overview(
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

    pub(super) fn to_json(&self) -> Value {
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
    use super::{
        DecisionActionClass, DecisionActionPhase, DecisionActionProfile, DecisionMutabilityScope,
        DecisionRiskContext, DecisionRiskProfile, DecisionRiskTier,
    };

    #[test]
    fn action_profile_model_groups_evidence_collection_actions() {
        for action in [
            "start_monitor",
            "take_snapshot",
            "generate_activity_then_stats",
        ] {
            let profile = DecisionActionProfile::from_action(action, "standard");

            assert_eq!(
                profile.action_class(),
                DecisionActionClass::EvidenceCollection
            );
            assert_eq!(profile.phase(), DecisionActionPhase::Observe);
            assert_eq!(
                profile.mutability_scope(),
                DecisionMutabilityScope::NonCodeStateChange
            );
            assert!(!profile.verification_required());
        }
    }

    #[test]
    fn action_profile_model_marks_recovery_actions_as_verification_required() {
        let profile = DecisionActionProfile::from_action("review_failing_verification", "standard");

        assert_eq!(
            profile.action_class(),
            DecisionActionClass::VerificationRecovery
        );
        assert_eq!(profile.phase(), DecisionActionPhase::Stabilize);
        assert_eq!(
            profile.mutability_scope(),
            DecisionMutabilityScope::ReadMostly
        );
        assert!(profile.verification_required());
        assert!(profile.primary_goal().contains("failing"));
    }

    #[test]
    fn action_profile_model_renders_json_contract() {
        let profile = DecisionActionProfile::from_action("inspect_hot_files", "conservative");
        let json = profile.to_json();

        assert_eq!(json["action_class"], "refactor_review");
        assert_eq!(json["phase"], "review");
        assert_eq!(json["mutability_scope"], "review_before_modify");
        assert_eq!(json["verification_required"], false);
        assert_eq!(json["strategy_mode"], "conservative");
    }

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
