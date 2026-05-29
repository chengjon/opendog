use serde::Serialize;
use serde_json::Value;

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
