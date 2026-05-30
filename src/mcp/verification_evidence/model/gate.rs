use crate::storage::queries::VerificationRun;
use serde_json::{json, Value};

use super::super::super::observation::freshness_policy;

mod rules;

pub(in crate::mcp::verification_evidence) use rules::{
    blocker_reasons, failing_kinds, gate_blockers, gate_kinds, gate_next_steps, gate_reasons,
    kind_state_sets, pipeline_caution_kinds, suspicious_summary_kinds, suspicious_summary_signals,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::mcp::verification_evidence) enum VerificationGateTarget {
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
pub(in crate::mcp::verification_evidence) struct VerificationGateAssessment {
    pub(in crate::mcp::verification_evidence) allowed: bool,
    pub(in crate::mcp::verification_evidence) level: String,
    pub(in crate::mcp::verification_evidence) required_kinds: Vec<String>,
    pub(in crate::mcp::verification_evidence) advisory_kinds: Vec<String>,
    pub(in crate::mcp::verification_evidence) missing_kinds: Vec<String>,
    pub(in crate::mcp::verification_evidence) failing_kinds: Vec<String>,
    pub(in crate::mcp::verification_evidence) stale_kinds: Vec<String>,
    pub(in crate::mcp::verification_evidence) pipeline_caution_kinds: Vec<String>,
    pub(in crate::mcp::verification_evidence) suspicious_summary_kinds: Vec<String>,
    pub(in crate::mcp::verification_evidence) reasons: Vec<String>,
    pub(in crate::mcp::verification_evidence) next_steps: Vec<String>,
}

impl VerificationGateAssessment {
    pub(in crate::mcp::verification_evidence) fn from_runs(
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

    pub(in crate::mcp::verification_evidence) fn to_json(&self) -> Value {
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

#[cfg(test)]
pub(in crate::mcp::verification_evidence) fn gate_assessment(
    runs: &[VerificationRun],
    target: &str,
    now_secs: i64,
) -> Value {
    VerificationGateAssessment::from_runs(runs, VerificationGateTarget::from_name(target), now_secs)
        .to_json()
}

pub(in crate::mcp::verification_evidence) fn project_gate_level(
    project: &Value,
    target: &str,
) -> String {
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
