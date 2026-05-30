use super::*;

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
