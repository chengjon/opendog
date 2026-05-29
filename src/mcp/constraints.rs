use serde_json::{json, Value};
use std::path::Path;

mod external_truth;
mod model;
mod repo_truth;
mod review_focus;

pub(crate) use self::external_truth::external_truth_boundary_for_top_project;
pub(crate) use self::repo_truth::repo_truth_gap_projection;
pub(crate) use self::review_focus::review_focus_projection_for_top_project;
use super::guidance_types::{ConstraintsBoundariesLayer, ConstraintsBoundariesLayerStatus};
use super::serialization::to_value_or_error;
use model::{ProjectReadinessAssessment, ReadinessTarget};

pub(super) struct WorkspaceCounts {
    pub(super) projects_not_ready_for_cleanup: usize,
    pub(super) projects_not_ready_for_refactor: usize,
    pub(super) projects_with_hardcoded_data_candidates: usize,
    pub(super) projects_missing_snapshot: usize,
    pub(super) projects_with_stale_snapshot: usize,
    pub(super) projects_missing_activity: usize,
    pub(super) projects_with_stale_activity: usize,
    pub(super) projects_missing_verification: usize,
    pub(super) projects_with_stale_verification: usize,
    pub(super) projects_with_storage_maintenance_candidates: u64,
}

fn string_array_field(value: &Value, key: &str) -> Vec<String> {
    value[key]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn default_boundary_guardrails() -> Vec<String> {
    vec![
        "Do not treat activity-derived signals as proof of safety without shell verification."
            .to_string(),
        "Do not start broad cleanup or refactor work while verification is failing or missing."
            .to_string(),
        "Do not perform broad modifications while a repository is mid-merge, rebase, cherry-pick, or bisect."
            .to_string(),
    ]
}

pub(super) fn default_destructive_operations() -> Vec<String> {
    vec![
        "Deleting unused-file candidates without validating imports, runtime paths, and tests."
            .to_string(),
        "Running broad cleanup while verification evidence is missing, failing, or stale."
            .to_string(),
        "Starting large refactors while git reports merge/rebase/cherry-pick/bisect state."
            .to_string(),
    ]
}

#[cfg(test)]
pub(super) fn project_readiness_reasons(
    repo_risk: &Value,
    verification_layer: &Value,
    target: &str,
) -> Vec<String> {
    ProjectReadinessAssessment::from_layers(repo_risk, verification_layer)
        .reasons_for(ReadinessTarget::from_name(target))
        .to_vec()
}

pub(super) fn project_readiness_snapshot(repo_risk: &Value, verification_layer: &Value) -> Value {
    ProjectReadinessAssessment::from_layers(repo_risk, verification_layer).to_json()
}

pub(super) fn readiness_reason_summary(target: &str, safe: bool, reasons: &[String]) -> String {
    ReadinessTarget::from_name(target).reason_summary(safe, reasons)
}

pub(super) fn build_constraints_boundaries_layer(
    repo_risk: Option<&Value>,
    verification_layer: Option<&Value>,
    direct_observations: Vec<String>,
    inferences: Vec<String>,
    blind_spots: Vec<String>,
    requires_shell_verification: Vec<String>,
    workspace_counts: Option<WorkspaceCounts>,
) -> Value {
    let readiness = match (repo_risk, verification_layer) {
        (Some(repo_risk), Some(verification)) => {
            project_readiness_snapshot(repo_risk, verification)
        }
        _ => json!({
            "cleanup_blockers": [],
            "refactor_blockers": [],
        }),
    };
    let cleanup_blockers = string_array_field(&readiness, "cleanup_blockers");
    let refactor_blockers = string_array_field(&readiness, "refactor_blockers");

    let mut human_review_required_for = Vec::new();
    if !cleanup_blockers.is_empty() {
        human_review_required_for
            .push("Cleanup candidates blocked by verification or repository risk.".to_string());
    }
    if !refactor_blockers.is_empty() {
        human_review_required_for
            .push("Refactor candidates blocked by verification or repository risk.".to_string());
    }
    if repo_risk
        .and_then(|risk| risk["lockfile_anomalies"].as_array())
        .map(|items| !items.is_empty())
        .unwrap_or(false)
    {
        human_review_required_for
            .push("Dependency manifest/lockfile mismatch signals.".to_string());
    }

    to_value_or_error(
        "ConstraintsBoundariesLayer",
        ConstraintsBoundariesLayer {
            status: ConstraintsBoundariesLayerStatus::Available,
            direct_observations,
            inferences,
            blind_spots,
            guardrails: default_boundary_guardrails(),
            destructive_operations_requiring_confirmation: default_destructive_operations(),
            human_review_required_for,
            cleanup_blockers,
            refactor_blockers,
            requires_shell_verification,
            projects_not_ready_for_cleanup: workspace_counts
                .as_ref()
                .map(|c| c.projects_not_ready_for_cleanup),
            projects_not_ready_for_refactor: workspace_counts
                .as_ref()
                .map(|c| c.projects_not_ready_for_refactor),
            projects_with_hardcoded_data_candidates: workspace_counts
                .as_ref()
                .map(|c| c.projects_with_hardcoded_data_candidates),
            projects_missing_snapshot: workspace_counts
                .as_ref()
                .map(|c| c.projects_missing_snapshot),
            projects_with_stale_snapshot: workspace_counts
                .as_ref()
                .map(|c| c.projects_with_stale_snapshot),
            projects_missing_activity: workspace_counts
                .as_ref()
                .map(|c| c.projects_missing_activity),
            projects_with_stale_activity: workspace_counts
                .as_ref()
                .map(|c| c.projects_with_stale_activity),
            projects_missing_verification: workspace_counts
                .as_ref()
                .map(|c| c.projects_missing_verification),
            projects_with_stale_verification: workspace_counts
                .as_ref()
                .map(|c| c.projects_with_stale_verification),
            projects_with_storage_maintenance_candidates: workspace_counts
                .as_ref()
                .map(|c| c.projects_with_storage_maintenance_candidates),
        },
    )
}

pub(super) fn common_boundary_hints(root: &Path) -> Value {
    let protected_paths = [".git", ".opendog"]
        .iter()
        .filter(|path| root.join(path).exists())
        .map(|path| (*path).to_string())
        .collect::<Vec<_>>();
    let generated_artifact_directories = [
        "target",
        "node_modules",
        "dist",
        "build",
        ".next",
        "coverage",
        ".turbo",
    ]
    .iter()
    .filter(|path| root.join(path).exists())
    .map(|path| (*path).to_string())
    .collect::<Vec<_>>();

    json!({
        "protected_paths": protected_paths,
        "generated_artifact_directories": generated_artifact_directories,
    })
}

#[cfg(test)]
mod tests;
