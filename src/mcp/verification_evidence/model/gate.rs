use crate::core::verification::{
    command_contains_pipeline_operators, detect_suspicious_pass_signals,
};
use crate::storage::queries::VerificationRun;
use serde_json::{json, Value};

use super::super::super::observation::{freshness_policy, verification_is_stale};
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

pub(in crate::mcp::verification_evidence) fn gate_kinds(
    target: &str,
) -> (&'static [&'static str], &'static [&'static str]) {
    match target {
        "refactor" => (&["test", "build"], &["lint"]),
        _ => (&["test"], &["lint", "build"]),
    }
}

pub(in crate::mcp::verification_evidence) fn latest_run_for_kind<'a>(
    runs: &'a [VerificationRun],
    kind: &str,
) -> Option<&'a VerificationRun> {
    runs.iter().find(|run| run.kind == kind)
}

pub(in crate::mcp::verification_evidence) fn kind_state_sets<'a>(
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

pub(in crate::mcp::verification_evidence) fn failing_kinds(runs: &[VerificationRun]) -> Vec<&str> {
    runs.iter()
        .filter(|run| run.status != "passed")
        .map(|run| run.kind.as_str())
        .collect()
}

pub(in crate::mcp::verification_evidence) fn pipeline_caution_kinds(
    runs: &[VerificationRun],
) -> Vec<&str> {
    runs.iter()
        .filter(|run| run.status == "passed" && command_contains_pipeline_operators(&run.command))
        .map(|run| run.kind.as_str())
        .collect()
}

pub(in crate::mcp::verification_evidence) fn suspicious_summary_signals(
    run: &VerificationRun,
) -> Vec<String> {
    if run.status != "passed" {
        return Vec::new();
    }
    detect_suspicious_pass_signals(run.summary.as_deref().unwrap_or_default(), "")
}

pub(in crate::mcp::verification_evidence) fn suspicious_summary_kinds(
    runs: &[VerificationRun],
) -> Vec<&str> {
    runs.iter()
        .filter(|run| !suspicious_summary_signals(run).is_empty())
        .map(|run| run.kind.as_str())
        .collect()
}

pub(in crate::mcp::verification_evidence) fn blocker_reasons(
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

pub(in crate::mcp::verification_evidence) fn gate_blockers(
    runs: &[VerificationRun],
    target: &str,
    now_secs: i64,
) -> Vec<String> {
    let (required_kinds, _) = gate_kinds(target);
    let (required_missing, required_stale) = kind_state_sets(runs, required_kinds, now_secs);
    let failing = failing_kinds(runs);
    blocker_reasons(target, &required_missing, &required_stale, &failing)
}

pub(in crate::mcp::verification_evidence) fn gate_reasons(
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

pub(in crate::mcp::verification_evidence) fn gate_next_steps(
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
