mod helpers;
mod hints;
mod types;

use self::helpers::{json_to_string, now_timestamp, string_list_to_json};
use self::hints::compute_observation_hints;
use crate::error::{OpenDogError, Result};
use crate::storage::database::Database;
use crate::storage::queries::{self, GovernanceLane, NewGovernanceLane, UpsertGovernanceNode};

pub use self::types::{
    CloseLaneInput, CreateLaneInput, GetGovernanceStateInput, GovernanceLaneSummary,
    GovernanceState, ObservationHints, UpsertNodeInput, UpsertNodeResult,
};

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
        return Err(OpenDogError::GovernanceNodeStateRequired(
            input.node_id.clone(),
        ));
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
    let node = nodes.into_iter().next().ok_or_else(|| {
        OpenDogError::GovernanceLaneNotFound(format!(
            "node {} vanished after upsert",
            input.node_id
        ))
    })?;

    Ok(UpsertNodeResult {
        node_id: node.node_id,
        lane_id: node.lane_id,
        state: node.state,
        created,
    })
}

/// Retrieve governance state, optionally filtered by lane and/or node.
pub fn get_governance_state(
    db: &Database,
    input: GetGovernanceStateInput,
) -> Result<GovernanceState> {
    let nodes =
        queries::get_governance_nodes(db, input.lane_id.as_deref(), input.node_id.as_deref())?;

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
        all_lanes.iter().filter(|l| l.status == "active").collect()
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

#[cfg(test)]
mod tests;
