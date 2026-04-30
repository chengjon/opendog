use serde_json::{json, Value};

use crate::contracts::{
    MCP_RECORD_VERIFICATION_V1, MCP_RUN_VERIFICATION_V1, MCP_VERIFICATION_STATUS_V1,
};
use crate::core::verification::ExecutedVerificationResult;
use crate::storage::queries::VerificationRun;

use super::constraints::readiness_reason_summary;
use super::observation::{freshness_detail, latest_verification_timestamp, verification_is_stale};
use super::{now_unix_secs, versioned_project_payload};

pub(crate) fn verification_status_layer(runs: &[VerificationRun]) -> Value {
    let now_secs = now_unix_secs();
    let expected_kinds = ["test", "lint", "build"];
    let recorded_kinds: Vec<&str> = runs.iter().map(|r| r.kind.as_str()).collect();
    let missing_kinds: Vec<&str> = expected_kinds
        .iter()
        .copied()
        .filter(|kind| !recorded_kinds.iter().any(|recorded| recorded == kind))
        .collect();
    let all_recorded = missing_kinds.is_empty();
    let cleanup_gate = gate_assessment(runs, "cleanup", now_secs);
    let refactor_gate = gate_assessment(runs, "refactor", now_secs);
    let safe_for_cleanup = cleanup_gate["allowed"].as_bool().unwrap_or(false);
    let safe_for_refactor = refactor_gate["allowed"].as_bool().unwrap_or(false);
    let cleanup_blockers = gate_blockers(runs, "cleanup", now_secs);
    let refactor_blockers = gate_blockers(runs, "refactor", now_secs);
    let latest_finished_at = latest_verification_timestamp(runs);
    let freshness = freshness_detail(
        "verification",
        latest_finished_at.as_deref(),
        !runs.is_empty(),
        now_secs,
    );

    if runs.is_empty() {
        json!({
            "status": "not_recorded",
            "summary": "No test/lint/build results have been recorded yet.",
            "latest_runs": [],
            "latest_finished_at": latest_finished_at,
            "freshness": freshness,
            "missing_kinds": expected_kinds,
            "all_expected_kinds_recorded": false,
            "safe_for_cleanup": false,
            "safe_for_refactor": false,
            "cleanup_blockers": cleanup_blockers,
            "refactor_blockers": refactor_blockers,
            "safe_for_cleanup_reason": readiness_reason_summary("cleanup", false, &cleanup_blockers),
            "safe_for_refactor_reason": readiness_reason_summary("refactor", false, &refactor_blockers),
            "gate_assessment": {
                "cleanup": cleanup_gate,
                "refactor": refactor_gate,
            },
        })
    } else {
        let failing_runs: Vec<&VerificationRun> =
            runs.iter().filter(|r| r.status != "passed").collect();
        json!({
            "status": "available",
            "summary": if failing_runs.is_empty() {
                "Recorded verification results exist and the latest known runs are passing."
            } else {
                "Recorded verification results include failing or uncertain runs."
            },
            "latest_runs": runs.iter().map(|run| json!({
                "kind": run.kind,
                "status": run.status,
                "command": run.command,
                "exit_code": run.exit_code,
                "summary": run.summary,
                "source": run.source,
                "finished_at": run.finished_at,
            })).collect::<Vec<_>>(),
            "latest_finished_at": latest_finished_at,
            "freshness": freshness,
            "failing_runs": failing_runs.iter().map(|run| json!({
                "kind": run.kind,
                "status": run.status,
                "command": run.command,
            })).collect::<Vec<_>>(),
            "missing_kinds": missing_kinds,
            "all_expected_kinds_recorded": all_recorded,
            "safe_for_cleanup": safe_for_cleanup,
            "safe_for_refactor": safe_for_refactor,
            "cleanup_blockers": cleanup_blockers,
            "refactor_blockers": refactor_blockers,
            "safe_for_cleanup_reason": readiness_reason_summary("cleanup", safe_for_cleanup, &cleanup_blockers),
            "safe_for_refactor_reason": readiness_reason_summary("refactor", safe_for_refactor, &refactor_blockers),
            "gate_assessment": {
                "cleanup": cleanup_gate,
                "refactor": refactor_gate,
            },
        })
    }
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

fn project_gate_level(project: &Value, target: &str) -> String {
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

pub(super) fn workspace_verification_evidence_layer(
    project_overviews: &[Value],
    project_count: usize,
    monitoring_count: usize,
) -> Value {
    let projects_with_recorded_verification = project_overviews
        .iter()
        .filter(|p| p["verification_evidence"]["status"] == "available")
        .count();
    let projects_missing_verification = project_overviews
        .iter()
        .filter(|p| p["verification_evidence"]["status"] == "not_recorded")
        .count();
    let projects_with_failing_verification = project_overviews
        .iter()
        .filter(|p| {
            p["verification_evidence"]["failing_runs"]
                .as_array()
                .map(|runs| !runs.is_empty())
                .unwrap_or(false)
        })
        .count();
    let projects_with_stale_verification = project_overviews
        .iter()
        .filter(|p| {
            matches!(
                p["observation"]["freshness"]["verification"]["status"]
                    .as_str()
                    .unwrap_or(""),
                "stale" | "unknown"
            )
        })
        .count();
    let projects_safe_for_cleanup = project_overviews
        .iter()
        .filter(|p| p["safe_for_cleanup"].as_bool().unwrap_or(false))
        .count();
    let projects_safe_for_refactor = project_overviews
        .iter()
        .filter(|p| p["safe_for_refactor"].as_bool().unwrap_or(false))
        .count();
    let cleanup_gate_distribution = json!({
        "allow": project_overviews
            .iter()
            .filter(|p| project_gate_level(p, "cleanup") == "allow")
            .count(),
        "caution": project_overviews
            .iter()
            .filter(|p| project_gate_level(p, "cleanup") == "caution")
            .count(),
        "blocked": project_overviews
            .iter()
            .filter(|p| project_gate_level(p, "cleanup") == "blocked")
            .count(),
    });
    let refactor_gate_distribution = json!({
        "allow": project_overviews
            .iter()
            .filter(|p| project_gate_level(p, "refactor") == "allow")
            .count(),
        "caution": project_overviews
            .iter()
            .filter(|p| project_gate_level(p, "refactor") == "caution")
            .count(),
        "blocked": project_overviews
            .iter()
            .filter(|p| project_gate_level(p, "refactor") == "blocked")
            .count(),
    });

    let mut blocking_projects: Vec<Value> = project_overviews
        .iter()
        .filter(|p| {
            !p["safe_for_cleanup"].as_bool().unwrap_or(false)
                || !p["safe_for_refactor"].as_bool().unwrap_or(false)
        })
        .map(|p| {
            let cleanup_reason = p["safe_for_cleanup_reason"]
                .as_str()
                .unwrap_or("Cleanup readiness is blocked.")
                .to_string();
            let refactor_reason = p["safe_for_refactor_reason"]
                .as_str()
                .unwrap_or("Refactor readiness is blocked.")
                .to_string();
            let primary_reason = if p["verification_evidence"]["failing_runs"]
                .as_array()
                .map(|runs| !runs.is_empty())
                .unwrap_or(false)
            {
                cleanup_reason.clone()
            } else if p["verification_evidence"]["status"] == "not_recorded" {
                cleanup_reason.clone()
            } else if matches!(
                p["observation"]["freshness"]["verification"]["status"]
                    .as_str()
                    .unwrap_or(""),
                "stale" | "unknown"
            ) {
                cleanup_reason.clone()
            } else {
                refactor_reason.clone()
            };

            json!({
                "project_id": p["project_id"].clone(),
                "verification_status": p["verification_evidence"]["status"].clone(),
                "verification_freshness": p["observation"]["freshness"]["verification"].clone(),
                "failing_run_count": p["verification_evidence"]["failing_runs"]
                    .as_array()
                    .map(|runs| runs.len())
                    .unwrap_or(0),
                "safe_for_cleanup": p["safe_for_cleanup"].clone(),
                "safe_for_refactor": p["safe_for_refactor"].clone(),
                "cleanup_gate_level": project_gate_level(p, "cleanup"),
                "refactor_gate_level": project_gate_level(p, "refactor"),
                "cleanup_reason": cleanup_reason,
                "refactor_reason": refactor_reason,
                "primary_reason": primary_reason,
            })
        })
        .collect();

    blocking_projects.sort_by(|a, b| {
        let a_failing = a["failing_run_count"].as_u64().unwrap_or(0);
        let b_failing = b["failing_run_count"].as_u64().unwrap_or(0);
        let a_missing = a["verification_status"] == "not_recorded";
        let b_missing = b["verification_status"] == "not_recorded";
        let a_stale = matches!(
            a["verification_freshness"]["status"].as_str().unwrap_or(""),
            "stale" | "unknown"
        );
        let b_stale = matches!(
            b["verification_freshness"]["status"].as_str().unwrap_or(""),
            "stale" | "unknown"
        );

        b_failing
            .cmp(&a_failing)
            .then_with(|| b_missing.cmp(&a_missing))
            .then_with(|| b_stale.cmp(&a_stale))
            .then_with(|| {
                a["project_id"]
                    .as_str()
                    .unwrap_or("")
                    .cmp(b["project_id"].as_str().unwrap_or(""))
            })
    });

    let mut verified_conclusions = Vec::new();
    if projects_safe_for_cleanup > 0 {
        verified_conclusions.push(json!({
            "summary": format!(
                "{} project(s) currently have verification evidence that supports cleanup review.",
                projects_safe_for_cleanup
            ),
            "basis": [
                "verification_evidence.safe_for_cleanup == true",
                "latest recorded verification for those projects is not blocked"
            ]
        }));
    }
    if projects_safe_for_refactor > 0 {
        verified_conclusions.push(json!({
            "summary": format!(
                "{} project(s) currently have verification evidence that supports scoped refactor work.",
                projects_safe_for_refactor
            ),
            "basis": [
                "verification_evidence.safe_for_refactor == true",
                "required test/build evidence is recorded for those projects"
            ]
        }));
    }

    let mut unverified_conclusions = Vec::new();
    if projects_missing_verification > 0 {
        unverified_conclusions.push(json!({
            "summary": format!(
                "{} project(s) are still missing verification evidence.",
                projects_missing_verification
            ),
            "basis": [
                "verification_evidence.status == not_recorded"
            ]
        }));
    }
    if projects_with_stale_verification > 0 {
        unverified_conclusions.push(json!({
            "summary": format!(
                "{} project(s) only have stale verification evidence.",
                projects_with_stale_verification
            ),
            "basis": [
                "observation.freshness.verification.status in [stale, unknown]"
            ]
        }));
    }
    if projects_with_failing_verification > 0 {
        unverified_conclusions.push(json!({
            "summary": format!(
                "{} project(s) currently have failing or uncertain verification runs.",
                projects_with_failing_verification
            ),
            "basis": [
                "verification_evidence.failing_runs is non-empty"
            ]
        }));
    }

    json!({
        "status": "available",
        "projects_with_recorded_verification": projects_with_recorded_verification,
        "projects_missing_verification": projects_missing_verification,
        "projects_with_failing_verification": projects_with_failing_verification,
        "projects_with_stale_verification": projects_with_stale_verification,
        "projects_safe_for_cleanup": projects_safe_for_cleanup,
        "projects_safe_for_refactor": projects_safe_for_refactor,
        "cleanup_gate_distribution": cleanup_gate_distribution,
        "refactor_gate_distribution": refactor_gate_distribution,
        "direct_observations": [
            format!("Registered projects: {}.", project_count),
            format!("Projects currently marked as monitoring: {}.", monitoring_count),
            format!(
                "Projects with recorded verification evidence: {}.",
                projects_with_recorded_verification
            ),
            format!(
                "Projects missing verification evidence: {}.",
                projects_missing_verification
            ),
            format!(
                "Projects with failing or uncertain verification runs: {}.",
                projects_with_failing_verification
            ),
            format!(
                "Projects with stale verification evidence: {}.",
                projects_with_stale_verification
            ),
        ],
        "inferences": [
            "Project recommendations can rely more strongly on projects whose verification evidence is both recorded and fresh.",
            "Missing or stale verification evidence should be refreshed before broad cleanup or refactor work.",
            "Failing verification evidence should outrank cleanup-oriented follow-up work."
        ],
        "verified_conclusions": verified_conclusions,
        "unverified_conclusions": unverified_conclusions,
        "blocking_projects": blocking_projects,
        "confidence": if project_overviews.is_empty() {
            "low"
        } else if projects_missing_verification == 0 && projects_with_stale_verification == 0 {
            "high"
        } else if projects_with_recorded_verification > 0 {
            "medium"
        } else {
            "low"
        },
    })
}

fn gate_kinds(target: &str) -> (&'static [&'static str], &'static [&'static str]) {
    match target {
        "refactor" => (&["test", "build"], &["lint"]),
        _ => (&["test"], &["lint", "build"]),
    }
}

