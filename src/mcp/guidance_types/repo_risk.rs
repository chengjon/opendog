use serde::{Serialize, Serializer};
use serde_json::Value;
use std::collections::BTreeMap;

mod projections;

pub(crate) use projections::{
    ExternalTruthBoundary, ExternalTruthBoundaryMode, ReviewFocusProjection,
};

#[derive(Serialize)]
pub(crate) struct RepoTruthSummary {
    pub(crate) projects_with_repo_truth_gaps: u64,
    pub(crate) repo_truth_gap_distribution: RepoTruthGapDistribution,
    pub(crate) mandatory_shell_check_examples: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct RepoTruthGapDistribution {
    counts: BTreeMap<String, u64>,
}

impl RepoTruthGapDistribution {
    pub(crate) fn increment_gap(&mut self, gap_key: &str) {
        *self.counts.entry(gap_key.to_string()).or_insert(0) += 1;
    }

    #[cfg(test)]
    pub(crate) fn count(&self, gap_key: &str) -> u64 {
        self.counts.get(gap_key).copied().unwrap_or(0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RepoRiskCouplingStatus {
    NoRepoRiskSignal,
    Coupled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RepoRiskCouplingSource {
    PrimaryRepoRiskFinding,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RecommendedNextAction {
    StartMonitor,
    TakeSnapshot,
    GenerateActivityThenStats,
    ReviewFailingVerification,
    RunVerificationBeforeHighRiskChanges,
    StabilizeRepositoryState,
    ReviewUnusedFiles,
    InspectHotFiles,
    Other(String),
}

impl RecommendedNextAction {
    pub(crate) fn from_action(value: &str) -> Self {
        match value {
            "start_monitor" => Self::StartMonitor,
            "take_snapshot" => Self::TakeSnapshot,
            "generate_activity_then_stats" => Self::GenerateActivityThenStats,
            "review_failing_verification" => Self::ReviewFailingVerification,
            "run_verification_before_high_risk_changes" => {
                Self::RunVerificationBeforeHighRiskChanges
            }
            "stabilize_repository_state" => Self::StabilizeRepositoryState,
            "review_unused_files" => Self::ReviewUnusedFiles,
            "inspect_hot_files" => Self::InspectHotFiles,
            value => Self::Other(value.to_string()),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::StartMonitor => "start_monitor",
            Self::TakeSnapshot => "take_snapshot",
            Self::GenerateActivityThenStats => "generate_activity_then_stats",
            Self::ReviewFailingVerification => "review_failing_verification",
            Self::RunVerificationBeforeHighRiskChanges => {
                "run_verification_before_high_risk_changes"
            }
            Self::StabilizeRepositoryState => "stabilize_repository_state",
            Self::ReviewUnusedFiles => "review_unused_files",
            Self::InspectHotFiles => "inspect_hot_files",
            Self::Other(value) => value.as_str(),
        }
    }
}

impl Serialize for RecommendedNextAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RepoRiskStrategyMode {
    VerifyBeforeModify,
    StabilizeBeforeModify,
    CollectWorkspaceContext,
    CollectEvidenceFirst,
    VerifyBeforeHighRiskChanges,
    ActivityGuidedReview,
    Other(String),
}

impl RepoRiskStrategyMode {
    pub(crate) fn from_mode(value: &str) -> Self {
        match value {
            "verify_before_modify" => Self::VerifyBeforeModify,
            "stabilize_before_modify" => Self::StabilizeBeforeModify,
            "collect_workspace_context" => Self::CollectWorkspaceContext,
            "collect_evidence_first" => Self::CollectEvidenceFirst,
            "verify_before_high_risk_changes" => Self::VerifyBeforeHighRiskChanges,
            "activity_guided_review" => Self::ActivityGuidedReview,
            value => Self::Other(value.to_string()),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::VerifyBeforeModify => "verify_before_modify",
            Self::StabilizeBeforeModify => "stabilize_before_modify",
            Self::CollectWorkspaceContext => "collect_workspace_context",
            Self::CollectEvidenceFirst => "collect_evidence_first",
            Self::VerifyBeforeHighRiskChanges => "verify_before_high_risk_changes",
            Self::ActivityGuidedReview => "activity_guided_review",
            Self::Other(value) => value.as_str(),
        }
    }
}

impl Serialize for RepoRiskStrategyMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RepoRiskPreferredTool {
    Opendog,
    Shell,
    ShellVerification,
    Other(String),
}

impl RepoRiskPreferredTool {
    pub(crate) fn from_tool(value: &str) -> Self {
        match value {
            "opendog" => Self::Opendog,
            "shell" => Self::Shell,
            "shell_verification" => Self::ShellVerification,
            value => Self::Other(value.to_string()),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Opendog => "opendog",
            Self::Shell => "shell",
            Self::ShellVerification => "shell_verification",
            Self::Other(value) => value.as_str(),
        }
    }
}

impl Serialize for RepoRiskPreferredTool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExecutionEvidencePriority {
    Verification,
    RepositoryRisk,
    ActivitySignals,
    Other(String),
}

impl ExecutionEvidencePriority {
    pub(crate) fn from_priority(value: &str) -> Self {
        match value {
            "verification" => Self::Verification,
            "repository_risk" => Self::RepositoryRisk,
            "activity_signals" => Self::ActivitySignals,
            value => Self::Other(value.to_string()),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Self::Verification => "verification",
            Self::RepositoryRisk => "repository_risk",
            Self::ActivitySignals => "activity_signals",
            Self::Other(value) => value.as_str(),
        }
    }
}

impl Serialize for ExecutionEvidencePriority {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RepoRiskFindingDetails {
    kind: String,
    directory: String,
}

impl RepoRiskFindingDetails {
    fn from_value(value: &Value) -> Option<Self> {
        let details = value.as_object()?;
        Some(Self {
            kind: details.get("kind")?.as_str()?.to_string(),
            directory: details.get("directory")?.as_str()?.to_string(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RepoRiskFinding {
    kind: String,
    severity: String,
    priority: String,
    confidence: String,
    summary: String,
    evidence: Vec<String>,
    source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<RepoRiskFindingDetails>,
}

impl RepoRiskFinding {
    pub(crate) fn from_value(value: &Value) -> Option<Self> {
        let finding = value.as_object()?;
        let evidence = finding
            .get("evidence")?
            .as_array()?
            .iter()
            .map(|item| item.as_str().map(str::to_string))
            .collect::<Option<Vec<_>>>()?;
        Some(Self {
            kind: finding.get("kind")?.as_str()?.to_string(),
            severity: finding.get("severity")?.as_str()?.to_string(),
            priority: finding.get("priority")?.as_str()?.to_string(),
            confidence: finding.get("confidence")?.as_str()?.to_string(),
            summary: finding.get("summary")?.as_str()?.to_string(),
            evidence,
            source: finding.get("source")?.as_str()?.to_string(),
            details: finding.get("details").and_then(|details| {
                if details.is_null() {
                    None
                } else {
                    RepoRiskFindingDetails::from_value(details)
                }
            }),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct RepoRiskCoupling {
    status: RepoRiskCouplingStatus,
    pub(super) source: Option<RepoRiskCouplingSource>,
    source_project_id: Option<String>,
    recommended_next_action: Option<RecommendedNextAction>,
    strategy_mode: Option<RepoRiskStrategyMode>,
    preferred_primary_tool: Option<RepoRiskPreferredTool>,
    primary_repo_risk_finding: Option<RepoRiskFinding>,
    summary: Option<String>,
}

impl RepoRiskCoupling {
    pub(crate) fn no_signal(
        recommended_next_action: Option<RecommendedNextAction>,
        strategy_mode: Option<RepoRiskStrategyMode>,
        preferred_primary_tool: Option<RepoRiskPreferredTool>,
    ) -> Self {
        Self {
            status: RepoRiskCouplingStatus::NoRepoRiskSignal,
            source: None,
            source_project_id: None,
            recommended_next_action,
            strategy_mode,
            preferred_primary_tool,
            primary_repo_risk_finding: None,
            summary: None,
        }
    }

    pub(crate) fn coupled(
        source_project_id: &str,
        recommended_next_action: Option<RecommendedNextAction>,
        strategy_mode: Option<RepoRiskStrategyMode>,
        preferred_primary_tool: Option<RepoRiskPreferredTool>,
        primary_repo_risk_finding: RepoRiskFinding,
        summary: String,
    ) -> Self {
        Self {
            status: RepoRiskCouplingStatus::Coupled,
            source: Some(RepoRiskCouplingSource::PrimaryRepoRiskFinding),
            source_project_id: Some(source_project_id.to_string()),
            recommended_next_action,
            strategy_mode,
            preferred_primary_tool,
            primary_repo_risk_finding: Some(primary_repo_risk_finding),
            summary: Some(summary),
        }
    }

    pub(crate) fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}
