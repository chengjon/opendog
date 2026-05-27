#[cfg(test)]
use serde_json::json;
use serde_json::Value;

mod model;

use model::{DecisionActionProfile, DecisionRiskProfile};

pub(in crate::mcp) fn decision_action_profile(action: &str, strategy_mode: &str) -> Value {
    DecisionActionProfile::from_action(action, strategy_mode).to_json()
}

pub(in crate::mcp) fn decision_risk_profile(
    action: &str,
    matched_overview: &Value,
    verification_status: &str,
    safe_for_cleanup: Option<bool>,
    safe_for_refactor: Option<bool>,
) -> Value {
    DecisionRiskProfile::from_overview(
        action,
        matched_overview,
        verification_status,
        safe_for_cleanup,
        safe_for_refactor,
    )
    .to_json()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── decision_action_profile ──────────────────────────────────────

    #[test]
    fn action_profile_review_failing_verification() {
        let p = decision_action_profile("review_failing_verification", "conservative");
        assert_eq!(p["action_class"], "verification_recovery");
        assert_eq!(p["phase"], "stabilize");
        assert_eq!(p["mutability_scope"], "read_mostly");
        assert_eq!(p["verification_required"], true);
        assert_eq!(p["strategy_mode"], "conservative");
        assert!(p["primary_goal"].as_str().unwrap().contains("failing"));
    }

    #[test]
    fn action_profile_stabilize_repository_state() {
        let p = decision_action_profile("stabilize_repository_state", "aggressive");
        assert_eq!(p["action_class"], "repository_stabilization");
        assert_eq!(p["phase"], "stabilize");
        assert_eq!(p["mutability_scope"], "read_mostly");
        assert_eq!(p["verification_required"], true);
        assert_eq!(p["strategy_mode"], "aggressive");
    }

    #[test]
    fn action_profile_evidence_collection_group() {
        for action in &[
            "start_monitor",
            "take_snapshot",
            "generate_activity_then_stats",
        ] {
            let p = decision_action_profile(action, "standard");
            assert_eq!(
                p["action_class"], "evidence_collection",
                "failed for {}",
                action
            );
            assert_eq!(p["phase"], "observe", "failed for {}", action);
            assert_eq!(
                p["mutability_scope"], "non_code_state_change",
                "failed for {}",
                action
            );
            assert_eq!(p["verification_required"], false, "failed for {}", action);
        }
    }

    #[test]
    fn action_profile_run_verification_before_high_risk() {
        let p = decision_action_profile("run_verification_before_high_risk_changes", "standard");
        assert_eq!(p["action_class"], "verification_collection");
        assert_eq!(p["phase"], "verify");
        assert_eq!(p["mutability_scope"], "read_mostly");
        assert_eq!(p["verification_required"], true);
    }

    #[test]
    fn action_profile_review_unused_files() {
        let p = decision_action_profile("review_unused_files", "standard");
        assert_eq!(p["action_class"], "cleanup_review");
        assert_eq!(p["phase"], "review");
        assert_eq!(p["mutability_scope"], "review_before_modify");
        assert_eq!(p["verification_required"], false);
    }

    #[test]
    fn action_profile_inspect_hot_files() {
        let p = decision_action_profile("inspect_hot_files", "standard");
        assert_eq!(p["action_class"], "refactor_review");
        assert_eq!(p["phase"], "review");
        assert_eq!(p["mutability_scope"], "review_before_modify");
        assert_eq!(p["verification_required"], false);
    }

    #[test]
    fn action_profile_unknown_action_falls_back_to_triage() {
        let p = decision_action_profile("nonexistent_action", "standard");
        assert_eq!(p["action_class"], "workspace_triage");
        assert_eq!(p["phase"], "triage");
        assert_eq!(p["mutability_scope"], "read_only");
        assert_eq!(p["verification_required"], false);
    }

    #[test]
    fn action_profile_strategy_mode_echoed() {
        let p = decision_action_profile("start_monitor", "yolo_mode");
        assert_eq!(p["strategy_mode"], "yolo_mode");
    }

    // ── decision_risk_profile ────────────────────────────────────────

    fn minimal_overview() -> Value {
        json!({
            "repo_status_risk": {
                "risk_level": "low",
                "highest_priority_finding": null,
                "risk_findings": [],
                "finding_counts": {}
            },
            "verification_evidence": {
                "gate_assessment": {
                    "cleanup": { "level": "allow" },
                    "refactor": { "level": "allow" }
                }
            },
            "cleanup_blockers": [],
            "refactor_blockers": []
        })
    }

    #[test]
    fn risk_profile_high_risk_actions() {
        for action in &["review_failing_verification", "stabilize_repository_state"] {
            let r = decision_risk_profile(
                action,
                &minimal_overview(),
                "available",
                Some(true),
                Some(true),
            );
            assert_eq!(r["risk_tier"], "high", "expected high tier for {}", action);
        }
    }

    #[test]
    fn risk_profile_medium_risk_verification_collection() {
        let r = decision_risk_profile(
            "run_verification_before_high_risk_changes",
            &minimal_overview(),
            "available",
            Some(true),
            Some(true),
        );
        assert_eq!(r["risk_tier"], "medium");
    }

    #[test]
    fn risk_profile_review_unused_low_when_safe() {
        let r = decision_risk_profile(
            "review_unused_files",
            &minimal_overview(),
            "available",
            Some(true),
            Some(true),
        );
        assert_eq!(r["risk_tier"], "low");
    }

    #[test]
    fn risk_profile_review_unused_medium_when_repo_risk_high() {
        let mut overview = minimal_overview();
        overview["repo_status_risk"]["risk_level"] = json!("high");
        let r = decision_risk_profile(
            "review_unused_files",
            &overview,
            "available",
            Some(true),
            Some(true),
        );
        assert_eq!(r["risk_tier"], "medium");
    }

    #[test]
    fn risk_profile_review_unused_medium_when_verification_missing() {
        let r = decision_risk_profile(
            "review_unused_files",
            &minimal_overview(),
            "missing",
            Some(true),
            Some(true),
        );
        assert_eq!(r["risk_tier"], "medium");
    }

    #[test]
    fn risk_profile_review_unused_medium_when_cleanup_gate_blocked() {
        let mut overview = minimal_overview();
        overview["verification_evidence"]["gate_assessment"]["cleanup"]["level"] = json!("blocked");
        let r = decision_risk_profile(
            "review_unused_files",
            &overview,
            "available",
            Some(true),
            Some(true),
        );
        assert_eq!(r["risk_tier"], "medium");
    }

    #[test]
    fn risk_profile_inspect_hot_low_when_safe() {
        let r = decision_risk_profile(
            "inspect_hot_files",
            &minimal_overview(),
            "available",
            Some(true),
            Some(true),
        );
        assert_eq!(r["risk_tier"], "low");
    }

    #[test]
    fn risk_profile_inspect_hot_medium_when_refactor_gate_blocked() {
        let mut overview = minimal_overview();
        overview["verification_evidence"]["gate_assessment"]["refactor"]["level"] =
            json!("blocked");
        let r = decision_risk_profile(
            "inspect_hot_files",
            &overview,
            "available",
            Some(true),
            Some(true),
        );
        assert_eq!(r["risk_tier"], "medium");
    }

    #[test]
    fn risk_profile_unknown_action_is_low() {
        let r = decision_risk_profile(
            "something_else",
            &minimal_overview(),
            "available",
            None,
            None,
        );
        assert_eq!(r["risk_tier"], "low");
    }

    #[test]
    fn risk_profile_cleanup_ready_refactor_ready_echoed() {
        let r = decision_risk_profile(
            "inspect_hot_files",
            &minimal_overview(),
            "available",
            Some(true),
            Some(false),
        );
        assert_eq!(r["cleanup_ready"], true);
        assert_eq!(r["refactor_ready"], false);
    }

    #[test]
    fn risk_profile_manual_review_required_for_cleanup_and_refactor() {
        for action in &["review_unused_files", "inspect_hot_files"] {
            let r = decision_risk_profile(action, &minimal_overview(), "available", None, None);
            assert_eq!(
                r["manual_review_required"], true,
                "expected manual review for {}",
                action
            );
        }
    }

    #[test]
    fn risk_profile_no_manual_review_for_other_actions() {
        for action in &["review_failing_verification", "start_monitor", "unknown"] {
            let r = decision_risk_profile(action, &minimal_overview(), "available", None, None);
            assert_eq!(
                r["manual_review_required"], false,
                "expected no manual review for {}",
                action
            );
        }
    }

    #[test]
    fn risk_profile_destructive_change_always_false() {
        for action in &[
            "review_failing_verification",
            "review_unused_files",
            "inspect_hot_files",
            "unknown",
        ] {
            let r = decision_risk_profile(action, &minimal_overview(), "available", None, None);
            assert_eq!(r["destructive_change_recommended"], false);
        }
    }

    #[test]
    fn risk_profile_gate_fallback_when_overview_missing() {
        let empty = json!({});
        let r = decision_risk_profile(
            "review_unused_files",
            &empty,
            "available",
            Some(false),
            Some(false),
        );
        assert_eq!(r["cleanup_gate_level"], "blocked");
        assert_eq!(r["refactor_gate_level"], "blocked");
    }

    #[test]
    fn risk_profile_blockers_from_overview() {
        let mut overview = minimal_overview();
        overview["cleanup_blockers"] = json!(["stale snapshot"]);
        overview["refactor_blockers"] = json!(["failing tests", "no lint evidence"]);
        let r = decision_risk_profile(
            "review_unused_files",
            &overview,
            "available",
            Some(true),
            Some(true),
        );
        let cb = r["cleanup_blockers"].as_array().unwrap();
        assert_eq!(cb.len(), 1);
        assert_eq!(cb[0], "stale snapshot");
        let rb = r["refactor_blockers"].as_array().unwrap();
        assert_eq!(rb.len(), 2);
    }

    #[test]
    fn risk_profile_repo_risk_findings_propagated() {
        let mut overview = minimal_overview();
        overview["repo_status_risk"]["highest_priority_finding"] = json!("uncommitted changes");
        overview["repo_status_risk"]["risk_findings"] =
            json!(["uncommitted changes", "dirty index"]);
        overview["repo_status_risk"]["finding_counts"] = json!({"high": 1, "medium": 1});
        let r = decision_risk_profile(
            "stabilize_repository_state",
            &overview,
            "available",
            None,
            None,
        );
        assert_eq!(r["primary_repo_risk_finding"], "uncommitted changes");
        let findings = r["repo_risk_findings"].as_array().unwrap();
        assert_eq!(findings.len(), 2);
        assert_eq!(r["repo_risk_finding_counts"]["high"], 1);
    }
}
