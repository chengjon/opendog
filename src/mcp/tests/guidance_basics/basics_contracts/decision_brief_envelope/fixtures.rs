use super::*;

pub(super) fn demo_project_overview() -> serde_json::Value {
    json!({
        "project_id": "demo",
        "status": "monitoring",
        "recommended_next_action": "review_failing_verification",
        "recommended_reason": "Test evidence is failing.",
        "strategy_confidence": "high",
        "safe_for_cleanup": false,
        "safe_for_refactor": false,
        "cleanup_blockers": ["verification evidence is failing"],
        "refactor_blockers": ["verification evidence is failing"],
        "verification_gate_levels": {
            "cleanup": "blocked",
            "refactor": "blocked"
        },
        "observation": {
            "coverage_state": "ready",
            "freshness": {
                "snapshot": { "status": "fresh" },
                "activity": { "status": "fresh" },
                "verification": { "status": "fresh" }
            }
        },
        "repo_status_risk": {
            "status": "available",
            "risk_level": "medium",
            "is_dirty": false,
            "operation_states": [],
            "risk_findings": [],
            "finding_counts": {
                "total": 0,
                "high": 0,
                "medium": 0,
                "low": 0
            },
            "highest_priority_finding": null
        },
        "verification_evidence": {
            "status": "available",
            "failing_runs": [{"kind":"test","status":"failed"}],
            "gate_assessment": {
                "cleanup": { "level": "blocked" },
                "refactor": { "level": "blocked" }
            }
        },
        "project_toolchain": {
            "project_type": "rust",
            "recommended_test_commands": ["cargo test"],
            "recommended_lint_commands": ["cargo clippy --all-targets --all-features -- -D warnings"],
            "recommended_build_commands": ["cargo check"]
        },
        "storage_maintenance": {
            "status": "available",
            "maintenance_candidate": true,
            "vacuum_candidate": false,
            "approx_db_size_bytes": 4096,
            "approx_reclaimable_bytes": 2048,
            "reclaim_ratio": 0.35
        },
        "mock_data_summary": {
            "hardcoded_candidate_count": 1,
            "mock_candidate_count": 2,
            "mixed_review_file_count": 1
        }
    })
}

pub(super) fn demo_recommendation() -> serde_json::Value {
    json!({
        "project_id": "demo",
        "recommended_next_action": "review_failing_verification",
        "reason": "Test evidence is failing.",
        "confidence": "high"
    })
}

pub(super) fn demo_workspace_data_guidance() -> serde_json::Value {
    workspace_data_risk_overview_payload(
        &[json!({
            "project_id": "demo",
            "status": "monitoring",
            "hardcoded_candidate_count": 1,
            "mock_candidate_count": 2,
            "mixed_review_file_count": 1,
            "recommended_next_action": "review_failing_verification",
            "reason": "Test evidence is failing.",
            "confidence": "high",
            "rule_groups_summary": [
                {"group": "content", "severity": "high", "count": 1}
            ],
            "rule_hits_summary": [
                {
                    "rule": "content.business_literal_combo",
                    "group": "content",
                    "severity": "high",
                    "description": "business-like literals",
                    "count": 1
                }
            ]
        })],
        1,
    )
}
