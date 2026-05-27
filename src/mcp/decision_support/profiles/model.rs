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

#[cfg(test)]
mod tests {
    use super::{
        DecisionActionClass, DecisionActionPhase, DecisionActionProfile, DecisionMutabilityScope,
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
}