fn latest_run_for_kind<'a>(runs: &'a [VerificationRun], kind: &str) -> Option<&'a VerificationRun> {
    runs.iter().find(|run| run.kind == kind)
}

fn kind_state_sets<'a>(
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

fn failing_kinds(runs: &[VerificationRun]) -> Vec<&str> {
    runs.iter()
        .filter(|run| run.status != "passed")
        .map(|run| run.kind.as_str())
        .collect()
}

fn blocker_reasons(
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

fn gate_blockers(runs: &[VerificationRun], target: &str, now_secs: i64) -> Vec<String> {
    let (required_kinds, _) = gate_kinds(target);
    let (required_missing, required_stale) = kind_state_sets(runs, required_kinds, now_secs);
    let failing = failing_kinds(runs);
    blocker_reasons(target, &required_missing, &required_stale, &failing)
}

fn gate_reasons(
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

fn gate_next_steps(
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

fn gate_assessment(runs: &[VerificationRun], target: &str, now_secs: i64) -> Value {
    let (required_kinds, advisory_kinds) = gate_kinds(target);
    let (required_missing, required_stale) = kind_state_sets(runs, required_kinds, now_secs);
    let (advisory_missing, advisory_stale) = kind_state_sets(runs, advisory_kinds, now_secs);
    let failing = failing_kinds(runs);
    let blockers = blocker_reasons(target, &required_missing, &required_stale, &failing);
    let reasons = gate_reasons(
        target,
        &required_missing,
        &required_stale,
        &advisory_missing,
        &advisory_stale,
        &failing,
    );
    let level = if !blockers.is_empty() {
        "blocked"
    } else if !advisory_missing.is_empty() || !advisory_stale.is_empty() {
        "caution"
    } else {
        "allow"
    };

    let missing_kinds = required_missing
        .iter()
        .chain(advisory_missing.iter())
        .copied()
        .collect::<Vec<_>>();
    let stale_kinds = required_stale
        .iter()
        .chain(advisory_stale.iter())
        .copied()
        .collect::<Vec<_>>();

    json!({
        "allowed": blockers.is_empty(),
        "level": level,
        "required_kinds": required_kinds,
        "advisory_kinds": advisory_kinds,
        "missing_kinds": missing_kinds,
        "failing_kinds": failing,
        "stale_kinds": stale_kinds,
        "reasons": reasons,
        "next_steps": gate_next_steps(
            target,
            &required_missing,
            &required_stale,
            &advisory_missing,
            &advisory_stale,
            &failing
        ),
    })
}
