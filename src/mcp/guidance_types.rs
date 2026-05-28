use serde::{Serialize, Serializer};
use std::collections::BTreeMap;

use serde_json::Value;

#[derive(Serialize)]
pub(crate) struct Recommendation {
    pub(crate) project_id: String,
    pub(crate) recommended_next_action: String,
    pub(crate) recommended_flow: Vec<String>,
    pub(crate) reason: String,
    pub(crate) confidence: String,
    pub(crate) strategy_mode: String,
    pub(crate) strategy_profile: Value,
    pub(crate) verification_gate_levels: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cleanup_blockers: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) refactor_blockers: Option<Value>,
    pub(crate) repo_truth_gaps: Value,
    pub(crate) mandatory_shell_checks: Value,
    pub(crate) suggested_commands: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct ProjectOverview {
    pub(crate) project_id: String,
    pub(crate) status: String,
    pub(crate) snapshot_available: bool,
    pub(crate) activity_available: bool,
    pub(crate) unused_files: i64,
    pub(crate) observation: Value,
    pub(crate) repo_status_risk: Value,
    pub(crate) verification_evidence: Value,
    pub(crate) mock_data_summary: Value,
    pub(crate) storage_maintenance: Value,
    pub(crate) project_toolchain: Value,
    pub(crate) verification_safe_for_cleanup: Value,
    pub(crate) verification_safe_for_refactor: Value,
    pub(crate) verification_gate_levels: Value,
    pub(crate) safe_for_cleanup: Value,
    pub(crate) safe_for_cleanup_reason: Value,
    pub(crate) cleanup_blockers: Value,
    pub(crate) safe_for_refactor: Value,
    pub(crate) safe_for_refactor_reason: Value,
    pub(crate) refactor_blockers: Value,
    pub(crate) recommended_next_action: Value,
    pub(crate) recommended_flow: Value,
    pub(crate) recommended_reason: Value,
    pub(crate) strategy_confidence: Value,
}

#[derive(Serialize)]
pub(crate) struct AttentionSummary {
    pub(crate) attention_score: i64,
    pub(crate) attention_band: String,
    pub(crate) attention_reasons: Vec<String>,
    pub(crate) evidence_quality: String,
    pub(crate) priority_basis: AttentionPriorityBasis,
}

#[derive(Serialize)]
pub(crate) struct AttentionPriorityBasis {
    pub(crate) recommended_next_action: String,
    pub(crate) recommended_action_base: i64,
    pub(crate) repo_risk_level: String,
    pub(crate) repo_in_operation: bool,
    pub(crate) repo_is_dirty: bool,
    pub(crate) verification_status: String,
    pub(crate) has_failing_verification: bool,
    pub(crate) coverage_state: String,
    pub(crate) snapshot_freshness: String,
    pub(crate) activity_freshness: String,
    pub(crate) verification_freshness: String,
    pub(crate) hardcoded_candidate_count: u64,
    pub(crate) mock_candidate_count: u64,
    pub(crate) safe_for_cleanup: bool,
    pub(crate) safe_for_refactor: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkspacePortfolioLayerStatus {
    Available,
}

impl WorkspacePortfolioLayerStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
        }
    }
}

#[derive(Serialize)]
pub(crate) struct WorkspacePortfolioLayer {
    pub(crate) status: WorkspacePortfolioLayerStatus,
    pub(crate) project_count: usize,
    pub(crate) monitoring_count: usize,
    pub(crate) monitored_projects: Vec<Value>,
    pub(crate) priority_candidates: Vec<Value>,
    pub(crate) project_overviews: Vec<Value>,
    pub(crate) priority_model: String,
    pub(crate) dirty_projects: usize,
    pub(crate) high_risk_projects: usize,
    pub(crate) projects_with_failing_verification: usize,
    pub(crate) projects_safe_for_cleanup: usize,
    pub(crate) projects_safe_for_refactor: usize,
    pub(crate) projects_with_hardcoded_candidates: usize,
    pub(crate) projects_with_hardcoded_data_candidates: usize,
    pub(crate) total_mock_candidates: u64,
    pub(crate) total_hardcoded_candidates: u64,
    pub(crate) projects_in_operation: Vec<Value>,
    pub(crate) attention_queue: Vec<Value>,
    pub(crate) attention_batches: Value,
}

#[derive(Serialize)]
pub(crate) struct DecisionBrief {
    pub(crate) summary: String,
    pub(crate) recommended_next_action: String,
    pub(crate) reason: Value,
    pub(crate) repo_truth_gaps: Value,
    pub(crate) mandatory_shell_checks: Value,
    pub(crate) external_truth_boundary: Value,
    pub(crate) review_focus: Value,
    pub(crate) execution_sequence: Value,
    pub(crate) data_risk_focus: Value,
    pub(crate) target_project_id: Option<String>,
    pub(crate) strategy_mode: Value,
    pub(crate) preferred_primary_tool: Value,
    pub(crate) preferred_secondary_tool: Value,
    pub(crate) recommended_flow: Value,
    pub(crate) safe_for_cleanup: Option<bool>,
    pub(crate) safe_for_refactor: Option<bool>,
    pub(crate) verification_status: String,
    pub(crate) requires_verification: bool,
    pub(crate) action_profile: Value,
    pub(crate) risk_profile: Value,
    pub(crate) signals: DecisionSignals,
}

