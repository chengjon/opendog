use serde_json::Value;

use super::super::guidance_types::Recommendation;
use super::super::strategy_profile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ForcedRecommendationAction {
    ReviewFailingVerification,
    StabilizeRepositoryState,
}

impl ForcedRecommendationAction {
    pub(super) fn from_name(action: &str) -> Option<Self> {
        match action {
            "review_failing_verification" => Some(Self::ReviewFailingVerification),
            "stabilize_repository_state" => Some(Self::StabilizeRepositoryState),
            _ => None,
        }
    }

    fn recommended_next_action(self) -> &'static str {
        match self {
            Self::ReviewFailingVerification => "review_failing_verification",
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
            Self::StabilizeRepositoryState => vec![
                "Stabilize the repository before broader code changes.".to_string(),
                "Use git status and diff to understand the in-progress operation.".to_string(),
                "Only return to OPENDOG-guided cleanup or review after the repository state is stable."
                    .to_string(),
            ],
        }
    }

    fn reason(self) -> &'static str {
        match self {
            Self::ReviewFailingVerification => {
                "Recent verification evidence includes failing or uncertain runs, so review and stabilize those results before broader cleanup or refactoring."
            }
            Self::StabilizeRepositoryState => {
                "The repository is mid-operation (merge/rebase/cherry-pick/bisect), so avoid broad modifications until that state is resolved."
            }
        }
    }

    fn strategy_mode(self) -> &'static str {
        match self {
            Self::ReviewFailingVerification => "verify_before_modify",
            Self::StabilizeRepositoryState => "stabilize_before_modify",
        }
    }

    fn evidence_priority(self) -> &'static [&'static str] {
        match self {
            Self::ReviewFailingVerification => {
                &["verification", "repository_risk", "activity_signals"]
            }
            Self::StabilizeRepositoryState => {
                &["repository_risk", "verification", "activity_signals"]
            }
        }
    }

    fn suggested_commands(self, primary_verification_command: &str) -> Vec<String> {
        match self {
            Self::ReviewFailingVerification => vec![
                "opendog verification --id <project>".to_string(),
                primary_verification_command.to_string(),
                "git diff".to_string(),
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
    project_id: String,
    primary_verification_command: String,
    verification_gate_levels: Value,
    cleanup_blockers: Vec<Value>,
    refactor_blockers: Vec<Value>,
    repo_truth_gaps: Value,
    mandatory_shell_checks: Value,
}

impl ForcedRecommendationContext {
    pub(super) fn new(
        project_id: String,
        primary_verification_command: String,
        verification_gate_levels: Value,
        cleanup_blockers: Vec<Value>,
        refactor_blockers: Vec<Value>,
        repo_truth_gaps: Value,
        mandatory_shell_checks: Value,
    ) -> Self {
        Self {
            project_id,
            primary_verification_command,
            verification_gate_levels,
            cleanup_blockers,
            refactor_blockers,
            repo_truth_gaps,
            mandatory_shell_checks,
        }
    }

    #[cfg(test)]
    pub(super) fn for_test(
        project_id: &str,
        primary_verification_command: &str,
        verification_gate_levels: Value,
        cleanup_blockers: Vec<Value>,
        refactor_blockers: Vec<Value>,
        repo_truth_gaps: Value,
        mandatory_shell_checks: Value,
    ) -> Self {
        Self::new(
            project_id.to_string(),
            primary_verification_command.to_string(),
            verification_gate_levels,
            cleanup_blockers,
            refactor_blockers,
            repo_truth_gaps,
            mandatory_shell_checks,
        )
    }
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

    pub(super) fn into_recommendation(self) -> Recommendation {
        let Self { action, context } = self;
        let cleanup_blockers = Some(Value::Array(context.cleanup_blockers));
        let refactor_blockers = Some(Value::Array(context.refactor_blockers));
        let suggested_commands = action.suggested_commands(&context.primary_verification_command);

        Recommendation {
            project_id: context.project_id,
            recommended_next_action: action.recommended_next_action().to_string(),
            recommended_flow: action.recommended_flow(),
            reason: action.reason().to_string(),
            confidence: "high".to_string(),
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

    fn context() -> ForcedRecommendationContext {
        ForcedRecommendationContext::for_test(
            "alpha",
            "cargo test",
            json!({ "cleanup": "blocked", "refactor": "allow" }),
            vec![json!("cleanup blocker")],
            vec![json!("refactor blocker")],
            json!(["repo truth gap"]),
            json!(["git status"]),
        )
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
