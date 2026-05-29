use serde_json::{json, Value};

use crate::contracts::{
    MCP_RECORD_VERIFICATION_V1, MCP_RUN_VERIFICATION_V1, MCP_VERIFICATION_STATUS_V1,
};
use crate::core::verification::ExecutedVerificationResult;
use crate::storage::queries::VerificationRun;

use super::{now_unix_secs, versioned_project_payload};

mod model;
use model::{VerificationEvidenceWorkspaceSummary, VerificationStatusSummary};

pub(crate) fn verification_status_layer(runs: &[VerificationRun]) -> Value {
    VerificationStatusSummary::from_runs(runs, now_unix_secs()).to_json()
}

pub(super) fn verification_status_payload(id: &str, runs: &[VerificationRun]) -> Value {
    versioned_project_payload(
        MCP_VERIFICATION_STATUS_V1,
        id,
        [("verification", verification_status_layer(runs))],
    )
}

pub(super) fn record_verification_payload(id: &str, run: &VerificationRun) -> Value {
    versioned_project_payload(MCP_RECORD_VERIFICATION_V1, id, [("recorded", json!(run))])
}

pub(super) fn run_verification_payload(id: &str, result: &ExecutedVerificationResult) -> Value {
    versioned_project_payload(MCP_RUN_VERIFICATION_V1, id, [("executed", json!(result))])
}

pub(super) fn verification_has_failures(runs: &[VerificationRun]) -> bool {
    runs.iter().any(|r| r.status != "passed")
}

pub(super) fn verification_is_missing(runs: &[VerificationRun]) -> bool {
    runs.is_empty()
}

#[cfg(test)]
fn project_gate_level(project: &Value, target: &str) -> String {
    model::project_gate_level(project, target)
}

pub(super) fn workspace_verification_evidence_layer(
    project_overviews: &[Value],
    project_count: usize,
    monitoring_count: usize,
) -> Value {
    let summary = VerificationEvidenceWorkspaceSummary::from_project_overviews(
        project_overviews,
        project_count,
        monitoring_count,
    );

    json!({
        "status": "available",
        "projects_with_recorded_verification": summary.projects_with_recorded_verification,
        "projects_missing_verification": summary.projects_missing_verification,
        "projects_with_failing_verification": summary.projects_with_failing_verification,
        "projects_with_stale_verification": summary.projects_with_stale_verification,
        "projects_safe_for_cleanup": summary.projects_safe_for_cleanup,
        "projects_safe_for_refactor": summary.projects_safe_for_refactor,
        "cleanup_gate_distribution": summary.cleanup_gate_distribution.to_json(),
        "refactor_gate_distribution": summary.refactor_gate_distribution.to_json(),
        "direct_observations": summary.direct_observations(),
        "inferences": [
            "Project recommendations can rely more strongly on projects whose verification evidence is both recorded and fresh.",
            "Missing or stale verification evidence should be refreshed before broad cleanup or refactor work.",
            "Failing verification evidence should outrank cleanup-oriented follow-up work."
        ],
        "verified_conclusions": summary.verified_conclusions_json(),
        "unverified_conclusions": summary.unverified_conclusions_json(),
        "blocking_projects": summary.blocking_projects_json(),
        "confidence": summary.confidence(),
    })
}

#[cfg(test)]
fn gate_kinds(target: &str) -> (&'static [&'static str], &'static [&'static str]) {
    model::gate_kinds(target)
}

#[cfg(test)]
fn kind_state_sets<'a>(
    runs: &'a [VerificationRun],
    kinds: &[&'a str],
    now_secs: i64,
) -> (Vec<&'a str>, Vec<&'a str>) {
    model::kind_state_sets(runs, kinds, now_secs)
}

#[cfg(test)]
fn failing_kinds(runs: &[VerificationRun]) -> Vec<&str> {
    model::failing_kinds(runs)
}

#[cfg(test)]
fn blocker_reasons(
    target: &str,
    required_missing: &[&str],
    required_stale: &[&str],
    failing_kinds: &[&str],
) -> Vec<String> {
    model::blocker_reasons(target, required_missing, required_stale, failing_kinds)
}

#[cfg(test)]
fn gate_blockers(runs: &[VerificationRun], target: &str, now_secs: i64) -> Vec<String> {
    model::gate_blockers(runs, target, now_secs)
}

#[cfg(test)]
fn gate_reasons(
    target: &str,
    required_missing: &[&str],
    required_stale: &[&str],
    advisory_missing: &[&str],
    advisory_stale: &[&str],
    failing: &[&str],
) -> Vec<String> {
    model::gate_reasons(
        target,
        required_missing,
        required_stale,
        advisory_missing,
        advisory_stale,
        failing,
    )
}

#[cfg(test)]
fn gate_next_steps(
    target: &str,
    required_missing: &[&str],
    required_stale: &[&str],
    advisory_missing: &[&str],
    advisory_stale: &[&str],
    failing: &[&str],
) -> Vec<String> {
    model::gate_next_steps(
        target,
        required_missing,
        required_stale,
        advisory_missing,
        advisory_stale,
        failing,
    )
}

#[cfg(test)]
fn gate_assessment(runs: &[VerificationRun], target: &str, now_secs: i64) -> Value {
    model::gate_assessment(runs, target, now_secs)
}

#[cfg(test)]
mod tests;
