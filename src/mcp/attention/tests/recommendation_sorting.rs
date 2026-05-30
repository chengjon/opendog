use super::*;

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
