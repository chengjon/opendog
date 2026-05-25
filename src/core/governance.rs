use crate::error::{OpenDogError, Result};
use crate::storage::database::Database;
use crate::storage::queries::{self, get_data_risk_cache, GovernanceLane, GovernanceNode, NewGovernanceLane, UpsertGovernanceNode};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Input / output types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLaneInput {
    pub lane_id: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertNodeInput {
    pub node_id: String,
    pub lane_id: String,
    pub state: Option<String>,
    pub summary: Option<String>,
    pub evidence_refs: Option<Vec<String>>,
    pub artifact_refs: Option<Vec<String>>,
    pub reported_git_head: Option<String>,
    pub suggested_next: Option<String>,
    pub forbidden_scope: Option<Vec<String>>,
    pub external_anchors: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGovernanceStateInput {
    pub lane_id: Option<String>,
    pub node_id: Option<String>,
    pub active_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseLaneInput {
    pub lane_id: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationHints {
    pub snapshot_freshness: String,
    pub verification_status: String,
    pub data_risk_candidates: usize,
    pub unused_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceState {
    pub lanes: Vec<GovernanceLaneSummary>,
    pub nodes: Vec<GovernanceNode>,
    pub observation_hints: ObservationHints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceLaneSummary {
    pub lane_id: String,
    pub title: String,
    pub status: String,
    pub node_count: usize,
    pub active_nodes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertNodeResult {
    pub node_id: String,
    pub lane_id: String,
    pub state: String,
    pub created: bool,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    now.as_secs().to_string()
}

fn json_to_string(val: &Option<serde_json::Value>) -> Option<String> {
    val.as_ref().map(|v| v.to_string())
}

fn string_list_to_json(list: &Option<Vec<String>>) -> Option<String> {
    list.as_ref().map(|items| serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string()))
}

fn compute_observation_hints(db: &Database) -> ObservationHints {
    // Snapshot freshness — check if any snapshot data exists
    let snapshot_freshness =
        if let Ok(entries) = queries::get_snapshot_paths(db) {
            if !entries.is_empty() {
                "fresh"
            } else {
                "unknown"
            }
        } else {
            "unknown"
        };

    // Verification status
    let verification_status = match queries::get_latest_verification_runs(db) {
        Ok(runs) if runs.iter().all(|r| r.status == "passed") => "passed",
        Ok(runs) if runs.is_empty() => "not_recorded",
        _ => "failed",
    };

    // Unused files count
    let unused_files = queries::count_unused(db).unwrap_or(0) as usize;

    // Data risk candidates — read from cache populated by data-risk detection
    let data_risk_candidates: usize = get_data_risk_cache(db)
        .ok()
        .flatten()
        .map(|c| c.mock_candidate_count + c.hardcoded_candidate_count)
        .unwrap_or(0);

    ObservationHints {
        snapshot_freshness: snapshot_freshness.to_string(),
        verification_status: verification_status.to_string(),
        data_risk_candidates,
        unused_files,
    }
}

// ---------------------------------------------------------------------------
// Public business logic
// ---------------------------------------------------------------------------

/// Create a new governance lane.
pub fn create_lane(db: &Database, input: CreateLaneInput) -> Result<GovernanceLane> {
    let now = now_timestamp();
    let new_lane = NewGovernanceLane {
        lane_id: input.lane_id,
        title: input.title,
        description: input.description,
    };
    queries::insert_governance_lane(db, &new_lane, &now)?;

    queries::get_governance_lane_by_id(db, &new_lane.lane_id)?
        .ok_or_else(|| OpenDogError::GovernanceLaneNotFound(new_lane.lane_id.clone()))
}

/// Insert a new governance node or update an existing one.
///
/// On create the `state` field is required. On update only the supplied
/// `Some(...)` fields are patched.
pub fn upsert_node(db: &Database, input: UpsertNodeInput) -> Result<UpsertNodeResult> {
    // Validate lane exists.
    let lane = queries::get_governance_lane_by_id(db, &input.lane_id)?;
    if lane.is_none() {
        return Err(OpenDogError::GovernanceLaneNotFound(input.lane_id.clone()));
    }

    // Check if node already exists.
    let existing = queries::get_governance_nodes(db, None, Some(&input.node_id))?;
    let is_create = existing.is_empty();

    // On create, state is mandatory.
    if is_create && input.state.is_none() {
        return Err(OpenDogError::GovernanceNodeStateRequired(input.node_id.clone()));
    }

    let upsert = UpsertGovernanceNode {
        node_id: input.node_id.clone(),
        lane_id: input.lane_id.clone(),
        state: input.state,
        summary: input.summary,
        evidence_refs: string_list_to_json(&input.evidence_refs),
        artifact_refs: string_list_to_json(&input.artifact_refs),
        reported_git_head: input.reported_git_head,
        suggested_next: input.suggested_next,
        forbidden_scope: string_list_to_json(&input.forbidden_scope),
        external_anchors: json_to_string(&input.external_anchors),
    };

    let created = queries::upsert_governance_node(db, &upsert, &now_timestamp())?;

    // Read back to get authoritative state.
    let nodes = queries::get_governance_nodes(db, None, Some(&input.node_id))?;
    let node = nodes
        .into_iter()
        .next()
        .ok_or_else(|| OpenDogError::GovernanceLaneNotFound(format!("node {} vanished after upsert", input.node_id)))?;

    Ok(UpsertNodeResult {
        node_id: node.node_id,
        lane_id: node.lane_id,
        state: node.state,
        created,
    })
}

/// Retrieve governance state, optionally filtered by lane and/or node.
pub fn get_governance_state(db: &Database, input: GetGovernanceStateInput) -> Result<GovernanceState> {
    let nodes = queries::get_governance_nodes(
        db,
        input.lane_id.as_deref(),
        input.node_id.as_deref(),
    )?;

    let all_lanes = queries::get_governance_lanes(db)?;

    // Collect the unique lane IDs from the returned nodes so we can decide
    // which lane summaries to include.  If a lane filter was provided we
    // only include that lane; otherwise include all lanes.
    let lane_ids_to_include: Vec<String> = if let Some(ref lid) = input.lane_id {
        vec![lid.clone()]
    } else {
        all_lanes.iter().map(|l| l.lane_id.clone()).collect()
    };

    let active_only = input.active_only.unwrap_or(false);

    // Filter out non-active lanes when active_only is set
    let lanes_to_show: Vec<&crate::storage::queries::GovernanceLane> = if active_only {
        all_lanes.iter()
            .filter(|l| l.status == "active")
            .collect()
    } else {
        all_lanes.iter().collect()
    };

    let mut lanes = Vec::with_capacity(lanes_to_show.len());
    for lane in &lanes_to_show {
        if lane_ids_to_include.contains(&lane.lane_id) {
            let node_count = queries::count_nodes_for_lane(db, &lane.lane_id)?;
            let active_nodes = queries::count_active_nodes_for_lane(db, &lane.lane_id)?;
            lanes.push(GovernanceLaneSummary {
                lane_id: lane.lane_id.clone(),
                title: lane.title.clone(),
                status: lane.status.clone(),
                node_count,
                active_nodes,
            });
        }
    }

    let observation_hints = compute_observation_hints(db);

    // Filter out closed nodes when active_only is set
    let nodes = if active_only {
        nodes.into_iter().filter(|n| n.state != "closed").collect()
    } else {
        nodes
    };

    Ok(GovernanceState {
        lanes,
        nodes,
        observation_hints,
    })
}

/// Close a lane. Actions: "complete", "defer", "delete".
///
/// Returns `(status_string, nodes_affected_count)`.
pub fn close_lane(db: &Database, input: CloseLaneInput) -> Result<(String, usize)> {
    // Validate lane exists.
    let lane = queries::get_governance_lane_by_id(db, &input.lane_id)?;
    if lane.is_none() {
        return Err(OpenDogError::GovernanceLaneNotFound(input.lane_id.clone()));
    }

    let now = now_timestamp();

    match input.action.as_str() {
        "complete" => {
            let node_count = queries::count_nodes_for_lane(db, &input.lane_id)?;
            queries::update_lane_status(db, &input.lane_id, "completed", &now)?;
            Ok(("completed".to_string(), node_count))
        }
        "defer" => {
            let node_count = queries::count_nodes_for_lane(db, &input.lane_id)?;
            queries::update_lane_status(db, &input.lane_id, "deferred", &now)?;
            Ok(("deferred".to_string(), node_count))
        }
        "delete" => {
            let nodes_deleted = queries::delete_governance_nodes_by_lane(db, &input.lane_id)?;
            queries::delete_governance_lane(db, &input.lane_id)?;
            Ok(("deleted".to_string(), nodes_deleted))
        }
        _ => Err(OpenDogError::InvalidInput(format!(
            "invalid close action '{}'; expected one of: complete, defer, delete",
            input.action
        ))),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("governance_core_test.db");
        let db = Database::open_project(&db_path).unwrap();
        Box::leak(Box::new(dir));
        db
    }

    #[test]
    fn create_lane_and_upsert_node_flow() {
        let db = test_db();

        // Create lane
        let lane = create_lane(
            &db,
            CreateLaneInput {
                lane_id: "lane-flow".to_string(),
                title: "Happy path".to_string(),
                description: Some("full flow test".to_string()),
            },
        )
        .unwrap();
        assert_eq!(lane.lane_id, "lane-flow");
        assert_eq!(lane.title, "Happy path");
        assert_eq!(lane.status, "active");

        // Upsert node (create)
        let result = upsert_node(
            &db,
            UpsertNodeInput {
                node_id: "node-flow-1".to_string(),
                lane_id: "lane-flow".to_string(),
                state: Some("open".to_string()),
                summary: Some("first node".to_string()),
                evidence_refs: Some(vec!["e1".to_string(), "e2".to_string()]),
                artifact_refs: None,
                reported_git_head: Some("abc123".to_string()),
                suggested_next: None,
                forbidden_scope: Some(vec!["scope-a".to_string()]),
                external_anchors: Some(serde_json::json!({"link": "https://example.com"})),
            },
        )
        .unwrap();
        assert!(result.created);
        assert_eq!(result.node_id, "node-flow-1");
        assert_eq!(result.state, "open");

        // Upsert same node (update) — change state, leave summary unchanged
        let updated = upsert_node(
            &db,
            UpsertNodeInput {
                node_id: "node-flow-1".to_string(),
                lane_id: "lane-flow".to_string(),
                state: Some("in_progress".to_string()),
                summary: None,
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
            },
        )
        .unwrap();
        assert!(!updated.created);
        assert_eq!(updated.state, "in_progress");

        // Verify via get_governance_state
        let state = get_governance_state(
            &db,
            GetGovernanceStateInput {
                lane_id: Some("lane-flow".to_string()),
                node_id: None,
                active_only: None,
            },
        )
        .unwrap();
        assert_eq!(state.nodes.len(), 1);
        assert_eq!(state.nodes[0].summary, Some("first node".to_string()));
        assert_eq!(state.lanes.len(), 1);
        assert_eq!(state.lanes[0].node_count, 1);
        assert_eq!(state.lanes[0].active_nodes, 1);
    }

    #[test]
    fn upsert_rejects_missing_state_on_create() {
        let db = test_db();

        create_lane(
            &db,
            CreateLaneInput {
                lane_id: "lane-state".to_string(),
                title: "State required".to_string(),
                description: None,
            },
        )
        .unwrap();

        let err = upsert_node(
            &db,
            UpsertNodeInput {
                node_id: "node-no-state".to_string(),
                lane_id: "lane-state".to_string(),
                state: None,
                summary: None,
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
            },
        )
        .unwrap_err();

        match err {
            OpenDogError::GovernanceNodeStateRequired(node_id) => {
                assert_eq!(node_id, "node-no-state");
            }
            other => panic!("expected GovernanceNodeStateRequired, got {:?}", other),
        }
    }

    #[test]
    fn upsert_rejects_unknown_lane() {
        let db = test_db();

        let err = upsert_node(
            &db,
            UpsertNodeInput {
                node_id: "node-ghost".to_string(),
                lane_id: "no-such-lane".to_string(),
                state: Some("open".to_string()),
                summary: None,
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
            },
        )
        .unwrap_err();

        match err {
            OpenDogError::GovernanceLaneNotFound(lane_id) => {
                assert_eq!(lane_id, "no-such-lane");
            }
            other => panic!("expected GovernanceLaneNotFound, got {:?}", other),
        }
    }

    #[test]
    fn close_lane_complete_marks_completed() {
        let db = test_db();

        create_lane(
            &db,
            CreateLaneInput {
                lane_id: "lane-complete".to_string(),
                title: "To complete".to_string(),
                description: None,
            },
        )
        .unwrap();

        upsert_node(
            &db,
            UpsertNodeInput {
                node_id: "node-c1".to_string(),
                lane_id: "lane-complete".to_string(),
                state: Some("open".to_string()),
                summary: None,
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
            },
        )
        .unwrap();

        let (status, count) = close_lane(
            &db,
            CloseLaneInput {
                lane_id: "lane-complete".to_string(),
                action: "complete".to_string(),
            },
        )
        .unwrap();

        assert_eq!(status, "completed");
        assert_eq!(count, 1);

        // Verify lane status changed
        let lane = queries::get_governance_lane_by_id(&db, "lane-complete")
            .unwrap()
            .unwrap();
        assert_eq!(lane.status, "completed");
    }

    #[test]
    fn close_lane_delete_removes_everything() {
        let db = test_db();

        create_lane(
            &db,
            CreateLaneInput {
                lane_id: "lane-delete".to_string(),
                title: "To delete".to_string(),
                description: None,
            },
        )
        .unwrap();

        // Create two nodes
        for i in 0..2 {
            upsert_node(
                &db,
                UpsertNodeInput {
                    node_id: format!("node-d{}", i),
                    lane_id: "lane-delete".to_string(),
                    state: Some("open".to_string()),
                    summary: None,
                    evidence_refs: None,
                    artifact_refs: None,
                    reported_git_head: None,
                    suggested_next: None,
                    forbidden_scope: None,
                    external_anchors: None,
                },
            )
            .unwrap();
        }

        let (status, count) = close_lane(
            &db,
            CloseLaneInput {
                lane_id: "lane-delete".to_string(),
                action: "delete".to_string(),
            },
        )
        .unwrap();

        assert_eq!(status, "deleted");
        assert_eq!(count, 2);

        // Lane is gone
        assert!(queries::get_governance_lane_by_id(&db, "lane-delete")
            .unwrap()
            .is_none());

        // Nodes are gone
        assert_eq!(
            queries::get_governance_nodes(&db, Some("lane-delete"), None)
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn get_governance_nodes_returns_latest_updated_first() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open_project(&db_path).unwrap();

        let now = "2026-01-01T00:00:00Z";
        let lane = NewGovernanceLane {
            lane_id: "lane-order".to_string(),
            title: "Order test".to_string(),
            description: None,
        };
        queries::insert_governance_lane(&db, &lane, now).unwrap();

        let node_a = UpsertNodeInput {
            lane_id: "lane-order".to_string(),
            node_id: "node-a".to_string(),
            state: Some("planning".to_string()),
            summary: Some("First node".to_string()),
            evidence_refs: None,
            artifact_refs: None,
            reported_git_head: None,
            suggested_next: None,
            forbidden_scope: None,
            external_anchors: None,
        };
        upsert_node(&db, node_a).unwrap();

        let node_b = UpsertNodeInput {
            lane_id: "lane-order".to_string(),
            node_id: "node-b".to_string(),
            state: Some("executing".to_string()),
            summary: Some("Second node".to_string()),
            evidence_refs: None,
            artifact_refs: None,
            reported_git_head: None,
            suggested_next: None,
            forbidden_scope: None,
            external_anchors: None,
        };
        upsert_node(&db, node_b).unwrap();

        // Update node-a so it has a later updated_at than node-b
        let node_a_update = UpsertNodeInput {
            lane_id: "lane-order".to_string(),
            node_id: "node-a".to_string(),
            state: Some("reviewing".to_string()),
            summary: Some("Updated first node".to_string()),
            evidence_refs: None,
            artifact_refs: None,
            reported_git_head: None,
            suggested_next: None,
            forbidden_scope: None,
            external_anchors: None,
        };
        upsert_node(&db, node_a_update).unwrap();

        let nodes = queries::get_governance_nodes(&db, Some("lane-order"), None).unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].node_id, "node-a", "most recently updated node should be first");
    }
}
