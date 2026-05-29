use crate::core::verification::command_contains_pipeline_operators;
use crate::storage::queries::VerificationRun;
use serde_json::{json, Value};

use super::super::super::constraints::readiness_reason_summary;
use super::super::super::observation::{freshness_detail, latest_verification_timestamp};
use super::{
    gate_blockers, pipeline_caution_kinds, suspicious_summary_kinds, suspicious_summary_signals,
    VerificationGateAssessment, VerificationGateTarget, EXPECTED_KINDS,
};

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
pub(in crate::mcp::verification_evidence) struct VerificationStatusSummary {
    pub(in crate::mcp::verification_evidence) status: &'static str,
    summary: &'static str,
    latest_runs: Vec<VerificationLatestRunSummary>,
    latest_finished_at: Option<String>,
    freshness: Value,
    pub(in crate::mcp::verification_evidence) missing_kinds: Vec<String>,
    pub(in crate::mcp::verification_evidence) all_expected_kinds_recorded: bool,
    pub(in crate::mcp::verification_evidence) safe_for_cleanup: bool,
    pub(in crate::mcp::verification_evidence) safe_for_refactor: bool,
    cleanup_blockers: Vec<String>,
    refactor_blockers: Vec<String>,
    cleanup_gate: VerificationGateAssessment,
    refactor_gate: VerificationGateAssessment,
    failing_runs: Vec<VerificationFailingRunSummary>,
}

impl VerificationStatusSummary {
    pub(in crate::mcp::verification_evidence) fn from_runs(
        runs: &[VerificationRun],
        now_secs: i64,
    ) -> Self {
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

    pub(in crate::mcp::verification_evidence) fn to_json(&self) -> Value {
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
pub(in crate::mcp::verification_evidence) struct GateDistribution {
    pub(in crate::mcp::verification_evidence) allow: usize,
    pub(in crate::mcp::verification_evidence) caution: usize,
    pub(in crate::mcp::verification_evidence) blocked: usize,
}

impl GateDistribution {
    pub(super) fn from_levels(levels: impl Iterator<Item = String>) -> Self {
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

    pub(in crate::mcp::verification_evidence) fn to_json(&self) -> Value {
        json!({
            "allow": self.allow,
            "caution": self.caution,
            "blocked": self.blocked,
        })
    }
}
