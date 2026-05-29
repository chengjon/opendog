use crate::core::verification::{
    command_contains_pipeline_operators, detect_suspicious_pass_signals,
};
use crate::storage::queries::VerificationRun;
use serde_json::{json, Value};

use super::super::observation::{freshness_policy, verification_is_stale};

mod status;
mod workspace;

pub(super) use status::{GateDistribution, VerificationStatusSummary};
pub(super) use workspace::VerificationEvidenceWorkspaceSummary;

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
