use super::*;

fn portfolio_overview(project_id: &str, recommended_next_action: &str) -> serde_json::Value {
    json!({
        "project_id": project_id,
        "unused_files": 0,
        "recommended_next_action": recommended_next_action,
        "strategy_confidence": "high",
        "observation": {
            "coverage_state": "ready",
            "freshness": {
                "snapshot": {"status": "fresh"},
                "activity": {"status": "fresh"},
                "verification": {"status": "fresh"}
            }
        },
        "mock_data_summary": {
            "hardcoded_candidate_count": 0,
            "mock_candidate_count": 0
        },
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
        "safe_for_cleanup": true,
        "safe_for_refactor": true
    })
}

fn assert_attention_batch_metadata(value: &serde_json::Value, batched: usize, unbatched: usize) {
    assert_eq!(value["attention_batches"]["status"], json!("available"));
    assert_eq!(
        value["attention_batches"]["source"],
        json!("attention_queue")
    );
    assert_eq!(
        value["attention_batches"]["batched_project_count"],
        json!(batched)
    );
    assert_eq!(
        value["attention_batches"]["unbatched_project_count"],
        json!(unbatched)
    );
}

fn thin_attention_queue_entry(project: &serde_json::Value) -> serde_json::Value {
    json!({
        "project_id": project["project_id"].clone(),
        "recommended_next_action": project["recommended_next_action"].clone(),
        "attention_score": project["attention_score"].clone(),
        "attention_band": project["attention_band"].clone(),
    })
}

fn assert_attention_batch_matches_queue_slice(
    value: &serde_json::Value,
    bucket: &str,
    start: usize,
    end: usize,
) {
    let attention_queue = value["attention_queue"].as_array().unwrap();
    let expected = attention_queue
        .get(start..end)
        .unwrap_or(&[])
        .iter()
        .map(thin_attention_queue_entry)
        .collect::<Vec<_>>();

    assert_eq!(value["attention_batches"][bucket], json!(expected));
}

fn assert_attention_batches_match_queue(value: &serde_json::Value) {
    let attention_queue_len = value["attention_queue"].as_array().unwrap().len();
    assert_attention_batch_matches_queue_slice(value, "immediate", 0, attention_queue_len.min(1));
    assert_attention_batch_matches_queue_slice(value, "next", 1, attention_queue_len.min(3));
    assert_attention_batch_matches_queue_slice(value, "later", 3, attention_queue_len);
}

#[test]
fn workspace_portfolio_layer_exposes_attention_scores_and_reasons() {
    let mut alpha = portfolio_overview("alpha", "run_verification_before_high_risk_changes");
    alpha["observation"]["coverage_state"] = json!("stale_evidence");
    alpha["observation"]["freshness"]["verification"]["status"] = json!("missing");
    alpha["mock_data_summary"]["hardcoded_candidate_count"] = json!(2);
    alpha["mock_data_summary"]["mock_candidate_count"] = json!(1);
    alpha["verification_evidence"]["status"] = json!("not_recorded");
    alpha["repo_status_risk"]["risk_level"] = json!("medium");
    alpha["repo_status_risk"]["is_dirty"] = json!(true);
    alpha["safe_for_cleanup"] = json!(false);
    alpha["safe_for_refactor"] = json!(false);

    let value = workspace_portfolio_layer(&[alpha]);

    assert!(value["attention_queue"][0]["attention_score"].is_i64());
    assert!(value["attention_queue"][0]["attention_band"].is_string());
    assert!(value["attention_queue"][0]["attention_reasons"].is_array());
    assert_eq!(
        value["attention_queue"][0]["priority_basis"]["recommended_next_action"],
        json!("run_verification_before_high_risk_changes")
    );
}

