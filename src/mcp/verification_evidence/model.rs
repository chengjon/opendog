use crate::core::verification::{
    command_contains_pipeline_operators, detect_suspicious_pass_signals,
};
use crate::storage::queries::VerificationRun;
use serde_json::{json, Value};

use super::super::constraints::readiness_reason_summary;
use super::super::observation::{
    freshness_detail, freshness_policy, latest_verification_timestamp, verification_is_stale,
};

const EXPECTED_KINDS: [&str; 3] = ["test", "lint", "build"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VerificationGateTarget {
    Cleanup,
    Refactor,
}

impl VerificationGateTarget {
    #[cfg(test)]
    fn from_name(target: &str) -> Self {
        match target {
            "refactor" => Self::Refactor,
            _ => Self::Cleanup,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Cleanup => "cleanup",
            Self::Refactor => "refactor",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct VerificationGateAssessment {
    pub(super) allowed: bool,
    pub(super) level: String,
    pub(super) required_kinds: Vec<String>,
    pub(super) advisory_kinds: Vec<String>,
    pub(super) missing_kinds: Vec<String>,
    pub(super) failing_kinds: Vec<String>,
    pub(super) stale_kinds: Vec<String>,
    pub(super) pipeline_caution_kinds: Vec<String>,
    pub(super) suspicious_summary_kinds: Vec<String>,
    pub(super) reasons: Vec<String>,
    pub(super) next_steps: Vec<String>,
}

impl VerificationGateAssessment {
    pub(super) fn from_runs(
        runs: &[VerificationRun],
        target: VerificationGateTarget,
        now_secs: i64,
    ) -> Self {
        let target_name = target.as_str();
        let (required_kinds, advisory_kinds) = gate_kinds(target_name);
        let (required_missing, required_stale) = kind_state_sets(runs, required_kinds, now_secs);
        let (advisory_missing, advisory_stale) = kind_state_sets(runs, advisory_kinds, now_secs);
        let failing = failing_kinds(runs);
        let pipeline_caution = pipeline_caution_kinds(runs);
        let suspicious_summary = suspicious_summary_kinds(runs);
        let blockers = blocker_reasons(target_name, &required_missing, &required_stale, &failing);
        let mut reasons = gate_reasons(
            target_name,
            &required_missing,
            &required_stale,
            &advisory_missing,
            &advisory_stale,
            &failing,
        );
        if !pipeline_caution.is_empty() {
            reasons.push(format!(
                "Passed {} verification used pipeline commands whose exit codes may be masked. Consider rerunning without pipes.",
                pipeline_caution.join(", ")
            ));
        }
        if !suspicious_summary.is_empty() {
            reasons.push(format!(
                "Passed {} verification includes suspicious pass signals in recorded summaries. Recheck the original command output.",
                suspicious_summary.join(", ")
            ));
        }
        let level = if !blockers.is_empty() {
            "blocked"
        } else if !advisory_missing.is_empty()
            || !advisory_stale.is_empty()
            || !pipeline_caution.is_empty()
            || !suspicious_summary.is_empty()
        {
            "caution"
        } else {
            "allow"
        };
        let missing_kinds = required_missing
            .iter()
            .chain(advisory_missing.iter())
            .copied()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let stale_kinds = required_stale
            .iter()
            .chain(advisory_stale.iter())
            .copied()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let mut next_steps = gate_next_steps(
            target_name,
            &required_missing,
            &required_stale,
            &advisory_missing,
            &advisory_stale,
            &failing,
        );
        if !pipeline_caution.is_empty() {
            next_steps.push(
                "Rerun pipeline commands without pipes for more reliable exit-code capture."
                    .to_string(),
            );
        }
        if !suspicious_summary.is_empty() {
            next_steps.push(
                "Rerun passed commands whose recorded summaries still contain error or failure text."
                    .to_string(),
            );
        }

        Self {
            allowed: blockers.is_empty(),
            level: level.to_string(),
            required_kinds: required_kinds
                .iter()
                .map(|kind| (*kind).to_string())
                .collect(),
            advisory_kinds: advisory_kinds
                .iter()
                .map(|kind| (*kind).to_string())
                .collect(),
            missing_kinds,
            failing_kinds: failing.iter().map(|kind| (*kind).to_string()).collect(),
            stale_kinds,
            pipeline_caution_kinds: pipeline_caution
                .iter()
                .map(|kind| (*kind).to_string())
                .collect(),
            suspicious_summary_kinds: suspicious_summary
                .iter()
                .map(|kind| (*kind).to_string())
                .collect(),
            reasons,
            next_steps,
        }
    }

    pub(super) fn to_json(&self) -> Value {
        json!({
            "allowed": self.allowed,
            "level": self.level,
            "required_kinds": self.required_kinds,
            "advisory_kinds": self.advisory_kinds,
            "missing_kinds": self.missing_kinds,
            "failing_kinds": self.failing_kinds,
            "stale_kinds": self.stale_kinds,
            "pipeline_caution_kinds": self.pipeline_caution_kinds,
            "suspicious_summary_kinds": self.suspicious_summary_kinds,
            "freshness_policy": freshness_policy(),
            "reasons": self.reasons,
            "next_steps": self.next_steps,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct VerificationLatestRunSummary {
    kind: String,
    status: String,
    command: String,
    exit_code: Option<i64>,
    summary: Option<String>,
    source: String,
    finished_at: String,
    exit_code_masked_possible: bool,
    suspicious_pass_signals: Vec<String>,
}

impl VerificationLatestRunSummary {
    fn from_run(run: &VerificationRun) -> Self {
        let pipeline = command_contains_pipeline_operators(&run.command);
        let suspicious_pass_signals = suspicious_summary_signals(run);
        Self {
            kind: run.kind.clone(),
            status: run.status.clone(),
            command: run.command.clone(),
            exit_code: run.exit_code,
            summary: run.summary.clone(),
            source: run.source.clone(),
            finished_at: run.finished_at.clone(),
            exit_code_masked_possible: pipeline && run.status == "passed",
            suspicious_pass_signals,
        }
    }

    fn trust_level(&self) -> &'static str {
        if self.exit_code_masked_possible || !self.suspicious_pass_signals.is_empty() {
            "caution"
        } else {
            "trusted"
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "kind": self.kind,
            "status": self.status,
            "command": self.command,
            "exit_code": self.exit_code,
            "summary": self.summary,
            "source": self.source,
            "finished_at": self.finished_at,
            "exit_code_masked_possible": self.exit_code_masked_possible,
            "suspicious_pass_signals": self.suspicious_pass_signals,
            "trust_level": self.trust_level(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct VerificationFailingRunSummary {
    kind: String,
    status: String,
    command: String,
}

impl VerificationFailingRunSummary {
    fn from_run(run: &VerificationRun) -> Self {
        Self {
            kind: run.kind.clone(),
            status: run.status.clone(),
            command: run.command.clone(),
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "kind": self.kind,
            "status": self.status,
            "command": self.command,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct VerificationStatusSummary {
    pub(super) status: &'static str,
    summary: &'static str,
    latest_runs: Vec<VerificationLatestRunSummary>,
    latest_finished_at: Option<String>,
    freshness: Value,
    pub(super) missing_kinds: Vec<String>,
    pub(super) all_expected_kinds_recorded: bool,
    pub(super) safe_for_cleanup: bool,
    pub(super) safe_for_refactor: bool,
    cleanup_blockers: Vec<String>,
    refactor_blockers: Vec<String>,
    cleanup_gate: VerificationGateAssessment,
    refactor_gate: VerificationGateAssessment,
    failing_runs: Vec<VerificationFailingRunSummary>,
}

impl VerificationStatusSummary {
    pub(super) fn from_runs(runs: &[VerificationRun], now_secs: i64) -> Self {
        let recorded_kinds = runs.iter().map(|run| run.kind.as_str()).collect::<Vec<_>>();
        let missing_kinds = EXPECTED_KINDS
            .iter()
            .copied()
            .filter(|kind| !recorded_kinds.iter().any(|recorded| recorded == kind))
            .map(str::to_string)
            .collect::<Vec<_>>();
        let cleanup_gate =
            VerificationGateAssessment::from_runs(runs, VerificationGateTarget::Cleanup, now_secs);
        let refactor_gate =
            VerificationGateAssessment::from_runs(runs, VerificationGateTarget::Refactor, now_secs);
        let cleanup_blockers = gate_blockers(runs, "cleanup", now_secs);
        let refactor_blockers = gate_blockers(runs, "refactor", now_secs);
        let latest_finished_at = latest_verification_timestamp(runs);
        let freshness = freshness_detail(
            "verification",
            latest_finished_at.as_deref(),
            !runs.is_empty(),
            now_secs,
        );
        let failing_runs = runs
            .iter()
            .filter(|run| run.status != "passed")
            .map(VerificationFailingRunSummary::from_run)
            .collect::<Vec<_>>();
        let latest_runs = runs
            .iter()
            .map(VerificationLatestRunSummary::from_run)
            .collect::<Vec<_>>();
        let pipeline_caution_runs = pipeline_caution_kinds(runs);
        let suspicious_summary_runs = suspicious_summary_kinds(runs);
        let status = if runs.is_empty() {
            "not_recorded"
        } else {
            "available"
        };
        let summary = if runs.is_empty() {
            "No test/lint/build results have been recorded yet."
        } else if failing_runs.is_empty()
            && pipeline_caution_runs.is_empty()
            && suspicious_summary_runs.is_empty()
        {
            "Recorded verification results exist and the latest known runs are passing."
        } else if !failing_runs.is_empty() {
            "Recorded verification results include failing or uncertain runs."
        } else if !suspicious_summary_runs.is_empty() {
            "Recorded verification results include passed runs with suspicious error signals in their summaries."
        } else {
            "Recorded verification results exist but some passed runs used pipeline commands whose exit codes may be masked."
        };

        Self {
            status,
            summary,
            latest_runs,
            latest_finished_at,
            freshness,
            all_expected_kinds_recorded: missing_kinds.is_empty(),
            missing_kinds,
            safe_for_cleanup: cleanup_gate.allowed,
            safe_for_refactor: refactor_gate.allowed,
            cleanup_blockers,
            refactor_blockers,
            cleanup_gate,
            refactor_gate,
            failing_runs,
        }
    }

    pub(super) fn to_json(&self) -> Value {
        json!({
            "status": self.status,
            "summary": self.summary,
            "latest_runs": self
                .latest_runs
                .iter()
                .map(VerificationLatestRunSummary::to_json)
                .collect::<Vec<_>>(),
            "latest_finished_at": self.latest_finished_at,
            "freshness": self.freshness,
            "failing_runs": self
                .failing_runs
                .iter()
                .map(VerificationFailingRunSummary::to_json)
                .collect::<Vec<_>>(),
            "missing_kinds": self.missing_kinds,
            "all_expected_kinds_recorded": self.all_expected_kinds_recorded,
            "safe_for_cleanup": self.safe_for_cleanup,
            "safe_for_refactor": self.safe_for_refactor,
            "cleanup_blockers": self.cleanup_blockers,
            "refactor_blockers": self.refactor_blockers,
            "safe_for_cleanup_reason": readiness_reason_summary(
                "cleanup",
                self.safe_for_cleanup,
                &self.cleanup_blockers,
            ),
            "safe_for_refactor_reason": readiness_reason_summary(
                "refactor",
                self.safe_for_refactor,
                &self.refactor_blockers,
            ),
            "gate_assessment": {
                "cleanup": self.cleanup_gate.to_json(),
                "refactor": self.refactor_gate.to_json(),
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GateDistribution {
    pub(super) allow: usize,
    pub(super) caution: usize,
    pub(super) blocked: usize,
}

impl GateDistribution {
    fn from_levels(levels: impl Iterator<Item = String>) -> Self {
        let mut distribution = Self {
            allow: 0,
            caution: 0,
            blocked: 0,
        };
        for level in levels {
            match level.as_str() {
                "allow" => distribution.allow += 1,
                "caution" => distribution.caution += 1,
                "blocked" => distribution.blocked += 1,
                _ => {}
            }
        }
        distribution
    }

    pub(super) fn to_json(&self) -> Value {
        json!({
            "allow": self.allow,
            "caution": self.caution,
            "blocked": self.blocked,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct VerificationEvidenceProjectSummary {
    project_id: Option<String>,
    verification_status: Option<String>,
    verification_freshness: Value,
    freshness_status: Option<String>,
    failing_run_count: usize,
    safe_for_cleanup: bool,
    safe_for_refactor: bool,
    cleanup_gate_level: String,
    refactor_gate_level: String,
    cleanup_reason: String,
    refactor_reason: String,
}

impl VerificationEvidenceProjectSummary {
    fn from_project_overview(project: &Value) -> Self {
        let verification_evidence = &project["verification_evidence"];
        let verification_freshness = project["observation"]["freshness"]["verification"].clone();
        Self {
            project_id: string_field(project, "project_id"),
            verification_status: string_field(verification_evidence, "status"),
            freshness_status: string_field(&verification_freshness, "status"),
            failing_run_count: verification_evidence["failing_runs"]
                .as_array()
                .map(|runs| runs.len())
                .unwrap_or(0),
            safe_for_cleanup: project["safe_for_cleanup"].as_bool().unwrap_or(false),
            safe_for_refactor: project["safe_for_refactor"].as_bool().unwrap_or(false),
            cleanup_gate_level: project_gate_level(project, "cleanup"),
            refactor_gate_level: project_gate_level(project, "refactor"),
            cleanup_reason: project["safe_for_cleanup_reason"]
                .as_str()
                .unwrap_or("Cleanup readiness is blocked.")
                .to_string(),
            refactor_reason: project["safe_for_refactor_reason"]
                .as_str()
                .unwrap_or("Refactor readiness is blocked.")
                .to_string(),
            verification_freshness,
        }
    }

    fn has_recorded_verification(&self) -> bool {
        self.verification_status.as_deref() == Some("available")
    }

    fn is_missing_verification(&self) -> bool {
        self.verification_status.as_deref() == Some("not_recorded")
    }

    fn has_failing_verification(&self) -> bool {
        self.failing_run_count > 0
    }

    fn has_stale_verification(&self) -> bool {
        matches!(self.freshness_status.as_deref(), Some("stale" | "unknown"))
    }

    fn is_blocking(&self) -> bool {
        !self.safe_for_cleanup || !self.safe_for_refactor
    }

    fn cleanup_blocked_by_verification(&self) -> bool {
        self.has_failing_verification()
            || self.is_missing_verification()
            || self.has_stale_verification()
    }

    fn primary_reason(&self) -> &str {
        if self.cleanup_blocked_by_verification() {
            &self.cleanup_reason
        } else {
            &self.refactor_reason
        }
    }

    fn blocking_project_json(&self) -> Value {
        json!({
            "project_id": self.project_id.as_deref(),
            "verification_status": self.verification_status.as_deref(),
            "verification_freshness": self.verification_freshness,
            "failing_run_count": self.failing_run_count,
            "safe_for_cleanup": self.safe_for_cleanup,
            "safe_for_refactor": self.safe_for_refactor,
            "cleanup_gate_level": self.cleanup_gate_level,
            "refactor_gate_level": self.refactor_gate_level,
            "cleanup_reason": self.cleanup_reason,
            "refactor_reason": self.refactor_reason,
            "primary_reason": self.primary_reason(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct VerificationEvidenceWorkspaceSummary {
    pub(super) project_count: usize,
    pub(super) monitoring_count: usize,
    pub(super) projects_with_recorded_verification: usize,
    pub(super) projects_missing_verification: usize,
    pub(super) projects_with_failing_verification: usize,
    pub(super) projects_with_stale_verification: usize,
    pub(super) projects_safe_for_cleanup: usize,
    pub(super) projects_safe_for_refactor: usize,
    pub(super) cleanup_gate_distribution: GateDistribution,
    pub(super) refactor_gate_distribution: GateDistribution,
    projects: Vec<VerificationEvidenceProjectSummary>,
}

impl VerificationEvidenceWorkspaceSummary {
    pub(super) fn from_project_overviews(
        project_overviews: &[Value],
        project_count: usize,
        monitoring_count: usize,
    ) -> Self {
        let projects = project_overviews
            .iter()
            .map(VerificationEvidenceProjectSummary::from_project_overview)
            .collect::<Vec<_>>();
        let projects_with_recorded_verification = projects
            .iter()
            .filter(|project| project.has_recorded_verification())
            .count();
        let projects_missing_verification = projects
            .iter()
            .filter(|project| project.is_missing_verification())
            .count();
        let projects_with_failing_verification = projects
            .iter()
            .filter(|project| project.has_failing_verification())
            .count();
        let projects_with_stale_verification = projects
            .iter()
            .filter(|project| project.has_stale_verification())
            .count();
        let projects_safe_for_cleanup = projects
            .iter()
            .filter(|project| project.safe_for_cleanup)
            .count();
        let projects_safe_for_refactor = projects
            .iter()
            .filter(|project| project.safe_for_refactor)
            .count();
        let cleanup_gate_distribution = GateDistribution::from_levels(
            projects
                .iter()
                .map(|project| project.cleanup_gate_level.clone()),
        );
        let refactor_gate_distribution = GateDistribution::from_levels(
            projects
                .iter()
                .map(|project| project.refactor_gate_level.clone()),
        );

        Self {
            project_count,
            monitoring_count,
            projects_with_recorded_verification,
            projects_missing_verification,
            projects_with_failing_verification,
            projects_with_stale_verification,
            projects_safe_for_cleanup,
            projects_safe_for_refactor,
            cleanup_gate_distribution,
            refactor_gate_distribution,
            projects,
        }
    }

    pub(super) fn blocking_projects_json(&self) -> Vec<Value> {
        let mut blocking_projects = self
            .projects
            .iter()
            .filter(|project| project.is_blocking())
            .cloned()
            .collect::<Vec<_>>();
        blocking_projects.sort_by(|a, b| {
            b.failing_run_count
                .cmp(&a.failing_run_count)
                .then_with(|| {
                    b.is_missing_verification()
                        .cmp(&a.is_missing_verification())
                })
                .then_with(|| b.has_stale_verification().cmp(&a.has_stale_verification()))
                .then_with(|| {
                    a.project_id
                        .as_deref()
                        .unwrap_or("")
                        .cmp(b.project_id.as_deref().unwrap_or(""))
                })
        });
        blocking_projects
            .iter()
            .map(VerificationEvidenceProjectSummary::blocking_project_json)
            .collect()
    }

    pub(super) fn verified_conclusions_json(&self) -> Vec<Value> {
        let mut conclusions = Vec::new();
        if self.projects_safe_for_cleanup > 0 {
            conclusions.push(json!({
                "summary": format!(
                    "{} project(s) currently have verification evidence that supports cleanup review.",
                    self.projects_safe_for_cleanup
                ),
                "basis": [
                    "verification_evidence.safe_for_cleanup == true",
                    "latest recorded verification for those projects is not blocked"
                ]
            }));
        }
        if self.projects_safe_for_refactor > 0 {
            conclusions.push(json!({
                "summary": format!(
                    "{} project(s) currently have verification evidence that supports scoped refactor work.",
                    self.projects_safe_for_refactor
                ),
                "basis": [
                    "verification_evidence.safe_for_refactor == true",
                    "required test/build evidence is recorded for those projects"
                ]
            }));
        }
        conclusions
    }

    pub(super) fn unverified_conclusions_json(&self) -> Vec<Value> {
        let mut conclusions = Vec::new();
        if self.projects_missing_verification > 0 {
            conclusions.push(json!({
                "summary": format!(
                    "{} project(s) are still missing verification evidence.",
                    self.projects_missing_verification
                ),
                "basis": [
                    "verification_evidence.status == not_recorded"
                ]
            }));
        }
        if self.projects_with_stale_verification > 0 {
            conclusions.push(json!({
                "summary": format!(
                    "{} project(s) only have stale verification evidence.",
                    self.projects_with_stale_verification
                ),
                "basis": [
                    "observation.freshness.verification.status in [stale, unknown]"
                ]
            }));
        }
        if self.projects_with_failing_verification > 0 {
            conclusions.push(json!({
                "summary": format!(
                    "{} project(s) currently have failing or uncertain verification runs.",
                    self.projects_with_failing_verification
                ),
                "basis": [
                    "verification_evidence.failing_runs is non-empty"
                ]
            }));
        }
        conclusions
    }

    pub(super) fn direct_observations(&self) -> Vec<String> {
        vec![
            format!("Registered projects: {}.", self.project_count),
            format!(
                "Projects currently marked as monitoring: {}.",
                self.monitoring_count
            ),
            format!(
                "Projects with recorded verification evidence: {}.",
                self.projects_with_recorded_verification
            ),
            format!(
                "Projects missing verification evidence: {}.",
                self.projects_missing_verification
            ),
            format!(
                "Projects with failing or uncertain verification runs: {}.",
                self.projects_with_failing_verification
            ),
            format!(
                "Projects with stale verification evidence: {}.",
                self.projects_with_stale_verification
            ),
        ]
    }

    pub(super) fn confidence(&self) -> &'static str {
        if self.projects.is_empty() {
            "low"
        } else if self.projects_missing_verification == 0
            && self.projects_with_stale_verification == 0
        {
            "high"
        } else if self.projects_with_recorded_verification > 0 {
            "medium"
        } else {
            "low"
        }
    }
}

fn string_field(source: &Value, field: &str) -> Option<String> {
    source[field].as_str().map(str::to_string)
}

pub(super) fn gate_kinds(target: &str) -> (&'static [&'static str], &'static [&'static str]) {
    match target {
        "refactor" => (&["test", "build"], &["lint"]),
        _ => (&["test"], &["lint", "build"]),
    }
}

pub(super) fn latest_run_for_kind<'a>(
    runs: &'a [VerificationRun],
    kind: &str,
) -> Option<&'a VerificationRun> {
    runs.iter().find(|run| run.kind == kind)
}

pub(super) fn kind_state_sets<'a>(
    runs: &'a [VerificationRun],
    kinds: &[&'a str],
    now_secs: i64,
) -> (Vec<&'a str>, Vec<&'a str>) {
    let missing = kinds
        .iter()
        .copied()
        .filter(|kind| latest_run_for_kind(runs, kind).is_none())
        .collect::<Vec<_>>();
    let stale = kinds
        .iter()
        .copied()
        .filter(|kind| {
            latest_run_for_kind(runs, kind)
                .map(|run| verification_is_stale(std::slice::from_ref(run), now_secs))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    (missing, stale)
}

pub(super) fn failing_kinds(runs: &[VerificationRun]) -> Vec<&str> {
    runs.iter()
        .filter(|run| run.status != "passed")
        .map(|run| run.kind.as_str())
        .collect()
}

pub(super) fn pipeline_caution_kinds(runs: &[VerificationRun]) -> Vec<&str> {
    runs.iter()
        .filter(|run| run.status == "passed" && command_contains_pipeline_operators(&run.command))
        .map(|run| run.kind.as_str())
        .collect()
}

pub(super) fn suspicious_summary_signals(run: &VerificationRun) -> Vec<String> {
    if run.status != "passed" {
        return Vec::new();
    }
    detect_suspicious_pass_signals(run.summary.as_deref().unwrap_or_default(), "")
}

pub(super) fn suspicious_summary_kinds(runs: &[VerificationRun]) -> Vec<&str> {
    runs.iter()
        .filter(|run| !suspicious_summary_signals(run).is_empty())
        .map(|run| run.kind.as_str())
        .collect()
}

pub(super) fn blocker_reasons(
    target: &str,
    required_missing: &[&str],
    required_stale: &[&str],
    failing_kinds: &[&str],
) -> Vec<String> {
    let mut reasons = Vec::new();
    if !failing_kinds.is_empty() {
        reasons.push(
            "Recorded verification includes failing or uncertain runs that should be stabilized first."
                .to_string(),
        );
    }

    if required_missing.contains(&"test") {
        reasons.push(
            "Missing recorded test evidence required before cleanup or refactor work.".to_string(),
        );
    }
    if target == "refactor" && required_missing.contains(&"build") {
        reasons.push(
            "Missing recorded build evidence required before broader refactor work.".to_string(),
        );
    }
    if !required_stale.is_empty() {
        reasons.push(
            "Recorded verification evidence is stale and should be refreshed before risky changes."
                .to_string(),
        );
    }

    reasons
}

pub(super) fn gate_blockers(runs: &[VerificationRun], target: &str, now_secs: i64) -> Vec<String> {
    let (required_kinds, _) = gate_kinds(target);
    let (required_missing, required_stale) = kind_state_sets(runs, required_kinds, now_secs);
    let failing = failing_kinds(runs);
    blocker_reasons(target, &required_missing, &required_stale, &failing)
}

pub(super) fn gate_reasons(
    target: &str,
    required_missing: &[&str],
    required_stale: &[&str],
    advisory_missing: &[&str],
    advisory_stale: &[&str],
    failing: &[&str],
) -> Vec<String> {
    let reasons = blocker_reasons(target, required_missing, required_stale, failing);
    if !reasons.is_empty() {
        return reasons;
    }

    let advisory_gaps = advisory_missing
        .iter()
        .chain(advisory_stale.iter())
        .copied()
        .collect::<Vec<_>>();
    if advisory_gaps.is_empty() {
        return Vec::new();
    }

    vec![format!(
        "Advisory verification evidence is incomplete for {} review: {}.",
        target,
        advisory_gaps.join(", ")
    )]
}

pub(super) fn gate_next_steps(
    target: &str,
    required_missing: &[&str],
    required_stale: &[&str],
    advisory_missing: &[&str],
    advisory_stale: &[&str],
    failing: &[&str],
) -> Vec<String> {
    let mut steps = Vec::new();
    if !failing.is_empty() {
        steps.push(
            "Stabilize failing or uncertain verification runs before broader cleanup or refactor work."
                .to_string(),
        );
    }
    if required_missing.contains(&"test") {
        steps.push("Run and record project-native test evidence.".to_string());
    }
    if target == "refactor" && required_missing.contains(&"build") {
        steps.push("Run and record project-native build evidence.".to_string());
    }
    if !required_stale.is_empty() {
        steps.push("Refresh stale verification evidence before risky changes.".to_string());
    }
    if steps.is_empty() {
        let advisory_gaps = advisory_missing
            .iter()
            .chain(advisory_stale.iter())
            .copied()
            .collect::<Vec<_>>();
        if !advisory_gaps.is_empty() {
            steps.push(format!(
                "Refresh advisory verification evidence when possible: {}.",
                advisory_gaps.join(", ")
            ));
        }
    }
    if steps.is_empty() {
        steps.push("Current verification evidence supports the requested review mode.".to_string());
    }

    steps
}

#[cfg(test)]
pub(super) fn gate_assessment(runs: &[VerificationRun], target: &str, now_secs: i64) -> Value {
    VerificationGateAssessment::from_runs(runs, VerificationGateTarget::from_name(target), now_secs)
        .to_json()
}

pub(super) fn project_gate_level(project: &Value, target: &str) -> String {
    project["verification_evidence"]["gate_assessment"][target]["level"]
        .as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            let key = match target {
                "refactor" => "safe_for_refactor",
                _ => "safe_for_cleanup",
            };
            if project[key].as_bool().unwrap_or(false) {
                "allow".to_string()
            } else {
                "blocked".to_string()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::{
        VerificationEvidenceWorkspaceSummary, VerificationGateAssessment, VerificationGateTarget,
        VerificationStatusSummary,
    };
    use crate::storage::queries::VerificationRun;
    use serde_json::json;

    const NOW: i64 = 1_700_000_000;

    fn make_run(kind: &str, status: &str, finished_at: &str) -> VerificationRun {
        VerificationRun {
            id: 1,
            kind: kind.to_string(),
            status: status.to_string(),
            command: format!("run-{}", kind),
            exit_code: Some(0),
            summary: Some(format!("{} summary", kind)),
            source: "test".to_string(),
            started_at: Some(finished_at.to_string()),
            finished_at: finished_at.to_string(),
        }
    }

    #[test]
    fn gate_assessment_model_blocks_missing_required_test() {
        let assessment =
            VerificationGateAssessment::from_runs(&[], VerificationGateTarget::Cleanup, NOW);

        assert!(!assessment.allowed);
        assert_eq!(assessment.level, "blocked");
        assert_eq!(assessment.required_kinds, vec!["test"]);
        assert_eq!(assessment.advisory_kinds, vec!["lint", "build"]);
        assert_eq!(assessment.missing_kinds, vec!["test", "lint", "build"]);
        assert!(assessment
            .reasons
            .iter()
            .any(|reason| reason.contains("Missing recorded test evidence")));
        assert_eq!(assessment.to_json()["level"], "blocked");
    }

    #[test]
    fn status_summary_model_marks_clean_full_evidence_available() {
        let runs = vec![
            make_run("test", "passed", "1700000000"),
            make_run("lint", "passed", "1700000000"),
            make_run("build", "passed", "1700000000"),
        ];

        let summary = VerificationStatusSummary::from_runs(&runs, NOW);
        let payload = summary.to_json();

        assert_eq!(summary.status, "available");
        assert!(summary.all_expected_kinds_recorded);
        assert!(summary.safe_for_cleanup);
        assert!(summary.safe_for_refactor);
        assert!(summary.missing_kinds.is_empty());
        assert_eq!(payload["latest_runs"][0]["trust_level"], "trusted");
        assert_eq!(payload["gate_assessment"]["cleanup"]["level"], "allow");
        assert_eq!(payload["gate_assessment"]["refactor"]["level"], "allow");
    }

    #[test]
    fn workspace_summary_counts_gate_levels_and_safety_flags() {
        let projects = vec![
            json!({
                "project_id": "ready",
                "safe_for_cleanup": true,
                "safe_for_refactor": true,
                "safe_for_cleanup_reason": "cleanup ok",
                "safe_for_refactor_reason": "refactor ok",
                "verification_evidence": {
                    "status": "available",
                    "failing_runs": [],
                    "gate_assessment": {
                        "cleanup": {"level": "allow"},
                        "refactor": {"level": "allow"}
                    }
                },
                "observation": {"freshness": {"verification": {"status": "fresh"}}}
            }),
            json!({
                "project_id": "stale",
                "safe_for_cleanup": true,
                "safe_for_refactor": false,
                "safe_for_cleanup_reason": "cleanup caution",
                "safe_for_refactor_reason": "Refactor readiness is blocked.",
                "verification_evidence": {
                    "status": "available",
                    "failing_runs": [],
                    "gate_assessment": {
                        "cleanup": {"level": "caution"},
                        "refactor": {"level": "blocked"}
                    }
                },
                "observation": {"freshness": {"verification": {"status": "stale"}}}
            }),
            json!({
                "project_id": "missing",
                "safe_for_cleanup": false,
                "safe_for_refactor": false,
                "safe_for_cleanup_reason": "Cleanup readiness is blocked.",
                "safe_for_refactor_reason": "Refactor readiness is blocked.",
                "verification_evidence": {
                    "status": "not_recorded",
                    "failing_runs": [],
                    "gate_assessment": {
                        "cleanup": {"level": "blocked"},
                        "refactor": {"level": "blocked"}
                    }
                },
                "observation": {"freshness": {"verification": {"status": "unknown"}}}
            }),
        ];

        let summary = VerificationEvidenceWorkspaceSummary::from_project_overviews(&projects, 3, 1);

        assert_eq!(summary.project_count, 3);
        assert_eq!(summary.monitoring_count, 1);
        assert_eq!(summary.projects_with_recorded_verification, 2);
        assert_eq!(summary.projects_missing_verification, 1);
        assert_eq!(summary.projects_with_failing_verification, 0);
        assert_eq!(summary.projects_with_stale_verification, 2);
        assert_eq!(summary.projects_safe_for_cleanup, 2);
        assert_eq!(summary.projects_safe_for_refactor, 1);
        assert_eq!(summary.cleanup_gate_distribution.allow, 1);
        assert_eq!(summary.cleanup_gate_distribution.caution, 1);
        assert_eq!(summary.cleanup_gate_distribution.blocked, 1);
        assert_eq!(summary.refactor_gate_distribution.allow, 1);
        assert_eq!(summary.refactor_gate_distribution.caution, 0);
        assert_eq!(summary.refactor_gate_distribution.blocked, 2);
        assert_eq!(summary.confidence(), "medium");
    }

    #[test]
    fn blocking_projects_sort_by_failing_missing_stale_then_project_id() {
        let projects = vec![
            json!({
                "project_id": "stale-a",
                "safe_for_cleanup": false,
                "safe_for_refactor": true,
                "safe_for_cleanup_reason": "stale cleanup",
                "safe_for_refactor_reason": "refactor ok",
                "verification_evidence": {
                    "status": "available",
                    "failing_runs": [],
                    "gate_assessment": {
                        "cleanup": {"level": "blocked"},
                        "refactor": {"level": "allow"}
                    }
                },
                "observation": {"freshness": {"verification": {"status": "stale"}}}
            }),
            json!({
                "project_id": "failing",
                "safe_for_cleanup": false,
                "safe_for_refactor": false,
                "safe_for_cleanup_reason": "failing cleanup",
                "safe_for_refactor_reason": "failing refactor",
                "verification_evidence": {
                    "status": "available",
                    "failing_runs": [{"kind": "test"}],
                    "gate_assessment": {
                        "cleanup": {"level": "blocked"},
                        "refactor": {"level": "blocked"}
                    }
                },
                "observation": {"freshness": {"verification": {"status": "fresh"}}}
            }),
            json!({
                "project_id": "missing",
                "safe_for_cleanup": false,
                "safe_for_refactor": false,
                "safe_for_cleanup_reason": "missing cleanup",
                "safe_for_refactor_reason": "missing refactor",
                "verification_evidence": {
                    "status": "not_recorded",
                    "failing_runs": [],
                    "gate_assessment": {
                        "cleanup": {"level": "blocked"},
                        "refactor": {"level": "blocked"}
                    }
                },
                "observation": {"freshness": {"verification": {"status": "unknown"}}}
            }),
        ];

        let summary = VerificationEvidenceWorkspaceSummary::from_project_overviews(&projects, 3, 0);
        let blocking = summary.blocking_projects_json();

        assert_eq!(blocking[0]["project_id"], "failing");
        assert_eq!(blocking[1]["project_id"], "missing");
        assert_eq!(blocking[2]["project_id"], "stale-a");
        assert_eq!(blocking[0]["failing_run_count"], 1);
        assert_eq!(blocking[0]["primary_reason"], "failing cleanup");
        assert_eq!(blocking[1]["verification_status"], "not_recorded");
        assert_eq!(
            blocking[2]["verification_freshness"],
            json!({"status": "stale"})
        );
    }
}
