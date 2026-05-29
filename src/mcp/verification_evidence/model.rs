mod gate;
mod status;
mod workspace;

#[cfg(test)]
pub(super) use gate::{
    blocker_reasons, failing_kinds, gate_assessment, gate_kinds, gate_next_steps, gate_reasons,
    kind_state_sets,
};
pub(super) use gate::{
    gate_blockers, pipeline_caution_kinds, project_gate_level, suspicious_summary_kinds,
    suspicious_summary_signals, VerificationGateAssessment, VerificationGateTarget,
};
pub(super) use status::{GateDistribution, VerificationStatusSummary};
pub(super) use workspace::VerificationEvidenceWorkspaceSummary;

const EXPECTED_KINDS: [&str; 3] = ["test", "lint", "build"];

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
