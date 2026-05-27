use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StrategyTool {
    Opendog,
    Shell,
}

impl StrategyTool {
    fn as_str(self) -> &'static str {
        match self {
            Self::Opendog => "opendog",
            Self::Shell => "shell",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WorkspaceStrategyMode {
    VerifyBeforeModify,
    StabilizeBeforeModify,
    CollectWorkspaceContext,
    CollectEvidenceFirst,
    VerifyBeforeHighRiskChanges,
    ActivityGuidedReview,
}

impl WorkspaceStrategyMode {
    fn from_inputs(
        project_count: usize,
        monitoring_count: usize,
        has_failing_verification: bool,
        has_mid_operation_repo: bool,
        missing_verification_projects: usize,
    ) -> Self {
        if has_failing_verification {
            Self::VerifyBeforeModify
        } else if has_mid_operation_repo {
            Self::StabilizeBeforeModify
        } else if project_count == 0 {
            Self::CollectWorkspaceContext
        } else if monitoring_count == 0 {
            Self::CollectEvidenceFirst
        } else if missing_verification_projects > 0 {
            Self::VerifyBeforeHighRiskChanges
        } else {
            Self::ActivityGuidedReview
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::VerifyBeforeModify => "verify_before_modify",
            Self::StabilizeBeforeModify => "stabilize_before_modify",
            Self::CollectWorkspaceContext => "collect_workspace_context",
            Self::CollectEvidenceFirst => "collect_evidence_first",
            Self::VerifyBeforeHighRiskChanges => "verify_before_high_risk_changes",
            Self::ActivityGuidedReview => "activity_guided_review",
        }
    }

    fn preferred_tools(self) -> (StrategyTool, StrategyTool) {
        match self {
            Self::VerifyBeforeModify | Self::StabilizeBeforeModify => {
                (StrategyTool::Shell, StrategyTool::Opendog)
            }
            Self::CollectWorkspaceContext
            | Self::CollectEvidenceFirst
            | Self::VerifyBeforeHighRiskChanges
            | Self::ActivityGuidedReview => (StrategyTool::Opendog, StrategyTool::Shell),
        }
    }

    fn recommended_flow(self) -> [&'static str; 3] {
        match self {
            Self::VerifyBeforeModify => [
                "Inspect recorded failing verification first; do not start broad refactors while test/lint/build evidence is failing or uncertain.",
                "Use `opendog verification --id <project>` or `get_verification_status` to inspect the latest failing verification records.",
                "Only return to activity-based cleanup or refactor work after verification is stable again.",
            ],
            Self::StabilizeBeforeModify => [
                "Stabilize repositories that are mid-merge, rebase, cherry-pick, or bisect before making broader code changes.",
                "Use `git status` and `git diff` to understand the in-progress repository operation.",
                "Once repository state is stable, resume OPENDOG-driven cleanup or hotspot review.",
            ],
            Self::CollectWorkspaceContext => [
                "Register a project first with `register_project` or `opendog register --id <project> --path <root>`.",
                "Start monitoring immediately after creation so opendog can build activity data.",
                "Use shell commands such as `rg` only after you know which project root you want to inspect.",
            ],
            Self::CollectEvidenceFirst => [
                "Use `opendog list` first to pick a project that should be monitored.",
                "Use `opendog start --id <project>` to ensure monitoring is active; it can take an initial snapshot automatically.",
                "After some workflow activity, use `opendog stats --id <project>` to inspect hotspots.",
            ],
            Self::VerifyBeforeHighRiskChanges | Self::ActivityGuidedReview => [
                "Use `opendog list` first to confirm which projects are already being monitored.",
                "Use `opendog stats --id <project>` or the `get_stats` MCP tool after monitoring to inspect activity hotspots.",
                "Use `opendog unused --id <project>` or the `get_unused_files` MCP tool to review never-accessed files before cleanup.",
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkspaceStrategyProfile {
    mode: WorkspaceStrategyMode,
}

impl WorkspaceStrategyProfile {
    pub(super) fn from_inputs(
        project_count: usize,
        monitoring_count: usize,
        has_failing_verification: bool,
        has_mid_operation_repo: bool,
        missing_verification_projects: usize,
    ) -> Self {
        Self {
            mode: WorkspaceStrategyMode::from_inputs(
                project_count,
                monitoring_count,
                has_failing_verification,
                has_mid_operation_repo,
                missing_verification_projects,
            ),
        }
    }

    #[cfg(test)]
    pub(super) fn mode(&self) -> WorkspaceStrategyMode {
        self.mode
    }

    #[cfg(test)]
    pub(super) fn primary_tool(&self) -> StrategyTool {
        self.mode.preferred_tools().0
    }

    #[cfg(test)]
    pub(super) fn secondary_tool(&self) -> StrategyTool {
        self.mode.preferred_tools().1
    }

    #[cfg(test)]
    pub(super) fn recommended_flow(&self) -> Vec<String> {
        self.recommended_flow_strings()
    }

    fn evidence_priority() -> [&'static str; 3] {
        ["verification", "repository_risk", "activity_signals"]
    }

    fn recommended_flow_strings(&self) -> Vec<String> {
        self.mode
            .recommended_flow()
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    pub(super) fn to_json(&self) -> Value {
        let (preferred_primary_tool, preferred_secondary_tool) = self.mode.preferred_tools();

        json!({
            "global_strategy_mode": self.mode.as_str(),
            "preferred_primary_tool": preferred_primary_tool.as_str(),
            "preferred_secondary_tool": preferred_secondary_tool.as_str(),
            "evidence_priority": Self::evidence_priority(),
            "recommended_flow": self.recommended_flow_strings(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{StrategyTool, WorkspaceStrategyMode, WorkspaceStrategyProfile};

    #[test]
    fn model_prioritizes_failing_verification_over_repo_state() {
        let profile = WorkspaceStrategyProfile::from_inputs(3, 2, true, true, 0);

        assert_eq!(profile.mode(), WorkspaceStrategyMode::VerifyBeforeModify);
        assert_eq!(profile.primary_tool(), StrategyTool::Shell);
        assert_eq!(profile.secondary_tool(), StrategyTool::Opendog);
        assert!(profile.recommended_flow()[0].contains("failing verification"));
    }

    #[test]
    fn model_requires_verification_before_high_risk_changes_when_evidence_is_partial() {
        let profile = WorkspaceStrategyProfile::from_inputs(3, 2, false, false, 2);

        assert_eq!(
            profile.mode(),
            WorkspaceStrategyMode::VerifyBeforeHighRiskChanges
        );
        assert_eq!(profile.primary_tool(), StrategyTool::Opendog);
        assert_eq!(profile.secondary_tool(), StrategyTool::Shell);
    }

    #[test]
    fn model_renders_stable_json_contract() {
        let profile = WorkspaceStrategyProfile::from_inputs(3, 2, false, false, 0);
        let json = profile.to_json();

        assert_eq!(json["global_strategy_mode"], "activity_guided_review");
        assert_eq!(json["preferred_primary_tool"], "opendog");
        assert_eq!(json["preferred_secondary_tool"], "shell");
        assert_eq!(json["evidence_priority"].as_array().unwrap().len(), 3);
        assert_eq!(json["recommended_flow"].as_array().unwrap().len(), 3);
    }
}
