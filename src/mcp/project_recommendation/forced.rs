use serde_json::Value;

use super::super::guidance_types::Recommendation;
use super::super::strategy_profile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ForcedRecommendationAction {
    ReviewFailingVerification,
    RunVerificationBeforeHighRiskChanges,
    StabilizeRepositoryState,
}

impl ForcedRecommendationAction {
    pub(super) fn from_name(action: &str) -> Option<Self> {
        match action {
            "review_failing_verification" => Some(Self::ReviewFailingVerification),
            "run_verification_before_high_risk_changes" => {
                Some(Self::RunVerificationBeforeHighRiskChanges)
            }
            "stabilize_repository_state" => Some(Self::StabilizeRepositoryState),
            _ => None,
        }
    }

    fn recommended_next_action(self) -> &'static str {
        match self {
            Self::ReviewFailingVerification => "review_failing_verification",
            Self::RunVerificationBeforeHighRiskChanges => {
                "run_verification_before_high_risk_changes"
            }
            Self::StabilizeRepositoryState => "stabilize_repository_state",
        }
    }

    fn recommended_flow(self) -> Vec<String> {
        match self {
            Self::ReviewFailingVerification => vec![
                "Inspect the latest failing or uncertain verification evidence first.".to_string(),
                "Use shell diff and project-native verification commands to stabilize the project."
                    .to_string(),
                "Return to cleanup or refactor review only after verification is passing again."
                    .to_string(),
            ],
            Self::RunVerificationBeforeHighRiskChanges => vec![
                "Run and record project-native verification before risky changes.".to_string(),
                "Use OPENDOG to persist the resulting evidence for later decisions.".to_string(),
                "Return to cleanup or refactor review only after verification evidence exists."
                    .to_string(),
            ],
            Self::StabilizeRepositoryState => vec![
                "Stabilize the repository before broader code changes.".to_string(),
                "Use git status and diff to understand the in-progress operation.".to_string(),
                "Only return to OPENDOG-guided cleanup or review after the repository state is stable."
                    .to_string(),
            ],
        }
    }

    fn reason(self, verification_missing: bool) -> &'static str {
        match self {
            Self::ReviewFailingVerification => {
                "Recent verification evidence includes failing or uncertain runs, so review and stabilize those results before broader cleanup or refactoring."
            }
            Self::RunVerificationBeforeHighRiskChanges if verification_missing => {
                "Activity evidence exists, but no recorded test/lint/build results are available yet. Verify first before risky cleanup or refactor work."
            }
            Self::RunVerificationBeforeHighRiskChanges => {
                "Recorded verification evidence exists but is stale, so refresh test/lint/build results before risky cleanup or refactor work."
            }
            Self::StabilizeRepositoryState => {
                "The repository is mid-operation (merge/rebase/cherry-pick/bisect), so avoid broad modifications until that state is resolved."
            }
        }
    }

    fn strategy_mode(self) -> &'static str {
        match self {
            Self::ReviewFailingVerification | Self::RunVerificationBeforeHighRiskChanges => {
                "verify_before_modify"
            }
            Self::StabilizeRepositoryState => "stabilize_before_modify",
        }
    }

    fn confidence(self) -> &'static str {
        match self {
            Self::ReviewFailingVerification | Self::StabilizeRepositoryState => "high",
            Self::RunVerificationBeforeHighRiskChanges => "medium",
        }
    }

    fn evidence_priority(self) -> &'static [&'static str] {
        match self {
            Self::ReviewFailingVerification => {
                &["verification", "repository_risk", "activity_signals"]
            }
            Self::RunVerificationBeforeHighRiskChanges => {
                &["verification", "activity_signals", "repository_risk"]
            }
            Self::StabilizeRepositoryState => {
                &["repository_risk", "verification", "activity_signals"]
            }
        }
    }

    fn suggested_commands(self, context: &ForcedRecommendationContext) -> Vec<String> {
        match self {
            Self::ReviewFailingVerification => vec![
                "opendog verification --id <project>".to_string(),
                context.primary_verification_command.clone(),
                "git diff".to_string(),
            ],
            Self::RunVerificationBeforeHighRiskChanges => vec![
                context.primary_verification_command.clone(),
                "opendog run-verification --id <project> --kind test --command '<cmd>'".to_string(),
                format!("opendog stats --id {}", context.project_id),
            ],
            Self::StabilizeRepositoryState => vec![
                "git status".to_string(),
                "git diff".to_string(),
                "opendog verification --id <project>".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ForcedRecommendationContext {
    pub(super) project_id: String,
    pub(super) primary_verification_command: String,
    pub(super) verification_missing: bool,
    pub(super) verification_gate_levels: Value,
    pub(super) cleanup_blockers: Vec<Value>,
    pub(super) refactor_blockers: Vec<Value>,
    pub(super) repo_truth_gaps: Value,
    pub(super) mandatory_shell_checks: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ForcedProjectRecommendation {
    action: ForcedRecommendationAction,
    context: ForcedRecommendationContext,
}

impl ForcedProjectRecommendation {
    pub(super) fn from_action(action: &str, context: ForcedRecommendationContext) -> Option<Self> {
        Some(Self {
            action: ForcedRecommendationAction::from_name(action)?,
            context,
        })
    }

    pub(super) fn from_immediate_action(
        action: &str,
        context: ForcedRecommendationContext,
    ) -> Option<Self> {
        let recommendation = Self::from_action(action, context)?;
        match recommendation.action {
            ForcedRecommendationAction::RunVerificationBeforeHighRiskChanges => None,
            _ => Some(recommendation),
        }
    }

    pub(super) fn into_recommendation(self) -> Recommendation {
        let Self { action, context } = self;
        let suggested_commands = action.suggested_commands(&context);
        let cleanup_blockers = Some(Value::Array(context.cleanup_blockers));
        let refactor_blockers = Some(Value::Array(context.refactor_blockers));

        Recommendation {
            project_id: context.project_id,
            recommended_next_action: action.recommended_next_action().to_string(),
            recommended_flow: action.recommended_flow(),
            reason: action.reason(context.verification_missing).to_string(),
            confidence: action.confidence().to_string(),
            strategy_mode: action.strategy_mode().to_string(),
            strategy_profile: strategy_profile(
                action.strategy_mode(),
                "shell",
                "opendog",
                action.evidence_priority(),
            ),
            verification_gate_levels: context.verification_gate_levels,
            cleanup_blockers,
            refactor_blockers,
            repo_truth_gaps: context.repo_truth_gaps,
            mandatory_shell_checks: context.mandatory_shell_checks,
            suggested_commands,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ForcedProjectRecommendation, ForcedRecommendationAction, ForcedRecommendationContext,
    };

    fn context_with_verification_missing(
        verification_missing: bool,
    ) -> ForcedRecommendationContext {
        ForcedRecommendationContext {
            project_id: "alpha".to_string(),
            primary_verification_command: "cargo test".to_string(),
            verification_missing,
            verification_gate_levels: json!({ "cleanup": "blocked", "refactor": "allow" }),
            cleanup_blockers: vec![json!("cleanup blocker")],
            refactor_blockers: vec![json!("refactor blocker")],
            repo_truth_gaps: json!(["repo truth gap"]),
            mandatory_shell_checks: json!(["git status"]),
        }
    }

    fn context() -> ForcedRecommendationContext {
        context_with_verification_missing(true)
    }

    #[test]
    fn forced_recommendation_builds_failing_verification_contract() {
        let recommendation =
            ForcedProjectRecommendation::from_action("review_failing_verification", context())
                .unwrap()
                .into_recommendation();

        assert_eq!(
            recommendation.recommended_next_action,
            "review_failing_verification"
        );
        assert_eq!(recommendation.strategy_mode, "verify_before_modify");
        assert_eq!(recommendation.suggested_commands[1], "cargo test");
        assert_eq!(
            recommendation.cleanup_blockers.unwrap()[0],
            "cleanup blocker"
        );
    }

    #[test]
    fn forced_recommendation_builds_stabilize_repository_contract() {
        let recommendation =
            ForcedProjectRecommendation::from_action("stabilize_repository_state", context())
                .unwrap()
                .into_recommendation();

        assert_eq!(
            recommendation.recommended_next_action,
            "stabilize_repository_state"
        );
        assert_eq!(recommendation.strategy_mode, "stabilize_before_modify");
        assert_eq!(recommendation.suggested_commands[0], "git status");
        assert!(recommendation.reason.contains("mid-operation"));
    }

    #[test]
    fn forced_recommendation_builds_run_verification_contract() {
        let recommendation = ForcedProjectRecommendation::from_action(
            "run_verification_before_high_risk_changes",
            context(),
        )
        .unwrap()
        .into_recommendation();

        assert_eq!(
            recommendation.recommended_next_action,
            "run_verification_before_high_risk_changes"
        );
        assert_eq!(recommendation.strategy_mode, "verify_before_modify");
        assert_eq!(recommendation.confidence, "medium");
        assert!(recommendation
            .reason
            .contains("no recorded test/lint/build"));
        assert_eq!(
            recommendation.suggested_commands[1],
            "opendog run-verification --id <project> --kind test --command '<cmd>'"
        );
        assert_eq!(
            recommendation.suggested_commands[2],
            "opendog stats --id alpha"
        );
    }

    #[test]
    fn forced_recommendation_reports_stale_verification_when_evidence_exists() {
        let recommendation = ForcedProjectRecommendation::from_action(
            "run_verification_before_high_risk_changes",
            context_with_verification_missing(false),
        )
        .unwrap()
        .into_recommendation();

        assert!(recommendation
            .reason
            .contains("verification evidence exists"));
        assert!(recommendation.reason.contains("stale"));
    }

    #[test]
    fn immediate_forced_recommendation_defers_verification_before_high_risk_action() {
        assert!(ForcedProjectRecommendation::from_immediate_action(
            "review_failing_verification",
            context()
        )
        .is_some());
        assert!(ForcedProjectRecommendation::from_immediate_action(
            "run_verification_before_high_risk_changes",
            context()
        )
        .is_none());
    }

    #[test]
    fn forced_recommendation_ignores_non_forced_actions() {
        let recommendation =
            ForcedProjectRecommendation::from_action("inspect_hot_files", context());

        assert!(recommendation.is_none());
        assert_eq!(
            ForcedRecommendationAction::from_name("inspect_hot_files"),
            None
        );
    }
}