#[test]
fn workspace_portfolio_layer_batches_attention_queue_into_immediate_next_and_later() {
    let mut alpha = portfolio_overview("alpha", "stabilize_repository_state");
    alpha["repo_status_risk"]["risk_level"] = json!("high");
    alpha["repo_status_risk"]["is_dirty"] = json!(true);
    alpha["repo_status_risk"]["operation_states"] = json!(["merge"]);
    alpha["safe_for_cleanup"] = json!(false);
    alpha["safe_for_refactor"] = json!(false);

    let mut beta = portfolio_overview("beta", "review_failing_verification");
    beta["repo_status_risk"]["risk_level"] = json!("medium");
    beta["verification_evidence"]["failing_runs"] = json!([{ "kind": "test" }]);

    let mut gamma = portfolio_overview("gamma", "run_verification_before_high_risk_changes");
    gamma["observation"]["coverage_state"] = json!("missing_verification");
    gamma["observation"]["freshness"]["verification"]["status"] = json!("missing");
    gamma["mock_data_summary"]["hardcoded_candidate_count"] = json!(1);
    gamma["mock_data_summary"]["mock_candidate_count"] = json!(1);
    gamma["verification_evidence"]["status"] = json!("not_recorded");
    gamma["repo_status_risk"]["risk_level"] = json!("medium");
    gamma["repo_status_risk"]["is_dirty"] = json!(true);
    gamma["safe_for_cleanup"] = json!(false);
    gamma["safe_for_refactor"] = json!(false);

    let mut delta = portfolio_overview("delta", "take_snapshot");
    delta["observation"]["coverage_state"] = json!("missing_snapshot");
    delta["observation"]["freshness"]["snapshot"]["status"] = json!("missing");

    let mut echo = portfolio_overview("echo", "inspect_hot_files");
    echo["observation"]["coverage_state"] = json!("stale_evidence");
    echo["observation"]["freshness"]["activity"]["status"] = json!("stale");

    let foxtrot = portfolio_overview("foxtrot", "inspect_hot_files");

    let value = workspace_portfolio_layer(&[alpha, beta, gamma, delta, echo, foxtrot]);

    assert_attention_batch_metadata(&value, 5, 1);
    assert_attention_batches_match_queue(&value);
}

#[test]
fn workspace_portfolio_layer_batches_attention_queue_safely_when_empty() {
    let value = workspace_portfolio_layer(&[]);

    assert_attention_batch_metadata(&value, 0, 0);
    assert_attention_batches_match_queue(&value);
}

#[test]
fn workspace_portfolio_layer_batches_single_attention_project_into_immediate_only() {
    let mut alpha = portfolio_overview("alpha", "review_failing_verification");
    alpha["repo_status_risk"]["risk_level"] = json!("medium");
    alpha["verification_evidence"]["failing_runs"] = json!([{ "kind": "test" }]);

    let value = workspace_portfolio_layer(&[alpha]);

    assert_attention_batch_metadata(&value, 1, 0);
    assert_attention_batches_match_queue(&value);
}

#[test]
fn workspace_portfolio_layer_batches_two_attention_projects_without_overflowing_next_or_later() {
    let mut alpha = portfolio_overview("alpha", "review_failing_verification");
    alpha["repo_status_risk"]["risk_level"] = json!("medium");
    alpha["verification_evidence"]["failing_runs"] = json!([{ "kind": "test" }]);

    let mut beta = portfolio_overview("beta", "take_snapshot");
    beta["observation"]["coverage_state"] = json!("missing_snapshot");
    beta["observation"]["freshness"]["snapshot"]["status"] = json!("missing");

    let value = workspace_portfolio_layer(&[alpha, beta]);

    assert_attention_batch_metadata(&value, 2, 0);
    assert_attention_batches_match_queue(&value);
}

#[test]
fn workspace_portfolio_layer_batches_three_attention_projects_without_overflowing_later() {
    let mut alpha = portfolio_overview("alpha", "review_failing_verification");
    alpha["repo_status_risk"]["risk_level"] = json!("medium");
    alpha["verification_evidence"]["failing_runs"] = json!([{ "kind": "test" }]);

    let mut beta = portfolio_overview("beta", "run_verification_before_high_risk_changes");
    beta["observation"]["coverage_state"] = json!("missing_verification");
    beta["observation"]["freshness"]["verification"]["status"] = json!("missing");
    beta["mock_data_summary"]["hardcoded_candidate_count"] = json!(1);
    beta["mock_data_summary"]["mock_candidate_count"] = json!(1);
    beta["verification_evidence"]["status"] = json!("not_recorded");
    beta["repo_status_risk"]["risk_level"] = json!("medium");
    beta["repo_status_risk"]["is_dirty"] = json!(true);
    beta["safe_for_cleanup"] = json!(false);
    beta["safe_for_refactor"] = json!(false);

    let mut gamma = portfolio_overview("gamma", "take_snapshot");
    gamma["observation"]["coverage_state"] = json!("missing_snapshot");
    gamma["observation"]["freshness"]["snapshot"]["status"] = json!("missing");

    let value = workspace_portfolio_layer(&[alpha, beta, gamma]);

    assert_attention_batch_metadata(&value, 3, 0);
    assert_attention_batches_match_queue(&value);
}
