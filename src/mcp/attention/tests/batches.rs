use super::*;

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
