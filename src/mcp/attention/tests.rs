use super::*;
use serde_json::json;

// --- attention_action_base ---

#[test]
fn attention_action_base_known_actions() {
    assert_eq!(attention_action_base("stabilize_repository_state"), 100);
    assert_eq!(attention_action_base("review_failing_verification"), 95);
    assert_eq!(
        attention_action_base("run_verification_before_high_risk_changes"),
        75
    );
    assert_eq!(attention_action_base("take_snapshot"), 65);
    assert_eq!(attention_action_base("start_monitor"), 60);
    assert_eq!(attention_action_base("generate_activity_then_stats"), 55);
    assert_eq!(attention_action_base("review_unused_files"), 40);
    assert_eq!(attention_action_base("inspect_hot_files"), 30);
}

#[test]
fn attention_action_base_unknown_action_returns_default() {
    assert_eq!(attention_action_base("nonexistent_action"), 20);
    assert_eq!(attention_action_base(""), 20);
    assert_eq!(attention_action_base("inspect_workspace_state"), 20);
}

// --- attention_band ---

#[test]
fn attention_band_critical() {
    assert_eq!(attention_band(120), "critical");
    assert_eq!(attention_band(200), "critical");
    assert_eq!(attention_band(i64::MAX), "critical");
}

#[test]
fn attention_band_high() {
    assert_eq!(attention_band(80), "high");
    assert_eq!(attention_band(100), "high");
    assert_eq!(attention_band(119), "high");
}

#[test]
fn attention_band_medium() {
    assert_eq!(attention_band(45), "medium");
    assert_eq!(attention_band(60), "medium");
    assert_eq!(attention_band(79), "medium");
}

#[test]
fn attention_band_low() {
    assert_eq!(attention_band(0), "low");
    assert_eq!(attention_band(44), "low");
    assert_eq!(attention_band(-10), "low");
    assert_eq!(attention_band(i64::MIN), "low");
}

// --- freshness_attention_score ---

#[test]
fn freshness_attention_score_missing() {
    assert_eq!(freshness_attention_score("missing", 14, 9), 14);
    assert_eq!(freshness_attention_score("missing", 20, 10), 20);
}

#[test]
fn freshness_attention_score_stale_and_unknown() {
    assert_eq!(freshness_attention_score("stale", 14, 9), 9);
    assert_eq!(freshness_attention_score("unknown", 14, 9), 9);
}

#[test]
fn freshness_attention_score_fresh_returns_zero() {
    assert_eq!(freshness_attention_score("fresh", 14, 9), 0);
    assert_eq!(freshness_attention_score("anything_else", 14, 9), 0);
    assert_eq!(freshness_attention_score("", 14, 9), 0);
}

// --- repo_risk_priority ---

#[test]
fn repo_risk_priority_all_levels() {
    assert_eq!(repo_risk_priority("high"), 3);
    assert_eq!(repo_risk_priority("medium"), 2);
    assert_eq!(repo_risk_priority("low"), 1);
    assert_eq!(repo_risk_priority("unknown"), 0);
    assert_eq!(repo_risk_priority(""), 0);
}

// --- confidence_priority ---

#[test]
fn confidence_priority_all_levels() {
    assert_eq!(confidence_priority("high"), 3);
    assert_eq!(confidence_priority("medium"), 2);
    assert_eq!(confidence_priority("low"), 1);
    assert_eq!(confidence_priority("unknown"), 0);
    assert_eq!(confidence_priority(""), 0);
}

// --- attention_batches_from_queue ---

#[test]
fn attention_batches_empty_queue() {
    let result = attention_batches_from_queue(&[], 3, "available");
    assert_eq!(result["status"], "available");
    assert_eq!(result["source"], "attention_queue");
    assert_eq!(result["batched_project_count"], 0);
    assert_eq!(result["unbatched_project_count"], 3);
    assert_eq!(result["immediate"].as_array().unwrap().len(), 0);
    assert_eq!(result["next"].as_array().unwrap().len(), 0);
    assert_eq!(result["later"].as_array().unwrap().len(), 0);
}

#[test]
fn attention_batches_single_item_goes_to_immediate() {
    let queue = vec![json!({
        "project_id": "proj1",
        "recommended_next_action": "take_snapshot",
        "attention_score": 80,
        "attention_band": "high",
    })];
    let result = attention_batches_from_queue(&queue, 1, "available");
    let immediate = result["immediate"].as_array().unwrap();
    let next = result["next"].as_array().unwrap();
    let later = result["later"].as_array().unwrap();
    assert_eq!(immediate.len(), 1);
    assert_eq!(immediate[0]["project_id"], "proj1");
    assert_eq!(next.len(), 0);
    assert_eq!(later.len(), 0);
}

