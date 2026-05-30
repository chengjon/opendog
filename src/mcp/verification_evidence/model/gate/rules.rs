use crate::core::verification::{
    command_contains_pipeline_operators, detect_suspicious_pass_signals,
};
use crate::storage::queries::VerificationRun;

use super::super::super::super::observation::verification_is_stale;

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
