use super::*;
use serde_json::json;

fn minimal_overview() -> Value {
    json!({
        "recommended_next_action": "inspect_hot_files",
        "repo_status_risk": {
            "risk_level": "low",
            "operation_states": [],
            "is_dirty": false,
        },
        "verification_evidence": {
            "status": "recorded",
            "failing_runs": [],
        },
        "observation": {
            "freshness": {
                "snapshot": { "status": "fresh" },
                "activity": { "status": "fresh" },
                "verification": { "status": "fresh" },
            },
            "coverage_state": "active",
        },
        "mock_data_summary": {
            "hardcoded_candidate_count": 0,
            "mock_candidate_count": 0,
        },
        "safe_for_cleanup": true,
        "safe_for_refactor": true,
    })
}

#[path = "tests/batches.rs"]
mod batches;
#[path = "tests/portfolio_layer.rs"]
mod portfolio_layer;
#[path = "tests/project_summary.rs"]
mod project_summary;
#[path = "tests/recommendation_sorting.rs"]
mod recommendation_sorting;
#[path = "tests/scoring.rs"]
mod scoring;
