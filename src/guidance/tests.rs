use super::*;
use serde_json::json;

// --- trim_agent_guidance_payload ---

#[test]
fn trim_agent_guidance_payload_truncates_project_recommendations() {
    let mut payload = json!({
        "guidance": {
            "project_recommendations": [
                {"project_id": "a"},
                {"project_id": "b"},
                {"project_id": "c"},
                {"project_id": "d"},
                {"project_id": "e"},
            ],
            "layers": {
                "execution_strategy": {
                    "project_recommendations": [
                        {"project_id": "a"},
                        {"project_id": "b"},
                        {"project_id": "c"},
                    ]
                },
                "multi_project_portfolio": {
                    "priority_candidates": [
                        {"project_id": "a"},
                        {"project_id": "b"},
                        {"project_id": "c"},
                        {"project_id": "d"},
                    ],
                    "attention_queue": [
                        {"project_id": "a"},
                        {"project_id": "b"},
                        {"project_id": "c"},
                        {"project_id": "d"},
                        {"project_id": "e"},
                    ],
                    "project_overviews": [
                        {"project_id": "a"},
                        {"project_id": "b"},
                        {"project_id": "c"},
                    ]
                }
            }
        }
    });

    trim_agent_guidance_payload(&mut payload, 2);

    // project_recommendations should be truncated to 2
    let recs = payload["guidance"]["project_recommendations"]
        .as_array()
        .unwrap();
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0]["project_id"], "a");
    assert_eq!(recs[1]["project_id"], "b");

    // execution_strategy project_recommendations truncated to 2
    let exec_recs = payload["guidance"]["layers"]["execution_strategy"]["project_recommendations"]
        .as_array()
        .unwrap();
    assert_eq!(exec_recs.len(), 2);

    // priority_candidates truncated to 2
    let candidates = payload["guidance"]["layers"]["multi_project_portfolio"]
        ["priority_candidates"]
        .as_array()
        .unwrap();
    assert_eq!(candidates.len(), 2);

    // attention_queue truncated to 2
    let queue = payload["guidance"]["layers"]["multi_project_portfolio"]["attention_queue"]
        .as_array()
        .unwrap();
    assert_eq!(queue.len(), 2);

    // project_overviews truncated to 2
    let overviews = payload["guidance"]["layers"]["multi_project_portfolio"]["project_overviews"]
        .as_array()
        .unwrap();
    assert_eq!(overviews.len(), 2);
}

#[test]
fn trim_agent_guidance_payload_no_op_when_under_limit() {
    let mut payload = json!({
        "guidance": {
            "project_recommendations": [{"project_id": "a"}],
            "layers": {}
        }
    });
    trim_agent_guidance_payload(&mut payload, 5);
    let recs = payload["guidance"]["project_recommendations"]
        .as_array()
        .unwrap();
    assert_eq!(recs.len(), 1);
}

#[test]
fn trim_agent_guidance_payload_zero_top() {
    let mut payload = json!({
        "guidance": {
            "project_recommendations": [{"project_id": "a"}, {"project_id": "b"}],
            "layers": {}
        }
    });
    trim_agent_guidance_payload(&mut payload, 0);
    let recs = payload["guidance"]["project_recommendations"]
        .as_array()
        .unwrap();
    assert_eq!(recs.len(), 0);
}

#[test]
fn trim_agent_guidance_payload_missing_paths_is_no_op() {
    let mut payload = json!({"other_key": "value"});
    trim_agent_guidance_payload(&mut payload, 3);
    // Should not panic; payload unchanged except no truncation targets exist
    assert_eq!(payload["other_key"], "value");
}

// --- guidance_notes ---

#[test]
fn guidance_notes_empty_list_warns_no_monitoring() {
    let notes = guidance_notes(&[]);
    assert_eq!(notes.len(), 1);
    assert!(notes[0].contains("No projects are currently marked as monitoring"));
}

#[test]
fn guidance_notes_single_project() {
    let notes = guidance_notes(&["myproject".to_string()]);
    assert_eq!(notes.len(), 1);
    assert!(notes[0].contains("myproject"));
    assert!(notes[0].contains("Currently monitored projects"));
}

#[test]
fn guidance_notes_multiple_projects() {
    let notes = guidance_notes(&["alpha".to_string(), "beta".to_string(), "gamma".to_string()]);
    assert_eq!(notes.len(), 1);
    assert!(notes[0].contains("alpha"));
    assert!(notes[0].contains("beta"));
    assert!(notes[0].contains("gamma"));
    // Comma-separated
    assert!(notes[0].contains("alpha, beta, gamma"));
}