#[test]
fn attention_batches_splits_into_immediate_next_later() {
    let queue: Vec<Value> = (0..6)
        .map(|i| {
            json!({
                "project_id": format!("proj{i}"),
                "recommended_next_action": "inspect_hot_files",
                "attention_score": 100 - i as i64 * 10,
                "attention_band": "high",
            })
        })
        .collect();
    let result = attention_batches_from_queue(&queue, 10, "available");
    let immediate = result["immediate"].as_array().unwrap();
    let next = result["next"].as_array().unwrap();
    let later = result["later"].as_array().unwrap();
    // immediate = first 1
    assert_eq!(immediate.len(), 1);
    assert_eq!(immediate[0]["project_id"], "proj0");
    // next = skip 1, take 2
    assert_eq!(next.len(), 2);
    assert_eq!(next[0]["project_id"], "proj1");
    assert_eq!(next[1]["project_id"], "proj2");
    // later = skip 3, rest
    assert_eq!(later.len(), 3);
    assert_eq!(later[0]["project_id"], "proj3");
    // unbatched = total(10) - queue(6) = 4
    assert_eq!(result["unbatched_project_count"], 4);
}

#[test]
fn attention_batches_thin_entries_only_have_four_fields() {
    let queue = vec![json!({
        "project_id": "proj1",
        "recommended_next_action": "take_snapshot",
        "attention_score": 80,
        "attention_band": "high",
        "extra_field": "should_not_appear",
    })];
    let result = attention_batches_from_queue(&queue, 1, "available");
    let entry = &result["immediate"].as_array().unwrap()[0];
    // Only 4 thin fields should be present
    assert!(entry.get("extra_field").is_none());
    assert!(entry.get("project_id").is_some());
    assert!(entry.get("recommended_next_action").is_some());
    assert!(entry.get("attention_score").is_some());
    assert!(entry.get("attention_band").is_some());
}

// --- project_attention_summary ---

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

#[test]
fn project_attention_summary_minimal_clean_state() {
    let overview = minimal_overview();
    let summary = project_attention_summary(&overview);
    // Base for "inspect_hot_files" = 30, repo_risk low = +0, all fresh = +0
    // hardcoded=0, mock=0, safe_cleanup=true, safe_refactor=true => +0
    assert_eq!(summary.attention_score, 30);
    assert_eq!(summary.attention_band, "low");
    assert_eq!(summary.evidence_quality, "ready");
    assert_eq!(
        summary.priority_basis.recommended_next_action,
        "inspect_hot_files"
    );
    assert_eq!(summary.priority_basis.recommended_action_base, 30);
    assert_eq!(summary.priority_basis.repo_risk_level, "low");
    assert!(!summary.priority_basis.repo_in_operation);
    assert!(!summary.priority_basis.repo_is_dirty);
    assert!(summary.priority_basis.safe_for_cleanup);
    assert!(summary.priority_basis.safe_for_refactor);
    // Routine reason
    assert_eq!(
        summary.attention_reasons,
        vec!["Current evidence supports routine review sequencing.".to_string()]
    );
}

