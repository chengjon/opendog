use serde_json::Value;

use super::super::guidance_types::Recommendation;
use super::super::strategy_profile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProjectReviewAction {
    ReviewUnusedFiles,
    InspectHotFiles,
}

impl ProjectReviewAction {
    fn recommended_next_action(self) -> &'static str {
        match self {
            Self::ReviewUnusedFiles => "review_unused_files",
            Self::InspectHotFiles => "inspect_hot_files",
        }
    }

    fn recommended_flow(self) -> Vec<String> {
        match self {
            Self::ReviewUnusedFiles => vec![
                "Inspect unused-file candidates before proposing cleanup.".to_string(),
                "Validate each candidate with shell search, imports, and tests.".to_string(),
                "Only delete or refactor after cleanup blockers are cleared.".to_string(),
            ],
            Self::InspectHotFiles => vec![
                "Inspect the hottest observed files first.".to_string(),
                "Use shell diff and symbol search after OPENDOG narrows the review target."
                    .to_string(),
                "Treat hotspot review as a precursor to targeted refactor, not broad cleanup."
                    .to_string(),
            ],
        }
    }

    fn strategy_mode(self, context: &ProjectReviewContext) -> &'static str {
        match self {
            Self::ReviewUnusedFiles if context.safe_for_cleanup => "review_then_modify",
            Self::ReviewUnusedFiles => "verify_before_modify",
            Self::InspectHotFiles if context.safe_for_refactor => "inspect_then_modify",
            Self::InspectHotFiles => "verify_before_modify",
        }
    }

    fn profile_tools(self, context: &ProjectReviewContext) -> (&'static str, &'static str) {
        match self {
            Self::ReviewUnusedFiles => ("opendog", "shell"),
            Self::InspectHotFiles if context.safe_for_refactor => ("opendog", "shell"),
            Self::InspectHotFiles => ("shell", "opendog"),
        }
    }

    fn suggested_commands(self, context: &ProjectReviewContext) -> Vec<String> {
        match self {
            Self::ReviewUnusedFiles => vec![
                format!("opendog unused --id {}", context.project_id),
                "rg \"<pattern>\" .".to_string(),
                context.project_command.clone(),
            ],
            Self::InspectHotFiles => vec![
                format!("opendog stats --id {}", context.project_id),
                "git diff".to_string(),
                "rg \"<pattern>\" .".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ProjectReviewContext {
    pub(super) project_id: String,
    pub(super) project_command: String,
    pub(super) reason: String,
    pub(super) confidence: String,
    pub(super) safe_for_cleanup: bool,
    pub(super) safe_for_refactor: bool,
    pub(super) verification_gate_levels: Value,
    pub(super) cleanup_blockers: Vec<Value>,
    pub(super) refactor_blockers: Vec<Value>,
    pub(super) repo_truth_gaps: Value,
    pub(super) mandatory_shell_checks: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ProjectReviewRecommendation {
    action: ProjectReviewAction,
    context: ProjectReviewContext,
}

impl ProjectReviewRecommendation {
    pub(super) fn new(action: ProjectReviewAction, context: ProjectReviewContext) -> Self {
        Self { action, context }
    }

    pub(super) fn into_recommendation(self) -> Recommendation {
        let Self { action, context } = self;
        let strategy_mode = action.strategy_mode(&context);
        let (primary_tool, secondary_tool) = action.profile_tools(&context);
        let suggested_commands = action.suggested_commands(&context);

        Recommendation {
            project_id: context.project_id,
            recommended_next_action: action.recommended_next_action().to_string(),
            recommended_flow: action.recommended_flow(),
            reason: context.reason,
            confidence: context.confidence,
            strategy_mode: strategy_mode.to_string(),
            strategy_profile: strategy_profile(
                strategy_mode,
                primary_tool,
                secondary_tool,
                &["activity_signals", "verification", "repository_risk"],
            ),
            verification_gate_levels: context.verification_gate_levels,
            cleanup_blockers: Some(Value::Array(context.cleanup_blockers)),
            refactor_blockers: Some(Value::Array(context.refactor_blockers)),
            repo_truth_gaps: context.repo_truth_gaps,
            mandatory_shell_checks: context.mandatory_shell_checks,
            suggested_commands,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{ProjectReviewAction, ProjectReviewContext, ProjectReviewRecommendation};

    fn context() -> ProjectReviewContext {
        ProjectReviewContext {
            project_id: "alpha".to_string(),
            project_command: "cargo test".to_string(),
            reason: "Current evidence favors this review.".to_string(),
            confidence: "high".to_string(),
            safe_for_cleanup: true,
            safe_for_refactor: true,
            verification_gate_levels: json!({ "cleanup": "allow", "refactor": "allow" }),
            cleanup_blockers: vec![json!("cleanup blocker")],
            refactor_blockers: vec![json!("refactor blocker")],
            repo_truth_gaps: json!(["repo truth gap"]),
            mandatory_shell_checks: json!(["git status"]),
        }
    }

    #[test]
    fn review_recommendation_builds_unused_file_contract() {
        let recommendation =
            ProjectReviewRecommendation::new(ProjectReviewAction::ReviewUnusedFiles, context())
                .into_recommendation();

        assert_eq!(
            recommendation.recommended_next_action,
            "review_unused_files"
        );
        assert_eq!(recommendation.strategy_mode, "review_then_modify");
        assert_eq!(recommendation.confidence, "high");
        assert_eq!(
            recommendation.suggested_commands,
            vec![
                "opendog unused --id alpha",
                "rg \"<pattern>\" .",
                "cargo test"
            ]
        );
        assert_eq!(
            recommendation.cleanup_blockers.unwrap()[0],
            "cleanup blocker"
        );
    }

    #[test]
    fn review_recommendation_builds_hot_file_contract() {
        let recommendation =
            ProjectReviewRecommendation::new(ProjectReviewAction::InspectHotFiles, context())
                .into_recommendation();

        assert_eq!(recommendation.recommended_next_action, "inspect_hot_files");
        assert_eq!(recommendation.strategy_mode, "inspect_then_modify");
        assert_eq!(
            recommendation.suggested_commands,
            vec!["opendog stats --id alpha", "git diff", "rg \"<pattern>\" ."]
        );
        assert_eq!(
            recommendation.strategy_profile["preferred_primary_tool"],
            "opendog"
        );
    }

    #[test]
    fn review_recommendation_verifies_before_modify_when_review_is_not_safe() {
        let mut context = context();
        context.safe_for_cleanup = false;
        context.safe_for_refactor = false;

        let cleanup = ProjectReviewRecommendation::new(
            ProjectReviewAction::ReviewUnusedFiles,
            context.clone(),
        )
        .into_recommendation();
        let refactor =
            ProjectReviewRecommendation::new(ProjectReviewAction::InspectHotFiles, context)
                .into_recommendation();

        assert_eq!(cleanup.strategy_mode, "verify_before_modify");
        assert_eq!(refactor.strategy_mode, "verify_before_modify");
        assert_eq!(refactor.strategy_profile["preferred_primary_tool"], "shell");
    }
}
