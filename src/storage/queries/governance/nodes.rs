use super::types::{GovernanceNode, UpsertGovernanceNode};
use crate::error::Result;
use crate::storage::database::Database;
use rusqlite::params;

pub fn delete_governance_nodes_by_lane(db: &Database, lane_id: &str) -> Result<usize> {
    db.execute(
        "DELETE FROM governance_nodes WHERE lane_id = ?1",
        params![lane_id],
    )
}

/// Upsert a governance node. Returns `true` if a new row was created, `false` if an existing row was updated.
///
/// On create (INSERT), the `state` field must be present — it is NOT NULL in the schema.
/// On update, only the fields that are `Some(...)` are patched; `None` fields are left unchanged.
pub fn upsert_governance_node(
    db: &Database,
    node: &UpsertGovernanceNode,
    now: &str,
) -> Result<bool> {
    // Try INSERT first. If the node_id already exists, fall through to dynamic UPDATE.
    let inserted = db.execute(
        "INSERT INTO governance_nodes
             (node_id, lane_id, state, summary, evidence_refs, artifact_refs,
              reported_git_head, suggested_next, forbidden_scope, external_anchors,
              created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
        params![
            node.node_id,
            node.lane_id,
            node.state,
            node.summary,
            node.evidence_refs,
            node.artifact_refs,
            node.reported_git_head,
            node.suggested_next,
            node.forbidden_scope,
            node.external_anchors,
            now,
        ],
    );

    match inserted {
        Ok(_) => Ok(true),
        Err(crate::error::OpenDogError::Database(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ErrorCode::ConstraintViolation,
                ..
            },
            _,
        ))) => {
            // UNIQUE constraint violation — node_id exists, perform dynamic UPDATE.
            let mut sets = vec!["updated_at = ?1".to_string()];
            let mut param_idx = 2u32;
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];

            macro_rules! maybe_set {
                ($field:expr, $col:expr) => {
                    if let Some(ref val) = $field {
                        sets.push(format!("{} = ?{}", $col, param_idx));
                        param_values.push(Box::new(val.clone()));
                        param_idx += 1;
                    }
                };
            }

            maybe_set!(node.state, "state");
            maybe_set!(node.summary, "summary");
            maybe_set!(node.evidence_refs, "evidence_refs");
            maybe_set!(node.artifact_refs, "artifact_refs");
            maybe_set!(node.reported_git_head, "reported_git_head");
            maybe_set!(node.suggested_next, "suggested_next");
            maybe_set!(node.forbidden_scope, "forbidden_scope");
            maybe_set!(node.external_anchors, "external_anchors");

            sets.push(format!("lane_id = ?{}", param_idx));
            param_values.push(Box::new(node.lane_id.clone()));
            param_idx += 1;

            let sql = format!(
                "UPDATE governance_nodes SET {} WHERE node_id = ?{}",
                sets.join(", "),
                param_idx
            );
            param_values.push(Box::new(node.node_id.clone()));

            // Build a Vec<&dyn ToSql> for rusqlite params.
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                param_values.iter().map(|p| p.as_ref()).collect();

            db.execute(&sql, param_refs.as_slice())?;
            Ok(false)
        }
        Err(e) => Err(e),
    }
}

/// Retrieve governance nodes, optionally filtered by lane_id and/or node_id.
pub fn get_governance_nodes(
    db: &Database,
    lane_id: Option<&str>,
    node_id: Option<&str>,
) -> Result<Vec<GovernanceNode>> {
    let mut sql = String::from(
        "SELECT node_id, lane_id, state, summary, evidence_refs, artifact_refs,
                reported_git_head, suggested_next, forbidden_scope, external_anchors,
                created_at, updated_at
         FROM governance_nodes WHERE 1=1",
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut idx = 1u32;

    if lane_id.is_some() {
        sql.push_str(&format!(" AND lane_id = ?{}", idx));
        idx += 1;
    }
    if node_id.is_some() {
        sql.push_str(&format!(" AND node_id = ?{}", idx));
    }
    sql.push_str(" ORDER BY updated_at DESC");

    if let Some(lid) = lane_id {
        param_values.push(Box::new(lid.to_string()));
    }
    if let Some(nid) = node_id {
        param_values.push(Box::new(nid.to_string()));
    }

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();

    db.prepare_and_query(&sql, param_refs.as_slice(), |row| {
        Ok(GovernanceNode {
            node_id: row.get(0)?,
            lane_id: row.get(1)?,
            state: row.get(2)?,
            summary: row.get(3)?,
            evidence_refs: row.get(4)?,
            artifact_refs: row.get(5)?,
            reported_git_head: row.get(6)?,
            suggested_next: row.get(7)?,
            forbidden_scope: row.get(8)?,
            external_anchors: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })
}
