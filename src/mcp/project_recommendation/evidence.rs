use serde_json::Value;

use super::super::guidance_types::Recommendation;
use super::super::strategy_profile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EvidenceCollectionAction {
    StartMonitor,
    TakeSnapshot,
    GenerateActivityThenStats,
}

impl EvidenceCollectionAction {
    fn recommended_next_action(self) -> &'static str {
        match self {
            Self::StartMonitor => "start_monitor",
            Self::TakeSnapshot => "take_snapshot",
            Self::GenerateActivityThenStats => "generate_activity_then_stats",
        }
    }

    fn recommended_flow(self) -> Vec<String> {
        match self {
            Self::StartMonitor => vec![
                "Start monitoring because fresh activity evidence does not exist yet.".to_string(),
                "Let real workflow activity happen after monitoring is active.".to_string(),
                "Inspect stats only after OPENDOG has observed meaningful activity.".to_string(),
            ],
            Self::TakeSnapshot => vec![
                "Take a snapshot to establish the project baseline.".to_string(),
                "Use stats only after the baseline inventory exists.".to_string(),
                "If monitoring is already active, keep it running so activity can accumulate after snapshot."
                    .to_string(),
            ],
            Self::GenerateActivityThenStats => vec![
                "Generate real project activity with edits, tests, or builds.".to_string(),
                "Avoid drawing hotspot or cleanup conclusions before activity exists.".to_string(),
                "Inspect stats after the observation window is meaningful.".to_string(),
            ],
        }
    }

    fn reason(self, context: &EvidenceCollectionContext) -> &'static str {
        match self {
            Self::StartMonitor => {
                "This project is not currently being monitored, so opendog cannot collect fresh activity data yet."
            }
            Self::TakeSnapshot if context.snapshot_missing => {
                "Monitoring is active but no snapshot data exists yet, so file inventory and stats are incomplete."
            }
            Self::TakeSnapshot => {
                "Snapshot evidence exists but is stale, so refresh the baseline before trusting cleanup or hotspot conclusions."
            }
            Self::GenerateActivityThenStats if context.activity_missing => {
                "Snapshot data exists, but no file access activity has been recorded yet."
            }
            Self::GenerateActivityThenStats => {
                "Activity evidence exists but is stale, so generate fresh workflow activity before trusting current hotspot or cleanup signals."
            }
        }
    }

    fn profile_tools(self) -> (&'static str, &'static str) {
        match self {
            Self::StartMonitor | Self::TakeSnapshot => ("opendog", "shell"),
            Self::GenerateActivityThenStats => ("shell", "opendog"),
        }
    }

    fn evidence_priority(self) -> &'static [&'static str] {
        match self {
            Self::StartMonitor | Self::TakeSnapshot => &["activity_signals", "repository_risk"],
            Self::GenerateActivityThenStats => {
                &["activity_signals", "verification", "repository_risk"]
            }
        }
    }

    fn suggested_commands(self, context: &EvidenceCollectionContext) -> Vec<String> {
        match self {
            Self::StartMonitor => vec![
                format!("opendog start --id {}", context.project_id),
                format!("opendog stats --id {}", context.project_id),
            ],
            Self::TakeSnapshot => vec![
                format!("opendog snapshot --id {}", context.project_id),
                format!("opendog stats --id {}", context.project_id),
            ],
            Self::GenerateActivityThenStats => vec![
                context.project_command.clone(),
                format!("opendog stats --id {}", context.project_id),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct EvidenceCollectionContext {
    pub(super) project_id: String,
    pub(super) project_command: String,
    pub(super) snapshot_missing: bool,
    pub(super) activity_missing: bool,
    pub(super) verification_gate_levels: Value,
    pub(super) repo_truth_gaps: Value,
    pub(super) mandatory_shell_checks: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct EvidenceCollectionRecommendation {
    action: EvidenceCollectionAction,
    context: EvidenceCollectionContext,
}

impl EvidenceCollectionRecommendation {
    pub(super) fn new(
        action: EvidenceCollectionAction,
        context: EvidenceCollectionContext,
    ) -> Self {
        Self { action, context }
    }

    pub(super) fn into_recommendation(self) -> Recommendation {
        let Self { action, context } = self;
        let reason = action.reason(&context).to_string();
        let suggested_commands = action.suggested_commands(&context);
        let (primary_tool, secondary_tool) = action.profile_tools();

        Recommendation {
            project_id: context.project_id,
            recommended_next_action: action.recommended_next_action().to_string(),
            recommended_flow: action.recommended_flow(),
            reason,
            confidence: "medium".to_string(),
            strategy_mode: "collect_evidence_first".to_string(),
            strategy_profile: strategy_profile(
                "collect_evidence_first",
                primary_tool,
                secondary_tool,
                action.evidence_priority(),
            ),
            verification_gate_levels: context.verification_gate_levels,
            cleanup_blockers: None,
            refactor_blockers: None,
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
        EvidenceCollectionAction, EvidenceCollectionContext, EvidenceCollectionRecommendation,
    };

    fn context() -> EvidenceCollectionContext {
        EvidenceCollectionContext {
            project_id: "alpha".to_string(),
            project_command: "cargo test".to_string(),
            snapshot_missing: true,
            activity_missing: true,
            verification_gate_levels: json!({ "cleanup": "blocked", "refactor": "blocked" }),
            repo_truth_gaps: json!(["repo truth gap"]),
            mandatory_shell_checks: json!(["git status"]),
        }
    }

    #[test]
    fn evidence_recommendation_builds_start_monitor_contract() {
        let recommendation = EvidenceCollectionRecommendation::new(
            EvidenceCollectionAction::StartMonitor,
            context(),
        )
        .into_recommendation();

        assert_eq!(recommendation.recommended_next_action, "start_monitor");
        assert_eq!(recommendation.strategy_mode, "collect_evidence_first");
        assert_eq!(recommendation.confidence, "medium");
        assert_eq!(
            recommendation.suggested_commands,
            vec!["opendog start --id alpha", "opendog stats --id alpha"]
        );
        assert!(recommendation.cleanup_blockers.is_none());
        assert!(recommendation.refactor_blockers.is_none());
    }

    #[test]
    fn evidence_recommendation_builds_take_snapshot_contract() {
        let recommendation = EvidenceCollectionRecommendation::new(
            EvidenceCollectionAction::TakeSnapshot,
            context(),
        )
        .into_recommendation();

        assert_eq!(recommendation.recommended_next_action, "take_snapshot");
        assert!(recommendation.reason.contains("no snapshot data exists"));
        assert_eq!(
            recommendation.suggested_commands,
            vec!["opendog snapshot --id alpha", "opendog stats --id alpha"]
        );
    }

    #[test]
    fn evidence_recommendation_builds_generate_activity_contract() {
        let recommendation = EvidenceCollectionRecommendation::new(
            EvidenceCollectionAction::GenerateActivityThenStats,
            context(),
        )
        .into_recommendation();

        assert_eq!(
            recommendation.recommended_next_action,
            "generate_activity_then_stats"
        );
        assert!(recommendation.reason.contains("no file access activity"));
        assert_eq!(
            recommendation.suggested_commands,
            vec!["cargo test", "opendog stats --id alpha"]
        );
    }

    #[test]
    fn evidence_recommendation_reports_stale_snapshot_and_activity() {
        let mut context = context();
        context.snapshot_missing = false;
        context.activity_missing = false;

        let snapshot = EvidenceCollectionRecommendation::new(
            EvidenceCollectionAction::TakeSnapshot,
            context.clone(),
        )
        .into_recommendation();
        let activity = EvidenceCollectionRecommendation::new(
            EvidenceCollectionAction::GenerateActivityThenStats,
            context,
        )
        .into_recommendation();

        assert!(snapshot
            .reason
            .contains("Snapshot evidence exists but is stale"));
        assert!(activity
            .reason
            .contains("Activity evidence exists but is stale"));
    }
}
