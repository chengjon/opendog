mod attention;
mod decision;
mod portfolio;
mod recommendation;
mod repo_risk;
mod summaries;
mod workspace_layers;

pub(crate) use attention::{AttentionPriorityBasis, AttentionSummary};
pub(crate) use decision::{DecisionBrief, DecisionSignals};
pub(crate) use portfolio::{WorkspacePortfolioLayer, WorkspacePortfolioLayerStatus};
pub(crate) use recommendation::{ProjectOverview, Recommendation};
#[cfg(test)]
use repo_risk::RepoRiskCouplingSource;
pub(crate) use repo_risk::{
    ExecutionEvidencePriority, ExternalTruthBoundary, ExternalTruthBoundaryMode,
    RecommendedNextAction, RepoRiskCoupling, RepoRiskFinding, RepoRiskPreferredTool,
    RepoRiskStrategyMode, RepoTruthGapDistribution, RepoTruthSummary, ReviewFocusProjection,
};
pub(crate) use summaries::{
    DataRiskFocusDistribution, DataRiskFocusSummary, ObservationSummary, StabilizationSummary,
    VerificationSummary,
};
pub(crate) use workspace_layers::{
    ConstraintsBoundariesLayer, ConstraintsBoundariesLayerStatus, ExecutionStrategyLayer,
    ExecutionStrategyLayerStatus, WorkspaceObservationAnalysisState, WorkspaceObservationLayer,
    WorkspaceObservationLayerStatus,
};

#[cfg(test)]
mod tests;
