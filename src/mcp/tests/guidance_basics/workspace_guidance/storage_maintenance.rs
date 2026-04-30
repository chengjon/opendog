use super::*;

#[test]
fn agent_guidance_reports_storage_maintenance_candidates() {
    let value = agent_guidance_payload(
        3,
        2,
        &["demo".to_string(), "other".to_string()],
        &["demo".to_string()],
        &[json!({
            "project_id": "demo",
            "recommended_next_action": "inspect_hot_files",
            "reason": "Activity exists.",
            "confidence": "medium"
        })],
        &[json!({
            "project_id": "demo",
            "safe_for_cleanup": true,
            "safe_for_refactor": true,
            "verification_evidence": {
                "status": "available",
                "failing_runs": []
            },
            "repo_status_risk": {
                "status": "available",
                "risk_level": "low",
                "is_dirty": false,
                "operation_states": []
            },
            "mock_data_summary": {
                "hardcoded_candidate_count": 0,
                "mock_candidate_count": 0
            },
            "storage_maintenance": {
                "maintenance_candidate": true,
                "vacuum_candidate": true,
                "approx_db_size_bytes": 8192,
                "approx_reclaimable_bytes": 4096,
                "reclaim_ratio": 0.5,
                "suggested_mode": "cleanup_and_vacuum"
            },
            "project_toolchain": {
                "project_type": "rust",
                "recommended_test_commands": ["cargo test"],
                "recommended_lint_commands": [],
                "recommended_build_commands": ["cargo check"]
            },
            "observation": {
                "coverage_state": "ready",
                "freshness": {
                    "snapshot": { "status": "fresh" },
                    "activity": { "status": "fresh" },
                    "verification": { "status": "fresh" }
                }
            }
        })],
    );

    assert_eq!(
        value["guidance"]["layers"]["storage_maintenance"]["projects_with_candidates"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["storage_maintenance"]["projects_with_vacuum_candidates"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["storage_maintenance"]["priority_projects"][0]["project_id"],
        json!("demo")
    );
    assert_eq!(
        value["guidance"]["layers"]["storage_maintenance"]["priority_projects"][0]
            ["suggested_mode"],
        json!("cleanup_and_vacuum")
    );
}