#[derive(Serialize)]
pub(crate) struct DecisionSignals {
    pub(crate) repo_risk_level: String,
    pub(crate) repo_is_dirty: bool,
    pub(crate) hardcoded_candidate_count: u64,
    pub(crate) mock_candidate_count: u64,
    pub(crate) mixed_review_file_count: u64,
    pub(crate) storage_maintenance_candidate: bool,
    pub(crate) storage_vacuum_candidate: bool,
    pub(crate) storage_reclaimable_bytes: i64,
    pub(crate) storage_db_size_bytes: i64,
    pub(crate) attention_score: i64,
    pub(crate) attention_band: String,
    pub(crate) attention_reasons: Vec<Value>,
    pub(crate) monitoring_count: u64,
}

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
enum RepoRiskCouplingSource {
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
    source: Option<RepoRiskCouplingSource>,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReviewFocusProjectionStatus {
    NoPriorityProject,
    Available,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct ReviewFocusProjection {
    status: ReviewFocusProjectionStatus,
    source: Option<String>,
    source_project_id: Option<String>,
    review_focus: Value,
}

impl ReviewFocusProjection {
    pub(crate) fn no_priority_project() -> Self {
        Self {
            status: ReviewFocusProjectionStatus::NoPriorityProject,
            source: None,
            source_project_id: None,
            review_focus: Value::Null,
        }
    }

    pub(crate) fn available(source_project_id: Option<String>, review_focus: Value) -> Self {
        Self {
            status: ReviewFocusProjectionStatus::Available,
            source: Some("top_priority_project".to_string()),
            source_project_id,
            review_focus,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExternalTruthBoundaryStatus {
    NoPriorityProject,
    Available,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExternalTruthBoundaryMode {
    MustSwitchToExternalTruth,
    OpendogGuidanceCanContinue,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct ExternalTruthBoundary {
    status: ExternalTruthBoundaryStatus,
    source: Option<String>,
    source_project_id: Option<String>,
    mode: Option<ExternalTruthBoundaryMode>,
    repo_state_required: bool,
    verification_required: bool,
    triggers: Vec<String>,
    minimum_external_checks: Vec<String>,
    summary: Option<String>,
}

impl ExternalTruthBoundary {
    pub(crate) fn no_priority_project() -> Self {
        Self {
            status: ExternalTruthBoundaryStatus::NoPriorityProject,
            source: None,
            source_project_id: None,
            mode: None,
            repo_state_required: false,
            verification_required: false,
            triggers: Vec::new(),
            minimum_external_checks: Vec::new(),
            summary: None,
        }
    }

    pub(crate) fn available(
        source_project_id: Option<String>,
        mode: ExternalTruthBoundaryMode,
        repo_state_required: bool,
        verification_required: bool,
        triggers: Vec<String>,
        minimum_external_checks: Vec<String>,
        summary: &str,
    ) -> Self {
        Self {
            status: ExternalTruthBoundaryStatus::Available,
            source: Some("top_priority_project".to_string()),
            source_project_id,
            mode: Some(mode),
            repo_state_required,
            verification_required,
            triggers,
            minimum_external_checks,
            summary: Some(summary.to_string()),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct StabilizationSummary {
    pub(crate) projects_requiring_repo_stabilization: u64,
    pub(crate) repo_stabilization_priority_projects: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct VerificationSummary {
    pub(crate) projects_requiring_verification_run: u64,
    pub(crate) projects_requiring_failing_verification_repair: u64,
}

#[derive(Serialize)]
pub(crate) struct ObservationSummary {
    pub(crate) projects_requiring_monitor_start: u64,
    pub(crate) projects_requiring_snapshot_refresh: u64,
    pub(crate) projects_requiring_activity_generation: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub(crate) struct DataRiskFocusDistribution {
    pub(crate) hardcoded: u64,
    pub(crate) mixed: u64,
    pub(crate) mock: u64,
    pub(crate) none: u64,
}

impl DataRiskFocusDistribution {
    pub(crate) fn increment_focus(&mut self, focus: &str) {
        match focus {
            "hardcoded" => self.hardcoded += 1,
            "mixed" => self.mixed += 1,
            "mock" => self.mock += 1,
            _ => self.none += 1,
        }
    }

    pub(crate) fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

#[derive(Serialize)]
pub(crate) struct DataRiskFocusSummary {
    pub(crate) data_risk_focus_distribution: DataRiskFocusDistribution,
    pub(crate) projects_requiring_hardcoded_review: u64,
    pub(crate) projects_requiring_mock_review: u64,
    pub(crate) projects_requiring_mixed_file_review: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkspaceObservationLayerStatus {
    Available,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkspaceObservationAnalysisState {
    Empty,
    InsufficientActivity,
    Ready,
    Stale,
}

#[derive(Serialize)]
pub(crate) struct WorkspaceObservationLayer {
    pub(crate) status: WorkspaceObservationLayerStatus,
    pub(crate) project_count: usize,
    pub(crate) monitoring_count: usize,
    pub(crate) analysis_state: WorkspaceObservationAnalysisState,
    pub(crate) projects_missing_snapshot: usize,
    pub(crate) projects_with_stale_snapshot: usize,
    pub(crate) projects_missing_activity: usize,
    pub(crate) projects_with_stale_activity: usize,
    pub(crate) projects_missing_verification: usize,
    pub(crate) projects_with_stale_verification: usize,
    pub(crate) projects_with_storage_maintenance_candidates: u64,
    pub(crate) projects_with_vacuum_candidates: u64,
    pub(crate) total_storage_reclaimable_bytes: Value,
    pub(crate) data_risk_focus_distribution: Value,
    pub(crate) projects_requiring_hardcoded_review: Value,
    pub(crate) projects_requiring_mock_review: Value,
    pub(crate) projects_requiring_mixed_file_review: Value,
    pub(crate) notes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExecutionStrategyLayerStatus {
    Available,
}

#[derive(Serialize)]
pub(crate) struct ExecutionStrategyLayer {
    pub(crate) status: ExecutionStrategyLayerStatus,
    pub(crate) recommended_flow: Vec<String>,
    pub(crate) project_recommendations: Vec<Value>,
    pub(crate) global_strategy_mode: RepoRiskStrategyMode,
    pub(crate) preferred_primary_tool: RepoRiskPreferredTool,
    pub(crate) preferred_secondary_tool: RepoRiskPreferredTool,
    pub(crate) evidence_priority: Vec<ExecutionEvidencePriority>,
    pub(crate) risk_strategy_coupling: RepoRiskCoupling,
    pub(crate) external_truth_boundary: ExternalTruthBoundary,
    pub(crate) review_focus_projection: ReviewFocusProjection,
    pub(crate) when_to_use_opendog: Vec<&'static str>,
    pub(crate) when_to_use_shell: Vec<&'static str>,
    pub(crate) guardrails: Vec<&'static str>,
    pub(crate) projects_not_ready_for_cleanup: usize,
    pub(crate) projects_not_ready_for_refactor: usize,
    pub(crate) projects_with_hardcoded_data_candidates: usize,
    pub(crate) projects_missing_snapshot: usize,
    pub(crate) projects_with_stale_snapshot: usize,
    pub(crate) projects_missing_activity: usize,
    pub(crate) projects_with_stale_activity: usize,
    pub(crate) projects_missing_verification: usize,
    pub(crate) projects_with_stale_verification: usize,
    pub(crate) projects_with_storage_maintenance_candidates: u64,
    pub(crate) projects_with_vacuum_candidates: u64,
    pub(crate) review_opendog_retention_before_large_cleanup: bool,
    pub(crate) recommend_manual_review_for_hardcoded_data: bool,
    pub(crate) data_risk_focus_distribution: DataRiskFocusDistribution,
    pub(crate) projects_requiring_hardcoded_review: u64,
    pub(crate) projects_requiring_mock_review: u64,
    pub(crate) projects_requiring_mixed_file_review: u64,
    pub(crate) projects_requiring_monitor_start: u64,
    pub(crate) projects_requiring_snapshot_refresh: u64,
    pub(crate) projects_requiring_activity_generation: u64,
    pub(crate) projects_with_repo_truth_gaps: u64,
    pub(crate) repo_truth_gap_distribution: RepoTruthGapDistribution,
    pub(crate) mandatory_shell_check_examples: Vec<String>,
    pub(crate) projects_requiring_verification_run: u64,
    pub(crate) projects_requiring_failing_verification_repair: u64,
    pub(crate) projects_requiring_repo_stabilization: u64,
    pub(crate) repo_stabilization_priority_projects: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConstraintsBoundariesLayerStatus {
    Available,
}

#[derive(Serialize)]
pub(crate) struct ConstraintsBoundariesLayer {
    pub(crate) status: ConstraintsBoundariesLayerStatus,
    pub(crate) direct_observations: Vec<String>,
    pub(crate) inferences: Vec<String>,
    pub(crate) blind_spots: Vec<String>,
    pub(crate) guardrails: Vec<String>,
    pub(crate) destructive_operations_requiring_confirmation: Vec<String>,
    pub(crate) human_review_required_for: Vec<String>,
    pub(crate) cleanup_blockers: Vec<String>,
    pub(crate) refactor_blockers: Vec<String>,
    pub(crate) requires_shell_verification: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_not_ready_for_cleanup: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_not_ready_for_refactor: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_hardcoded_data_candidates: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_missing_snapshot: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_stale_snapshot: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_missing_activity: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_stale_activity: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_missing_verification: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_stale_verification: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_storage_maintenance_candidates: Option<u64>,
}

#[cfg(test)]
mod tests;