#[test]
fn project_attention_summary_high_repo_risk_and_dirty() {
    let overview = json!({
        "recommended_next_action": "stabilize_repository_state",
        "repo_status_risk": {
            "risk_level": "high",
            "operation_states": [],
            "is_dirty": true,
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
    });
    let summary = project_attention_summary(&overview);
    // Base 100 + high_risk 18 + dirty 6 = 124
    assert_eq!(summary.attention_score, 124);
    assert_eq!(summary.attention_band, "critical");
    // evidence_quality is "ready" since no blockers
    assert_eq!(summary.evidence_quality, "ready");
}

#[test]
fn project_attention_summary_repo_in_operation_adds_30() {
    let mut overview = minimal_overview();
    overview["repo_status_risk"]["operation_states"] = json!(["merge"]);
    let summary = project_attention_summary(&overview);
    // Base 30 + operation 30 = 60
    assert_eq!(summary.attention_score, 60);
    assert!(summary
        .attention_reasons
        .iter()
        .any(|r| r.contains("mid-operation")));
    assert_eq!(summary.evidence_quality, "blocked");
}

#[test]
fn project_attention_summary_failing_verification_adds_25() {
    let mut overview = minimal_overview();
    overview["verification_evidence"]["failing_runs"] =
        json!([{"command": "cargo test", "status": "failed"}]);
    let summary = project_attention_summary(&overview);
    // Base 30 + failing 25 = 55
    assert_eq!(summary.attention_score, 55);
    assert!(summary
        .attention_reasons
        .iter()
        .any(|r| r.contains("failing")));
    assert_eq!(summary.evidence_quality, "blocked");
}

#[test]
fn project_attention_summary_missing_snapshot_adds_14() {
    let mut overview = minimal_overview();
    overview["observation"]["freshness"]["snapshot"]["status"] = json!("missing");
    let summary = project_attention_summary(&overview);
    // Base 30 + missing_snapshot 14 = 44
    assert_eq!(summary.attention_score, 44);
    assert!(summary
        .attention_reasons
        .iter()
        .any(|r| r.contains("Snapshot baseline is missing")));
}

#[test]
fn project_attention_summary_stale_activity_adds_8() {
    let mut overview = minimal_overview();
    overview["observation"]["freshness"]["activity"]["status"] = json!("stale");
    let summary = project_attention_summary(&overview);
    // Base 30 + stale_activity 8 = 38
    assert_eq!(summary.attention_score, 38);
    assert!(summary
        .attention_reasons
        .iter()
        .any(|r| r.contains("Activity evidence is stale")));
}

#[test]
fn project_attention_summary_not_recorded_verification_adds_18_plus_freshness_18() {
    let mut overview = minimal_overview();
    overview["verification_evidence"]["status"] = json!("not_recorded");
    overview["observation"]["freshness"]["verification"]["status"] = json!("missing");
    let summary = project_attention_summary(&overview);
    // Base 30 + not_recorded 18 + missing_verification_freshness 18 = 66
    assert_eq!(summary.attention_score, 66);
}

#[test]
fn project_attention_summary_hardcoded_candidates_capped_at_3() {
    let mut overview = minimal_overview();
    overview["mock_data_summary"]["hardcoded_candidate_count"] = json!(10);
    let summary = project_attention_summary(&overview);
    // Base 30 + min(10,3)*5 = 30 + 15 = 45
    assert_eq!(summary.attention_score, 45);
    assert!(summary
        .attention_reasons
        .iter()
        .any(|r| r.contains("Hardcoded-data candidates")));
}

#[test]
fn project_attention_summary_mock_candidates_capped_at_3() {
    let mut overview = minimal_overview();
    overview["mock_data_summary"]["mock_candidate_count"] = json!(5);
    let summary = project_attention_summary(&overview);
    // Base 30 + min(5,3)*2 = 30 + 6 = 36
    assert_eq!(summary.attention_score, 36);
    assert!(summary
        .attention_reasons
        .iter()
        .any(|r| r.contains("Mock-style data candidates")));
}

#[test]
fn project_attention_summary_not_safe_for_cleanup_adds_6() {
    let mut overview = minimal_overview();
    overview["safe_for_cleanup"] = json!(false);
    let summary = project_attention_summary(&overview);
    // Base 30 + not_safe_cleanup 6 = 36
    assert_eq!(summary.attention_score, 36);
}

#[test]
fn project_attention_summary_not_safe_for_refactor_adds_6() {
    let mut overview = minimal_overview();
    overview["safe_for_refactor"] = json!(false);
    let summary = project_attention_summary(&overview);
    // Base 30 + not_safe_refactor 6 = 36
    assert_eq!(summary.attention_score, 36);
}

#[test]
fn project_attention_summary_evidence_quality_missing_snapshot() {
    let mut overview = minimal_overview();
    overview["observation"]["coverage_state"] = json!("missing_snapshot");
    let summary = project_attention_summary(&overview);
    assert_eq!(summary.evidence_quality, "missing");
}

#[test]
fn project_attention_summary_evidence_quality_stale() {
    let mut overview = minimal_overview();
    overview["observation"]["freshness"]["snapshot"]["status"] = json!("stale");
    let summary = project_attention_summary(&overview);
    assert_eq!(summary.evidence_quality, "stale");
}

#[test]
fn project_attention_summary_missing_key_fields_uses_defaults() {
    let overview = json!({});
    let summary = project_attention_summary(&overview);
    // Default action "inspect_workspace_state" => base 20
    // Default repo_risk_level "unknown" => +4
    // Default verification_status "not_recorded" => +18
    // Default snapshot "unknown" => freshness_attention_score("unknown", 14, 9) = 9
    // Default activity "unknown" => freshness_attention_score("unknown", 12, 8) = 8
    // Default verification freshness "unknown" => freshness_attention_score("unknown", 18, 12) = 12
    // safe_for_cleanup default false => +6
    // safe_for_refactor default false => +6
    // Total: 20 + 4 + 18 + 9 + 8 + 12 + 6 + 6 = 83
    assert_eq!(summary.attention_score, 83);
    assert_eq!(
        summary.priority_basis.recommended_next_action,
        "inspect_workspace_state"
    );
    assert_eq!(summary.priority_basis.repo_risk_level, "unknown");
    assert!(!summary.priority_basis.safe_for_cleanup);
    assert!(!summary.priority_basis.safe_for_refactor);
}

// --- workspace_portfolio_layer ---

#[test]
fn workspace_portfolio_layer_sorts_by_attention_and_truncates_to_five() {
    let overviews: Vec<Value> = (0..7)
        .map(|i| {
            let mut ov = minimal_overview();
            ov["project_id"] = json!(format!("proj{i}"));
            // Higher index = higher score via action base differences
            ov["recommended_next_action"] = if i < 3 {
                json!("inspect_hot_files") // base 30
            } else if i < 5 {
                json!("stabilize_repository_state") // base 100
            } else {
                json!("review_failing_verification") // base 95
            };
            ov
        })
        .collect();
    let result = workspace_portfolio_layer(
        &overviews,
        2,
        &["proj0".to_string(), "proj1".to_string()],
        vec![],
        0,
    );
    assert_eq!(result.project_count, 7);
    assert_eq!(result.monitoring_count, 2);
    // attention_queue is truncated to 5
    assert_eq!(result.attention_queue.len(), 5);
    // All 7 project overviews are preserved (not truncated)
    assert_eq!(result.project_overviews.len(), 7);
}

#[test]
fn workspace_portfolio_layer_empty_projects() {
    let result = workspace_portfolio_layer(&[], 0, &[], vec![], 0);
    assert_eq!(result.project_count, 0);
    assert_eq!(result.attention_queue.len(), 0);
    assert_eq!(result.dirty_projects, 0);
    assert_eq!(result.high_risk_projects, 0);
}

#[test]
fn workspace_portfolio_layer_counts_dirty_and_high_risk() {
    let overviews = vec![
        {
            let mut ov = minimal_overview();
            ov["project_id"] = json!("dirty_high");
            ov["repo_status_risk"]["is_dirty"] = json!(true);
            ov["repo_status_risk"]["risk_level"] = json!("high");
            ov
        },
        {
            let mut ov = minimal_overview();
            ov["project_id"] = json!("clean_low");
            ov
        },
    ];
    let result = workspace_portfolio_layer(&overviews, 1, &[], vec![], 0);
    assert_eq!(result.dirty_projects, 1);
    assert_eq!(result.high_risk_projects, 1);
}

#[test]
fn workspace_portfolio_layer_counts_hardcoded_candidates() {
    let overviews = vec![{
        let mut ov = minimal_overview();
        ov["project_id"] = json!("proj1");
        ov["mock_data_summary"]["hardcoded_candidate_count"] = json!(5);
        ov["mock_data_summary"]["mock_candidate_count"] = json!(3);
        ov
    }];
    let result = workspace_portfolio_layer(&overviews, 0, &[], vec![], 2);
    assert_eq!(result.projects_with_hardcoded_candidates, 1);
    assert_eq!(result.total_hardcoded_candidates, 5);
    assert_eq!(result.total_mock_candidates, 3);
    assert_eq!(result.projects_with_hardcoded_data_candidates, 2);
}

// --- sort_project_recommendations ---

#[test]
fn sort_project_recommendations_joins_and_sorts_by_attention() {
    let recommendations = vec![
        json!({
            "project_id": "low_prio",
            "confidence": "low",
        }),
        json!({
            "project_id": "high_prio",
            "confidence": "high",
        }),
    ];
    let overviews = vec![
        {
            let mut ov = minimal_overview();
            ov["project_id"] = json!("low_prio");
            // Base 30 for inspect_hot_files
            ov
        },
        {
            let mut ov = minimal_overview();
            ov["project_id"] = json!("high_prio");
            ov["recommended_next_action"] = json!("stabilize_repository_state");
            // Base 100
            ov
        },
    ];
    let sorted = sort_project_recommendations(&recommendations, &overviews);
    assert_eq!(sorted.len(), 2);
    // high_prio should be first (higher attention score)
    assert_eq!(sorted[0]["project_id"], "high_prio");
    assert_eq!(sorted[1]["project_id"], "low_prio");
    // Should have attention_score injected
    assert!(sorted[0].get("attention_score").is_some());
    assert!(sorted[0].get("attention_band").is_some());
    assert!(sorted[0].get("attention_reasons").is_some());
}

#[test]
fn sort_project_recommendations_skips_overviews_without_matching_recommendation() {
    let recommendations = vec![json!({
        "project_id": "only_one",
        "confidence": "medium",
    })];
    let overviews = vec![
        {
            let mut ov = minimal_overview();
            ov["project_id"] = json!("only_one");
            ov
        },
        {
            let mut ov = minimal_overview();
            ov["project_id"] = json!("no_match");
            ov
        },
    ];
    let sorted = sort_project_recommendations(&recommendations, &overviews);
    assert_eq!(sorted.len(), 1);
    assert_eq!(sorted[0]["project_id"], "only_one");
}

#[test]
fn sort_project_recommendations_empty_inputs() {
    let sorted = sort_project_recommendations(&[], &[]);
    assert!(sorted.is_empty());
}
