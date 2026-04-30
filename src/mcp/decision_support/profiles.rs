use serde_json::{json, Value};

pub(in crate::mcp) fn decision_action_profile(action: &str, strategy_mode: &str) -> Value {
    let (action_class, phase, mutability, verification_required, primary_goal) = match action {
        "review_failing_verification" => (
            "verification_recovery",
            "stabilize",
            "read_mostly",
            true,
            "stabilize failing or uncertain evidence before broader edits",
        ),
        "stabilize_repository_state" => (
            "repository_stabilization",
            "stabilize",
            "read_mostly",
            true,
            "resolve in-progress repository state before broader changes",
        ),
        "start_monitor" | "take_snapshot" | "generate_activity_then_stats" => (
            "evidence_collection",
            "observe",
            "non_code_state_change",
            false,
            "collect missing activity or inventory evidence",
        ),
        "run_verification_before_high_risk_changes" => (
            "verification_collection",
            "verify",
            "read_mostly",
            true,
            "record test/lint/build evidence before risky work",
        ),
        "review_unused_files" => (
            "cleanup_review",
            "review",
            "review_before_modify",
            false,
            "inspect unused-file candidates before cleanup",
        ),
        "inspect_hot_files" => (
            "refactor_review",
            "review",
            "review_before_modify",
            false,
            "inspect activity hotspots before targeted refactor",
        ),
        _ => (
            "workspace_triage",
            "triage",
            "read_only",
            false,
            "choose the next project or tool path",
        ),
    };

    json!({
        "action_class": action_class,
        "phase": phase,
        "mutability_scope": mutability,
        "verification_required": verification_required,
        "strategy_mode": strategy_mode,
        "primary_goal": primary_goal,
    })
}

pub(in crate::mcp) fn decision_risk_profile(
    action: &str,
    matched_overview: &Value,
    verification_status: &str,
    safe_for_cleanup: Option<bool>,
    safe_for_refactor: Option<bool>,
) -> Value {
    let repo_risk = &matched_overview["repo_status_risk"];
    let repo_risk_level = repo_risk["risk_level"].as_str().unwrap_or("unknown");
    let cleanup_gate_level = matched_overview["verification_evidence"]["gate_assessment"]
        ["cleanup"]["level"]
        .as_str()
        .unwrap_or(if safe_for_cleanup.unwrap_or(false) {
            "allow"
        } else {
            "blocked"
        });
    let refactor_gate_level = matched_overview["verification_evidence"]["gate_assessment"]
        ["refactor"]["level"]
        .as_str()
        .unwrap_or(if safe_for_refactor.unwrap_or(false) {
            "allow"
        } else {
            "blocked"
        });
    let cleanup_blockers = matched_overview["cleanup_blockers"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let refactor_blockers = matched_overview["refactor_blockers"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let primary_repo_risk_finding = repo_risk["highest_priority_finding"].clone();
    let repo_risk_findings = repo_risk["risk_findings"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let risk_tier = match action {
        "review_failing_verification" | "stabilize_repository_state" => "high",
        "run_verification_before_high_risk_changes" => "medium",
        "review_unused_files" | "inspect_hot_files" => {
            let gate_is_cautious = match action {
                "review_unused_files" => cleanup_gate_level != "allow",
                _ => refactor_gate_level != "allow",
            };
            if repo_risk_level == "high" || verification_status != "available" || gate_is_cautious {
                "medium"
            } else {
                "low"
            }
        }
        _ => "low",
    };

    json!({
        "risk_tier": risk_tier,
        "repo_risk_level": repo_risk_level,
        "verification_status": verification_status,
        "cleanup_ready": safe_for_cleanup,
        "refactor_ready": safe_for_refactor,
        "cleanup_gate_level": cleanup_gate_level,
        "refactor_gate_level": refactor_gate_level,
        "cleanup_blockers": cleanup_blockers,
        "refactor_blockers": refactor_blockers,
        "primary_repo_risk_finding": primary_repo_risk_finding,
        "repo_risk_findings": repo_risk_findings,
        "repo_risk_finding_counts": repo_risk["finding_counts"].clone(),
        "destructive_change_recommended": false,
        "manual_review_required": action == "review_unused_files" || action == "inspect_hot_files",
    })
}
