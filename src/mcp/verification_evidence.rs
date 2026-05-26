use serde_json::{json, Value};

use crate::contracts::{
    MCP_RECORD_VERIFICATION_V1, MCP_RUN_VERIFICATION_V1, MCP_VERIFICATION_STATUS_V1,
};
use crate::core::verification::command_contains_pipeline_operators;
use crate::core::verification::ExecutedVerificationResult;
use crate::storage::queries::VerificationRun;

use super::constraints::readiness_reason_summary;
use super::observation::{
    freshness_detail, freshness_policy, latest_verification_timestamp, verification_is_stale,
};
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
        let pipeline_caution_runs: Vec<&VerificationRun> = runs
            .iter()
            .filter(|r| r.status == "passed" && command_contains_pipeline_operators(&r.command))
            .collect();
        json!({
            "status": "available",
            "summary": if failing_runs.is_empty() && pipeline_caution_runs.is_empty() {
                "Recorded verification results exist and the latest known runs are passing."
            } else if !pipeline_caution_runs.is_empty() {
                "Recorded verification results exist but some passed runs used pipeline commands whose exit codes may be masked."
            } else {
                "Recorded verification results include failing or uncertain runs."
            },
            "latest_runs": runs.iter().map(|run| {
                let pipeline = command_contains_pipeline_operators(&run.command);
                let masked = pipeline && run.status == "passed";
                json!({
                    "kind": run.kind,
                    "status": run.status,
                    "command": run.command,
                    "exit_code": run.exit_code,
                    "summary": run.summary,
                    "source": run.source,
                    "finished_at": run.finished_at,
                    "exit_code_masked_possible": masked,
                    "trust_level": if masked { "caution" } else { "trusted" },
                })
            }).collect::<Vec<_>>(),
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
            let cleanup_blocked_by_verification = p["verification_evidence"]["failing_runs"]
                .as_array()
                .map(|runs| !runs.is_empty())
                .unwrap_or(false)
                || p["verification_evidence"]["status"] == "not_recorded"
                || matches!(
                    p["observation"]["freshness"]["verification"]["status"]
                        .as_str()
                        .unwrap_or(""),
                    "stale" | "unknown"
                );
            let primary_reason = if cleanup_blocked_by_verification {
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

fn pipeline_caution_kinds(runs: &[VerificationRun]) -> Vec<&str> {
    runs.iter()
        .filter(|run| run.status == "passed" && command_contains_pipeline_operators(&run.command))
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
    let pipeline_caution_kinds = pipeline_caution_kinds(runs);
    let blockers = blocker_reasons(target, &required_missing, &required_stale, &failing);
    let mut reasons = gate_reasons(
        target,
        &required_missing,
        &required_stale,
        &advisory_missing,
        &advisory_stale,
        &failing,
    );
    if !pipeline_caution_kinds.is_empty() {
        reasons.push(format!(
            "Passed {} verification used pipeline commands whose exit codes may be masked. Consider rerunning without pipes.",
            pipeline_caution_kinds.join(", ")
        ));
    }
    let level = if !blockers.is_empty() {
        "blocked"
    } else if !advisory_missing.is_empty()
        || !advisory_stale.is_empty()
        || !pipeline_caution_kinds.is_empty()
    {
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

    let mut next_steps = gate_next_steps(
        target,
        &required_missing,
        &required_stale,
        &advisory_missing,
        &advisory_stale,
        &failing,
    );
    if !pipeline_caution_kinds.is_empty() {
        next_steps.push(
            "Rerun pipeline commands without pipes for more reliable exit-code capture.".to_string(),
        );
    }

    json!({
        "allowed": blockers.is_empty(),
        "level": level,
        "required_kinds": required_kinds,
        "advisory_kinds": advisory_kinds,
        "missing_kinds": missing_kinds,
        "failing_kinds": failing,
        "stale_kinds": stale_kinds,
        "pipeline_caution_kinds": pipeline_caution_kinds,
        "freshness_policy": freshness_policy(),
        "reasons": reasons,
        "next_steps": next_steps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::queries::VerificationRun;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    const NOW: i64 = 1_700_000_000;

    fn current_unix_secs() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

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

    // ---- gate_kinds ----

    #[test]
    fn gate_kinds_cleanup_returns_test_required() {
        let (required, advisory) = gate_kinds("cleanup");
        assert_eq!(required, &["test"]);
        assert_eq!(advisory, &["lint", "build"]);
    }

    #[test]
    fn gate_kinds_refactor_returns_test_build_required() {
        let (required, advisory) = gate_kinds("refactor");
        assert_eq!(required, &["test", "build"]);
        assert_eq!(advisory, &["lint"]);
    }

    #[test]
    fn gate_kinds_unknown_target_defaults_to_cleanup() {
        let (required, advisory) = gate_kinds("other");
        assert_eq!(required, &["test"]);
        assert_eq!(advisory, &["lint", "build"]);
    }

    // ---- failing_kinds ----

    #[test]
    fn failing_kinds_empty_runs() {
        let runs: Vec<VerificationRun> = vec![];
        assert!(failing_kinds(&runs).is_empty());
    }

    #[test]
    fn failing_kinds_all_passed() {
        let runs = vec![
            make_run("test", "passed", "1700000000"),
            make_run("lint", "passed", "1700000000"),
        ];
        assert!(failing_kinds(&runs).is_empty());
    }

    #[test]
    fn failing_kinds_some_failing() {
        let runs = vec![
            make_run("test", "passed", "1700000000"),
            make_run("lint", "failed", "1700000000"),
            make_run("build", "uncertain", "1700000000"),
        ];
        let failed = failing_kinds(&runs);
        assert_eq!(failed, vec!["lint", "build"]);
    }

    // ---- kind_state_sets ----

    #[test]
    fn kind_state_sets_all_missing() {
        let runs: Vec<VerificationRun> = vec![];
        let (missing, stale) = kind_state_sets(&runs, &["test", "build"], NOW);
        assert_eq!(missing, vec!["test", "build"]);
        assert!(stale.is_empty());
    }

    #[test]
    fn kind_state_sets_none_missing_fresh() {
        let runs = vec![
            make_run("test", "passed", &NOW.to_string()),
            make_run("build", "passed", &NOW.to_string()),
        ];
        let (missing, stale) = kind_state_sets(&runs, &["test", "build"], NOW);
        assert!(missing.is_empty());
        assert!(stale.is_empty());
    }

    #[test]
    fn kind_state_sets_stale_runs() {
        let old_ts = (NOW - 10 * 86400).to_string();
        let runs = vec![
            make_run("test", "passed", &old_ts),
        ];
        let (missing, stale) = kind_state_sets(&runs, &["test", "build"], NOW);
        assert_eq!(missing, vec!["build"]);
        assert_eq!(stale, vec!["test"]);
    }

    #[test]
    fn kind_state_sets_partial_kinds_present() {
        let runs = vec![make_run("test", "passed", &NOW.to_string())];
        let (missing, stale) = kind_state_sets(&runs, &["test"], NOW);
        assert!(missing.is_empty());
        assert!(stale.is_empty());
    }

    // ---- blocker_reasons ----

    #[test]
    fn blocker_reasons_no_blockers() {
        let reasons = blocker_reasons("cleanup", &[], &[], &[]);
        assert!(reasons.is_empty());
    }

    #[test]
    fn blocker_reasons_failing_kinds() {
        let reasons = blocker_reasons("cleanup", &[], &[], &["test"]);
        assert_eq!(reasons.len(), 1);
        assert!(reasons[0].contains("failing or uncertain"));
    }

    #[test]
    fn blocker_reasons_missing_test() {
        let reasons = blocker_reasons("cleanup", &["test"], &[], &[]);
        assert!(reasons.iter().any(|r| r.contains("Missing recorded test evidence")));
    }

    #[test]
    fn blocker_reasons_missing_build_for_refactor() {
        let reasons = blocker_reasons("refactor", &["test", "build"], &[], &[]);
        assert!(reasons.iter().any(|r| r.contains("Missing recorded test")));
        assert!(reasons.iter().any(|r| r.contains("Missing recorded build")));
    }

    #[test]
    fn blocker_reasons_missing_build_not_reported_for_cleanup() {
        let reasons = blocker_reasons("cleanup", &["build"], &[], &[]);
        // "build" is not "test", so the test-missing check won't fire.
        // And the refactor-specific build check won't fire for cleanup target.
        assert!(reasons.is_empty());
    }

    #[test]
    fn blocker_reasons_stale() {
        let reasons = blocker_reasons("cleanup", &[], &["test"], &[]);
        assert!(reasons.iter().any(|r| r.contains("stale")));
    }

    #[test]
    fn blocker_reasons_all_combined() {
        let reasons = blocker_reasons("refactor", &["test", "build"], &["lint"], &["lint"]);
        // failing + missing test + missing build + stale = 4
        assert_eq!(reasons.len(), 4);
    }

    // ---- gate_reasons ----

    #[test]
    fn gate_reasons_returns_blocker_reasons_when_blockers_exist() {
        let reasons = gate_reasons("cleanup", &["test"], &[], &[], &[], &[]);
        assert!(!reasons.is_empty());
        assert!(reasons.iter().any(|r| r.contains("Missing recorded test")));
    }

    #[test]
    fn gate_reasons_advisory_gaps_when_no_blockers() {
        let reasons = gate_reasons("cleanup", &[], &[], &["lint"], &[], &[]);
        assert_eq!(reasons.len(), 1);
        assert!(reasons[0].contains("Advisory verification evidence is incomplete"));
    }

    #[test]
    fn gate_reasons_empty_when_fully_satisfied() {
        let reasons = gate_reasons("cleanup", &[], &[], &[], &[], &[]);
        assert!(reasons.is_empty());
    }

    #[test]
    fn gate_reasons_advisory_stale_only() {
        let reasons = gate_reasons("cleanup", &[], &[], &[], &["lint"], &[]);
        assert_eq!(reasons.len(), 1);
        assert!(reasons[0].contains("Advisory verification evidence is incomplete"));
    }

    // ---- gate_next_steps ----

    #[test]
    fn gate_next_steps_failing_kinds() {
        let steps = gate_next_steps("cleanup", &[], &[], &[], &[], &["test"]);
        assert!(steps.iter().any(|s| s.contains("Stabilize failing")));
    }

    #[test]
    fn gate_next_steps_missing_test() {
        let steps = gate_next_steps("cleanup", &["test"], &[], &[], &[], &[]);
        assert!(steps.iter().any(|s| s.contains("Run and record project-native test")));
    }

    #[test]
    fn gate_next_steps_missing_build_for_refactor() {
        let steps = gate_next_steps("refactor", &["test", "build"], &[], &[], &[], &[]);
        assert!(steps.iter().any(|s| s.contains("Run and record project-native test")));
        assert!(steps.iter().any(|s| s.contains("Run and record project-native build")));
    }

    #[test]
    fn gate_next_steps_stale() {
        let steps = gate_next_steps("cleanup", &[], &["test"], &[], &[], &[]);
        assert!(steps.iter().any(|s| s.contains("Refresh stale")));
    }

    #[test]
    fn gate_next_steps_advisory_gaps_only() {
        let steps = gate_next_steps("cleanup", &[], &[], &["lint"], &[], &[]);
        assert!(steps.iter().any(|s| s.contains("Refresh advisory verification")));
    }

    #[test]
    fn gate_next_steps_fully_satisfied() {
        let steps = gate_next_steps("cleanup", &[], &[], &[], &[], &[]);
        assert_eq!(steps.len(), 1);
        assert!(steps[0].contains("supports the requested review mode"));
    }

    // ---- gate_assessment ----

    #[test]
    fn gate_assessment_blocked_when_no_runs() {
        let runs: Vec<VerificationRun> = vec![];
        let result = gate_assessment(&runs, "cleanup", NOW);
        assert_eq!(result["level"], "blocked");
        assert_eq!(result["allowed"], false);
        assert!(result["missing_kinds"].as_array().unwrap().len() > 0);
    }

    #[test]
    fn gate_assessment_blocked_when_failing() {
        let runs = vec![make_run("test", "failed", &NOW.to_string())];
        let result = gate_assessment(&runs, "cleanup", NOW);
        assert_eq!(result["level"], "blocked");
        assert_eq!(result["allowed"], false);
    }

    #[test]
    fn gate_assessment_caution_when_advisory_missing() {
        // cleanup: required=["test"], advisory=["lint","build"]
        // Provide test only
        let runs = vec![make_run("test", "passed", &NOW.to_string())];
        let result = gate_assessment(&runs, "cleanup", NOW);
        assert_eq!(result["level"], "caution");
        assert_eq!(result["allowed"], true);
    }

    #[test]
    fn gate_assessment_allow_when_all_present_and_fresh() {
        let runs = vec![
            make_run("test", "passed", &NOW.to_string()),
            make_run("lint", "passed", &NOW.to_string()),
            make_run("build", "passed", &NOW.to_string()),
        ];
        let result = gate_assessment(&runs, "cleanup", NOW);
        assert_eq!(result["level"], "allow");
        assert_eq!(result["allowed"], true);
        assert!(result["missing_kinds"].as_array().unwrap().is_empty());
    }

    #[test]
    fn gate_assessment_refactor_requires_build() {
        // refactor: required=["test","build"], advisory=["lint"]
        let runs = vec![make_run("test", "passed", &NOW.to_string())];
        let result = gate_assessment(&runs, "refactor", NOW);
        assert_eq!(result["level"], "blocked");
        assert_eq!(result["allowed"], false);
    }

    #[test]
    fn gate_assessment_refactor_allow_with_all() {
        let runs = vec![
            make_run("test", "passed", &NOW.to_string()),
            make_run("build", "passed", &NOW.to_string()),
            make_run("lint", "passed", &NOW.to_string()),
        ];
        let result = gate_assessment(&runs, "refactor", NOW);
        assert_eq!(result["level"], "allow");
        assert_eq!(result["allowed"], true);
    }

    #[test]
    fn gate_assessment_includes_freshness_policy() {
        let runs: Vec<VerificationRun> = vec![];
        let result = gate_assessment(&runs, "cleanup", NOW);
        assert!(result["freshness_policy"].is_object());
    }

    // ---- pipeline caution ----

    fn make_pipeline_run(kind: &str, status: &str, finished_at: &str) -> VerificationRun {
        VerificationRun {
            id: 1,
            kind: kind.to_string(),
            status: status.to_string(),
            command: "npx vue-tsc --noEmit 2>&1 | tail -30".to_string(),
            exit_code: Some(0),
            summary: Some(format!("{} summary", kind)),
            source: "test".to_string(),
            started_at: Some(finished_at.to_string()),
            finished_at: finished_at.to_string(),
        }
    }

    #[test]
    fn gate_assessment_caution_when_pipeline_passed() {
        let runs = vec![make_pipeline_run("test", "passed", &NOW.to_string())];
        let result = gate_assessment(&runs, "cleanup", NOW);
        assert_eq!(result["level"], "caution");
        assert_eq!(result["pipeline_caution_kinds"], json!(["test"]));
        assert!(result["reasons"].as_array().unwrap().iter().any(|r| r.as_str().unwrap().contains("pipeline")));
        assert!(result["next_steps"].as_array().unwrap().iter().any(|s| s.as_str().unwrap().contains("without pipes")));
    }

    #[test]
    fn gate_assessment_no_pipeline_caution_for_clean_commands() {
        let runs = vec![make_run("test", "passed", &NOW.to_string())];
        let result = gate_assessment(&runs, "cleanup", NOW);
        assert!(result["pipeline_caution_kinds"].as_array().unwrap().is_empty());
    }

    #[test]
    fn gate_assessment_pipeline_does_not_block() {
        let runs = vec![make_pipeline_run("test", "passed", &NOW.to_string())];
        let result = gate_assessment(&runs, "cleanup", NOW);
        assert_eq!(result["allowed"], true, "pipeline caution should not block");
    }

    #[test]
    fn verification_status_layer_includes_trust_level() {
        let ts = NOW.to_string();
        let runs = vec![make_pipeline_run("test", "passed", &ts)];
        let result = verification_status_layer(&runs);
        let latest = result["latest_runs"].as_array().unwrap();
        assert_eq!(latest[0]["trust_level"], "caution");
        assert_eq!(latest[0]["exit_code_masked_possible"], true);
    }

    #[test]
    fn verification_status_layer_trusted_for_clean_commands() {
        let ts = NOW.to_string();
        let runs = vec![make_run("test", "passed", &ts)];
        let result = verification_status_layer(&runs);
        let latest = result["latest_runs"].as_array().unwrap();
        assert_eq!(latest[0]["trust_level"], "trusted");
        assert_eq!(latest[0]["exit_code_masked_possible"], false);
    }

    // ---- gate_blockers ----

    #[test]
    fn gate_blockers_empty_when_all_good() {
        let runs = vec![
            make_run("test", "passed", &NOW.to_string()),
            make_run("lint", "passed", &NOW.to_string()),
            make_run("build", "passed", &NOW.to_string()),
        ];
        let blockers = gate_blockers(&runs, "cleanup", NOW);
        assert!(blockers.is_empty());
    }

    #[test]
    fn gate_blockers_present_when_missing_required() {
        let runs: Vec<VerificationRun> = vec![];
        let blockers = gate_blockers(&runs, "cleanup", NOW);
        assert!(!blockers.is_empty());
    }

    // ---- verification_has_failures ----

    #[test]
    fn verification_has_failures_empty() {
        let runs: Vec<VerificationRun> = vec![];
        assert!(!verification_has_failures(&runs));
    }

    #[test]
    fn verification_has_failures_all_passed() {
        let runs = vec![make_run("test", "passed", "1700000000")];
        assert!(!verification_has_failures(&runs));
    }

    #[test]
    fn verification_has_failures_with_failure() {
        let runs = vec![make_run("test", "failed", "1700000000")];
        assert!(verification_has_failures(&runs));
    }

    #[test]
    fn verification_has_failures_uncertain() {
        let runs = vec![make_run("build", "uncertain", "1700000000")];
        assert!(verification_has_failures(&runs));
    }

    // ---- verification_is_missing ----

    #[test]
    fn verification_is_missing_empty() {
        let runs: Vec<VerificationRun> = vec![];
        assert!(verification_is_missing(&runs));
    }

    #[test]
    fn verification_is_missing_with_runs() {
        let runs = vec![make_run("test", "passed", "1700000000")];
        assert!(!verification_is_missing(&runs));
    }

    // ---- project_gate_level ----

    #[test]
    fn project_gate_level_from_assessment() {
        let project = json!({
            "verification_evidence": {
                "gate_assessment": {
                    "cleanup": { "level": "allow" },
                    "refactor": { "level": "caution" },
                }
            }
        });
        assert_eq!(project_gate_level(&project, "cleanup"), "allow");
        assert_eq!(project_gate_level(&project, "refactor"), "caution");
    }

    #[test]
    fn project_gate_level_fallback_safe_for_cleanup() {
        let project = json!({
            "safe_for_cleanup": true,
        });
        assert_eq!(project_gate_level(&project, "cleanup"), "allow");
    }

    #[test]
    fn project_gate_level_fallback_safe_for_refactor() {
        let project = json!({
            "safe_for_refactor": true,
        });
        assert_eq!(project_gate_level(&project, "refactor"), "allow");
    }

    #[test]
    fn project_gate_level_fallback_blocked() {
        let project = json!({
            "safe_for_cleanup": false,
        });
        assert_eq!(project_gate_level(&project, "cleanup"), "blocked");
    }

    // ---- verification_status_layer ----

    #[test]
    fn verification_status_layer_empty_runs() {
        let runs: Vec<VerificationRun> = vec![];
        let result = verification_status_layer(&runs);
        assert_eq!(result["status"], "not_recorded");
        assert_eq!(result["missing_kinds"], json!(["test", "lint", "build"]));
        assert_eq!(result["all_expected_kinds_recorded"], false);
        assert_eq!(result["safe_for_cleanup"], false);
        assert_eq!(result["safe_for_refactor"], false);
        assert!(result["latest_runs"].as_array().unwrap().is_empty());
    }

    #[test]
    fn verification_status_layer_all_passing() {
        let ts = current_unix_secs().to_string();
        let runs = vec![
            make_run("test", "passed", &ts),
            make_run("lint", "passed", &ts),
            make_run("build", "passed", &ts),
        ];
        let result = verification_status_layer(&runs);
        assert_eq!(result["status"], "available");
        assert_eq!(result["all_expected_kinds_recorded"], true);
        assert_eq!(result["safe_for_cleanup"], true);
        assert_eq!(result["safe_for_refactor"], true);
        assert!(result["failing_runs"].as_array().unwrap().is_empty());
    }

    #[test]
    fn verification_status_layer_with_failures() {
        let ts = current_unix_secs().to_string();
        let runs = vec![
            make_run("test", "passed", &ts),
            make_run("lint", "failed", &ts),
        ];
        let result = verification_status_layer(&runs);
        assert_eq!(result["status"], "available");
        assert_eq!(result["safe_for_cleanup"], false);
        let failing = result["failing_runs"].as_array().unwrap();
        assert_eq!(failing.len(), 1);
        assert_eq!(failing[0]["kind"], "lint");
    }

    #[test]
    fn verification_status_layer_partial_kinds() {
        let ts = current_unix_secs().to_string();
        let runs = vec![make_run("test", "passed", &ts)];
        let result = verification_status_layer(&runs);
        assert_eq!(result["status"], "available");
        assert_eq!(result["all_expected_kinds_recorded"], false);
        let missing = result["missing_kinds"].as_array().unwrap();
        assert!(missing.iter().any(|k| k == "lint"));
        assert!(missing.iter().any(|k| k == "build"));
    }

    #[test]
    fn verification_status_layer_has_gate_assessment() {
        let runs: Vec<VerificationRun> = vec![];
        let result = verification_status_layer(&runs);
        assert!(result["gate_assessment"]["cleanup"].is_object());
        assert!(result["gate_assessment"]["refactor"].is_object());
    }

    #[test]
    fn verification_status_layer_has_freshness() {
        let ts = current_unix_secs().to_string();
        let runs = vec![make_run("test", "passed", &ts)];
        let result = verification_status_layer(&runs);
        assert!(result["freshness"].is_object());
        assert_eq!(result["freshness"]["label"], "verification");
    }

    #[test]
    fn verification_status_layer_stale_runs_not_safe() {
        // Use a timestamp far enough in the past to be stale (>7 days)
        let old_ts = (current_unix_secs() - 10 * 86400).to_string();
        let runs = vec![
            make_run("test", "passed", &old_ts),
        ];
        let result = verification_status_layer(&runs);
        // Stale required test => blocked for cleanup
        assert_eq!(result["safe_for_cleanup"], false);
    }

    // ---- workspace_verification_evidence_layer ----

    #[test]
    fn workspace_verification_evidence_layer_empty() {
        let result = workspace_verification_evidence_layer(&[], 0, 0);
        assert_eq!(result["status"], "available");
        assert_eq!(result["projects_with_recorded_verification"], 0);
        assert_eq!(result["projects_missing_verification"], 0);
        assert_eq!(result["confidence"], "low");
        assert!(result["blocking_projects"].as_array().unwrap().is_empty());
    }

    #[test]
    fn workspace_verification_evidence_layer_single_project_all_passing() {
        let project = json!({
            "project_id": "proj-a",
            "verification_evidence": {
                "status": "available",
                "failing_runs": [],
            },
            "observation": {
                "freshness": {
                    "verification": { "status": "fresh" }
                }
            },
            "safe_for_cleanup": true,
            "safe_for_refactor": true,
            "safe_for_cleanup_reason": "ok",
            "safe_for_refactor_reason": "ok",
        });
        let result = workspace_verification_evidence_layer(&[project], 1, 1);
        assert_eq!(result["projects_with_recorded_verification"], 1);
        assert_eq!(result["projects_missing_verification"], 0);
        assert_eq!(result["projects_with_failing_verification"], 0);
        assert_eq!(result["projects_safe_for_cleanup"], 1);
        assert_eq!(result["projects_safe_for_refactor"], 1);
        assert_eq!(result["confidence"], "high");
        assert!(result["blocking_projects"].as_array().unwrap().is_empty());
    }

    #[test]
    fn workspace_verification_evidence_layer_mixed_projects() {
        let passing = json!({
            "project_id": "proj-good",
            "verification_evidence": {
                "status": "available",
                "failing_runs": [],
            },
            "observation": {
                "freshness": {
                    "verification": { "status": "fresh" }
                }
            },
            "safe_for_cleanup": true,
            "safe_for_refactor": true,
            "safe_for_cleanup_reason": "ok",
            "safe_for_refactor_reason": "ok",
        });
        let failing = json!({
            "project_id": "proj-bad",
            "verification_evidence": {
                "status": "available",
                "failing_runs": [{"kind": "test", "status": "failed", "command": "make test"}],
            },
            "observation": {
                "freshness": {
                    "verification": { "status": "fresh" }
                }
            },
            "safe_for_cleanup": false,
            "safe_for_refactor": false,
            "safe_for_cleanup_reason": "blocked",
            "safe_for_refactor_reason": "blocked",
        });
        let result = workspace_verification_evidence_layer(&[passing, failing], 2, 1);
        assert_eq!(result["projects_with_recorded_verification"], 2);
        assert_eq!(result["projects_with_failing_verification"], 1);
        assert_eq!(result["projects_safe_for_cleanup"], 1);
        assert_eq!(result["blocking_projects"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn workspace_verification_evidence_layer_missing_verification() {
        let project = json!({
            "project_id": "proj-new",
            "verification_evidence": {
                "status": "not_recorded",
                "failing_runs": [],
            },
            "observation": {
                "freshness": {
                    "verification": { "status": "missing" }
                }
            },
            "safe_for_cleanup": false,
            "safe_for_refactor": false,
            "safe_for_cleanup_reason": "no evidence",
            "safe_for_refactor_reason": "no evidence",
        });
        let result = workspace_verification_evidence_layer(&[project], 1, 0);
        assert_eq!(result["projects_missing_verification"], 1);
        assert_eq!(result["confidence"], "low");
    }

    #[test]
    fn workspace_verification_evidence_layer_has_gate_distribution() {
        let project = json!({
            "project_id": "p1",
            "verification_evidence": {
                "status": "available",
                "gate_assessment": {
                    "cleanup": { "level": "allow" },
                    "refactor": { "level": "allow" },
                },
                "failing_runs": [],
            },
            "observation": {
                "freshness": { "verification": { "status": "fresh" } }
            },
            "safe_for_cleanup": true,
            "safe_for_refactor": true,
            "safe_for_cleanup_reason": "ok",
            "safe_for_refactor_reason": "ok",
        });
        let result = workspace_verification_evidence_layer(&[project], 1, 1);
        assert_eq!(result["cleanup_gate_distribution"]["allow"], 1);
        assert_eq!(result["refactor_gate_distribution"]["allow"], 1);
    }

    #[test]
    fn workspace_verification_evidence_layer_direct_observations_count() {
        let result = workspace_verification_evidence_layer(&[], 5, 2);
        let obs = result["direct_observations"].as_array().unwrap();
        assert!(obs.iter().any(|o| o.as_str().unwrap().contains("Registered projects: 5")));
        assert!(obs.iter().any(|o| o.as_str().unwrap().contains("monitoring: 2")));
    }

    #[test]
    fn workspace_verification_evidence_layer_verified_conclusions() {
        let project = json!({
            "project_id": "p1",
            "verification_evidence": {
                "status": "available",
                "failing_runs": [],
            },
            "observation": {
                "freshness": { "verification": { "status": "fresh" } }
            },
            "safe_for_cleanup": true,
            "safe_for_refactor": true,
            "safe_for_cleanup_reason": "ok",
            "safe_for_refactor_reason": "ok",
        });
        let result = workspace_verification_evidence_layer(&[project], 1, 1);
        let vc = result["verified_conclusions"].as_array().unwrap();
        assert!(vc.iter().any(|c| c["summary"].as_str().unwrap().contains("1 project(s)")));
    }

    #[test]
    fn workspace_verification_evidence_layer_unverified_conclusions() {
        let project = json!({
            "project_id": "p1",
            "verification_evidence": {
                "status": "not_recorded",
                "failing_runs": [],
            },
            "observation": {
                "freshness": { "verification": { "status": "missing" } }
            },
            "safe_for_cleanup": false,
            "safe_for_refactor": false,
            "safe_for_cleanup_reason": "no evidence",
            "safe_for_refactor_reason": "no evidence",
        });
        let result = workspace_verification_evidence_layer(&[project], 1, 0);
        let uc = result["unverified_conclusions"].as_array().unwrap();
        assert!(uc.iter().any(|c| c["summary"].as_str().unwrap().contains("missing verification")));
    }
}
