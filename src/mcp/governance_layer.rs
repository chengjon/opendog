use serde_json::{json, Value};
use crate::storage::database::Database;
use crate::storage::queries::{self, get_data_risk_cache};

pub(crate) fn build_governance_layer(
    project_dbs: &[(&String, &Database)],
) -> Value {
    let mut project_governance: Vec<Value> = Vec::new();
    let mut total_active_lanes = 0usize;
    let mut total_active_nodes = 0usize;
    let mut projects_with_governance = 0usize;
    let total_projects = project_dbs.len();

    for (project_id, db) in project_dbs {
        let lanes = match queries::get_governance_lanes(db) {
            Ok(l) => l,
            Err(_) => continue,
        };

        if lanes.is_empty() {
            continue;
        }

        projects_with_governance += 1;

        let mut lane_entries: Vec<Value> = Vec::new();
        for lane in &lanes {
            let active_nodes = queries::count_active_nodes_for_lane(db, &lane.lane_id).unwrap_or(0);
            total_active_nodes += active_nodes;

            let nodes = queries::get_governance_nodes(db, Some(&lane.lane_id), None)
                .unwrap_or_default();
            let latest_node = nodes.first();

            let mut lane_json = json!({
                "lane_id": lane.lane_id,
                "title": lane.title,
                "status": lane.status,
                "active_nodes": active_nodes,
            });

            if let Some(node) = latest_node {
                lane_json["latest_node"] = json!({
                    "node_id": node.node_id,
                    "state": node.state,
                    "summary": node.summary,
                    "suggested_next": node.suggested_next,
                    "forbidden_scope": node.forbidden_scope,
                    "updated_at": node.updated_at,
                });
            }

            lane_entries.push(lane_json);
        }

        total_active_lanes += lanes.iter().filter(|l| l.status == "active").count();

        // Observation cross-reference hints
        let snapshot_freshness = if let Ok(entries) = queries::get_snapshot_paths(db) {
            if !entries.is_empty() { "fresh" } else { "unknown" }
        } else { "unknown" };

        let verification_status = match queries::get_latest_verification_runs(db) {
            Ok(runs) if runs.iter().all(|r| r.status == "passed") => "passed",
            Ok(runs) if runs.is_empty() => "not_recorded",
            _ => "failed",
        };

        let unused_files_total = queries::count_unused(db).unwrap_or(0) as usize;
        let data_risk_candidates_total: usize = get_data_risk_cache(db)
            .ok()
            .flatten()
            .map(|c| c.mock_candidate_count + c.hardcoded_candidate_count)
            .unwrap_or(0);

        project_governance.push(json!({
            "project_id": project_id,
            "lanes": lane_entries,
            "observation_cross_reference": {
                "snapshot_freshness": snapshot_freshness,
                "verification_status": verification_status,
                "unused_files_total": unused_files_total,
                "data_risk_candidates_total": data_risk_candidates_total,
            }
        }));
    }

    let has_governance_state = !project_governance.is_empty();

    json!({
        "has_governance_state": has_governance_state,
        "project_governance": project_governance,
        "workspace_summary": {
            "total_active_lanes": total_active_lanes,
            "total_active_nodes": total_active_nodes,
            "projects_with_governance": projects_with_governance,
            "projects_without_governance": total_projects - projects_with_governance,
        }
    })
}
