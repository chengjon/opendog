use crate::error::Result;
use crate::storage::database::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceLane {
    pub lane_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceNode {
    pub node_id: String,
    pub lane_id: String,
    pub state: String,
    pub summary: Option<String>,
    pub evidence_refs: Option<String>,
    pub artifact_refs: Option<String>,
    pub reported_git_head: Option<String>,
    pub suggested_next: Option<String>,
    pub forbidden_scope: Option<String>,
    pub external_anchors: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGovernanceLane {
    pub lane_id: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertGovernanceNode {
    pub node_id: String,
    pub lane_id: String,
    pub state: Option<String>,
    pub summary: Option<String>,
    pub evidence_refs: Option<String>,
    pub artifact_refs: Option<String>,
    pub reported_git_head: Option<String>,
    pub suggested_next: Option<String>,
    pub forbidden_scope: Option<String>,
    pub external_anchors: Option<String>,
}

pub fn insert_governance_lane(db: &Database, lane: &NewGovernanceLane, now: &str) -> Result<()> {
    db.execute(
        "INSERT INTO governance_lanes (lane_id, title, description, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'active', ?4, ?5)",
        params![lane.lane_id, lane.title, lane.description, now, now],
    )?;
    Ok(())
}

pub fn get_governance_lanes(db: &Database) -> Result<Vec<GovernanceLane>> {
    db.prepare_and_query(
        "SELECT lane_id, title, description, status, created_at, updated_at
         FROM governance_lanes
         ORDER BY created_at",
        params![],
        |row| {
            Ok(GovernanceLane {
                lane_id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    )
}

pub fn get_governance_lane_by_id(db: &Database, lane_id: &str) -> Result<Option<GovernanceLane>> {
    match db.query_row(
        "SELECT lane_id, title, description, status, created_at, updated_at
         FROM governance_lanes
         WHERE lane_id = ?1",
        params![lane_id],
        |row| {
            Ok(GovernanceLane {
                lane_id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    ) {
        Ok(lane) => Ok(Some(lane)),
        Err(crate::error::OpenDogError::Database(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn update_lane_status(db: &Database, lane_id: &str, status: &str, now: &str) -> Result<usize> {
    db.execute(
        "UPDATE governance_lanes SET status = ?1, updated_at = ?2 WHERE lane_id = ?3",
        params![status, now, lane_id],
    )
}

pub fn delete_governance_nodes_by_lane(db: &Database, lane_id: &str) -> Result<usize> {
    db.execute(
        "DELETE FROM governance_nodes WHERE lane_id = ?1",
        params![lane_id],
    )
}

pub fn delete_governance_lane(db: &Database, lane_id: &str) -> Result<usize> {
    db.execute(
        "DELETE FROM governance_lanes WHERE lane_id = ?1",
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

/// Counts nodes not in 'closed' state. Projects define their own state vocabulary;
/// this uses 'closed' as the default inactive-state convention.
pub fn count_active_nodes_for_lane(db: &Database, lane_id: &str) -> Result<usize> {
    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM governance_nodes WHERE lane_id = ?1 AND state != 'closed'",
        params![lane_id],
        |row| row.get(0),
    )?;
    Ok(count as usize)
}

/// Count all nodes for a lane regardless of state.
pub fn count_nodes_for_lane(db: &Database, lane_id: &str) -> Result<usize> {
    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM governance_nodes WHERE lane_id = ?1",
        params![lane_id],
        |row| row.get(0),
    )?;
    Ok(count as usize)
}

/// Count lanes with status = 'active'.
pub fn count_all_active_lanes(db: &Database) -> Result<usize> {
    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM governance_lanes WHERE status = 'active'",
        params![],
        |row| row.get(0),
    )?;
    Ok(count as usize)
}

/// Counts all nodes across all lanes where state != 'closed'. Projects define their
/// own state vocabulary; this uses 'closed' as the default inactive-state convention.
pub fn count_all_active_nodes(db: &Database) -> Result<usize> {
    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM governance_nodes WHERE state != 'closed'",
        params![],
        |row| row.get(0),
    )?;
    Ok(count as usize)
}

/// Returns `true` if the project has any governance data (at least one lane).
pub fn has_governance_data(db: &Database) -> Result<bool> {
    let count: i64 = db.query_row("SELECT COUNT(*) FROM governance_lanes", params![], |row| {
        row.get(0)
    })?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("governance_test.db");
        let db = Database::open_project(&db_path).unwrap();
        Box::leak(Box::new(dir));
        db
    }

    #[test]
    fn insert_and_read_lane() {
        let db = test_db();
        let now = "2026-05-24T12:00:00Z";

        let lane = NewGovernanceLane {
            lane_id: "lane-1".to_string(),
            title: "Test Lane".to_string(),
            description: Some("A test lane".to_string()),
        };
        insert_governance_lane(&db, &lane, now).unwrap();

        // Read back by id
        let found = get_governance_lane_by_id(&db, "lane-1").unwrap();
        assert!(found.is_some());
        let l = found.unwrap();
        assert_eq!(l.lane_id, "lane-1");
        assert_eq!(l.title, "Test Lane");
        assert_eq!(l.description, Some("A test lane".to_string()));
        assert_eq!(l.status, "active");
        assert_eq!(l.created_at, now);

        // List all lanes
        let lanes = get_governance_lanes(&db).unwrap();
        assert_eq!(lanes.len(), 1);

        // Not found returns None
        let missing = get_governance_lane_by_id(&db, "nope").unwrap();
        assert!(missing.is_none());

        // has_governance_data
        assert!(has_governance_data(&db).unwrap());
    }

    #[test]
    fn upsert_creates_and_updates_node() {
        let db = test_db();
        let now = "2026-05-24T12:00:00Z";

        // Create lane first
        let lane = NewGovernanceLane {
            lane_id: "lane-2".to_string(),
            title: "Node Test".to_string(),
            description: None,
        };
        insert_governance_lane(&db, &lane, now).unwrap();

        // Insert node
        let created = upsert_governance_node(
            &db,
            &UpsertGovernanceNode {
                node_id: "node-1".to_string(),
                lane_id: "lane-2".to_string(),
                state: Some("open".to_string()),
                summary: Some("initial summary".to_string()),
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
            },
            now,
        )
        .unwrap();
        assert!(created, "first upsert should create");

        // Update node — change state and summary, leave other fields as None (no overwrite)
        let updated = upsert_governance_node(
            &db,
            &UpsertGovernanceNode {
                node_id: "node-1".to_string(),
                lane_id: "lane-2".to_string(),
                state: Some("in_progress".to_string()),
                summary: None, // should retain old value
                evidence_refs: Some("ref-1".to_string()),
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: Some("do-next".to_string()),
                forbidden_scope: None,
                external_anchors: None,
            },
            "2026-05-24T13:00:00Z",
        )
        .unwrap();
        assert!(!updated, "second upsert should update");

        // Verify fields
        let nodes = get_governance_nodes(&db, Some("lane-2"), None).unwrap();
        assert_eq!(nodes.len(), 1);
        let n = &nodes[0];
        assert_eq!(n.node_id, "node-1");
        assert_eq!(n.state, "in_progress");
        assert_eq!(
            n.summary,
            Some("initial summary".to_string()),
            "summary should be retained from first insert"
        );
        assert_eq!(n.evidence_refs, Some("ref-1".to_string()));
        assert_eq!(n.suggested_next, Some("do-next".to_string()));

        // Count active nodes
        assert_eq!(count_active_nodes_for_lane(&db, "lane-2").unwrap(), 1);
        assert_eq!(count_nodes_for_lane(&db, "lane-2").unwrap(), 1);
    }

    #[test]
    fn close_lane_deletes_nodes_on_delete() {
        let db = test_db();
        let now = "2026-05-24T12:00:00Z";

        // Create lane
        let lane = NewGovernanceLane {
            lane_id: "lane-3".to_string(),
            title: "Delete Test".to_string(),
            description: None,
        };
        insert_governance_lane(&db, &lane, now).unwrap();

        // Create two nodes
        for i in 0..2 {
            upsert_governance_node(
                &db,
                &UpsertGovernanceNode {
                    node_id: format!("node-del-{}", i),
                    lane_id: "lane-3".to_string(),
                    state: Some("open".to_string()),
                    summary: None,
                    evidence_refs: None,
                    artifact_refs: None,
                    reported_git_head: None,
                    suggested_next: None,
                    forbidden_scope: None,
                    external_anchors: None,
                },
                now,
            )
            .unwrap();
        }
        assert_eq!(count_nodes_for_lane(&db, "lane-3").unwrap(), 2);

        // Delete nodes then lane
        let deleted_nodes = delete_governance_nodes_by_lane(&db, "lane-3").unwrap();
        assert_eq!(deleted_nodes, 2);

        let deleted_lane = delete_governance_lane(&db, "lane-3").unwrap();
        assert_eq!(deleted_lane, 1);

        // Verify empty
        assert!(get_governance_lane_by_id(&db, "lane-3").unwrap().is_none());
        assert_eq!(count_nodes_for_lane(&db, "lane-3").unwrap(), 0);

        // Global counters
        assert_eq!(count_all_active_lanes(&db).unwrap(), 0);
        assert_eq!(count_all_active_nodes(&db).unwrap(), 0);
        assert!(!has_governance_data(&db).unwrap());
    }

    #[test]
    fn get_governance_nodes_filters_by_node_id() {
        let db = test_db();
        let now = "2026-05-24T12:00:00Z";
        let lane = NewGovernanceLane {
            lane_id: "lane-filter".to_string(),
            title: "Filter Test".to_string(),
            description: None,
        };
        insert_governance_lane(&db, &lane, now).unwrap();

        for i in 0..3 {
            upsert_governance_node(
                &db,
                &UpsertGovernanceNode {
                    node_id: format!("node-f-{}", i),
                    lane_id: "lane-filter".to_string(),
                    state: Some("open".to_string()),
                    summary: None,
                    evidence_refs: None,
                    artifact_refs: None,
                    reported_git_head: None,
                    suggested_next: None,
                    forbidden_scope: None,
                    external_anchors: None,
                },
                now,
            )
            .unwrap();
        }

        // Filter by node_id
        let nodes = get_governance_nodes(&db, None, Some("node-f-1")).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_id, "node-f-1");

        // Filter by both lane_id and node_id
        let nodes = get_governance_nodes(&db, Some("lane-filter"), Some("node-f-2")).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_id, "node-f-2");
    }

    #[test]
    fn get_governance_nodes_returns_empty_for_no_match() {
        let db = test_db();
        let nodes = get_governance_nodes(&db, Some("nonexistent-lane"), None).unwrap();
        assert!(nodes.is_empty());

        let nodes = get_governance_nodes(&db, None, Some("nonexistent-node")).unwrap();
        assert!(nodes.is_empty());
    }

    #[test]
    fn get_governance_nodes_no_filter_returns_all() {
        let db = test_db();
        let now = "2026-05-24T12:00:00Z";
        let lane = NewGovernanceLane {
            lane_id: "lane-all".to_string(),
            title: "All Test".to_string(),
            description: None,
        };
        insert_governance_lane(&db, &lane, now).unwrap();

        for i in 0..2 {
            upsert_governance_node(
                &db,
                &UpsertGovernanceNode {
                    node_id: format!("node-all-{}", i),
                    lane_id: "lane-all".to_string(),
                    state: Some("open".to_string()),
                    summary: None,
                    evidence_refs: None,
                    artifact_refs: None,
                    reported_git_head: None,
                    suggested_next: None,
                    forbidden_scope: None,
                    external_anchors: None,
                },
                now,
            )
            .unwrap();
        }

        let nodes = get_governance_nodes(&db, None, None).unwrap();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn update_lane_status_changes_status() {
        let db = test_db();
        let now = "2026-05-24T12:00:00Z";
        let lane = NewGovernanceLane {
            lane_id: "lane-status".to_string(),
            title: "Status Test".to_string(),
            description: None,
        };
        insert_governance_lane(&db, &lane, now).unwrap();

        let affected =
            update_lane_status(&db, "lane-status", "complete", "2026-05-24T14:00:00Z").unwrap();
        assert_eq!(affected, 1);

        let found = get_governance_lane_by_id(&db, "lane-status")
            .unwrap()
            .unwrap();
        assert_eq!(found.status, "complete");
        assert_eq!(found.updated_at, "2026-05-24T14:00:00Z");
    }

    #[test]
    fn update_lane_status_nonexistent_affects_zero() {
        let db = test_db();
        let affected = update_lane_status(&db, "no-lane", "complete", "now").unwrap();
        assert_eq!(affected, 0);
    }

    #[test]
    fn count_active_nodes_excludes_closed_state() {
        let db = test_db();
        let now = "2026-05-24T12:00:00Z";
        let lane = NewGovernanceLane {
            lane_id: "lane-active".to_string(),
            title: "Active Test".to_string(),
            description: None,
        };
        insert_governance_lane(&db, &lane, now).unwrap();

        // Open node
        upsert_governance_node(
            &db,
            &UpsertGovernanceNode {
                node_id: "node-open".to_string(),
                lane_id: "lane-active".to_string(),
                state: Some("open".to_string()),
                summary: None,
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
            },
            now,
        )
        .unwrap();

        // Closed node
        upsert_governance_node(
            &db,
            &UpsertGovernanceNode {
                node_id: "node-closed".to_string(),
                lane_id: "lane-active".to_string(),
                state: Some("closed".to_string()),
                summary: None,
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
            },
            now,
        )
        .unwrap();

        assert_eq!(count_active_nodes_for_lane(&db, "lane-active").unwrap(), 1);
        assert_eq!(count_nodes_for_lane(&db, "lane-active").unwrap(), 2);
        assert_eq!(count_all_active_nodes(&db).unwrap(), 1);
    }

    #[test]
    fn count_all_active_lanes_counts_only_active() {
        let db = test_db();
        let now = "2026-05-24T12:00:00Z";

        for i in 0..3 {
            let lane = NewGovernanceLane {
                lane_id: format!("multi-lane-{}", i),
                title: format!("Lane {}", i),
                description: None,
            };
            insert_governance_lane(&db, &lane, now).unwrap();
        }
        assert_eq!(count_all_active_lanes(&db).unwrap(), 3);

        // Complete one lane
        update_lane_status(&db, "multi-lane-1", "complete", now).unwrap();
        assert_eq!(count_all_active_lanes(&db).unwrap(), 2);
    }

    #[test]
    fn upsert_governance_node_updates_all_optional_fields() {
        let db = test_db();
        let now = "2026-05-24T12:00:00Z";
        let lane = NewGovernanceLane {
            lane_id: "lane-fields".to_string(),
            title: "Fields Test".to_string(),
            description: None,
        };
        insert_governance_lane(&db, &lane, now).unwrap();

        // Insert with minimal fields
        upsert_governance_node(
            &db,
            &UpsertGovernanceNode {
                node_id: "node-fields".to_string(),
                lane_id: "lane-fields".to_string(),
                state: Some("open".to_string()),
                summary: None,
                evidence_refs: None,
                artifact_refs: None,
                reported_git_head: None,
                suggested_next: None,
                forbidden_scope: None,
                external_anchors: None,
            },
            now,
        )
        .unwrap();

        // Update with all optional fields
        upsert_governance_node(
            &db,
            &UpsertGovernanceNode {
                node_id: "node-fields".to_string(),
                lane_id: "lane-fields".to_string(),
                state: Some("done".to_string()),
                summary: Some("updated summary".to_string()),
                evidence_refs: Some("evidence-1".to_string()),
                artifact_refs: Some("artifact-1".to_string()),
                reported_git_head: Some("abc123".to_string()),
                suggested_next: Some("review".to_string()),
                forbidden_scope: Some("production".to_string()),
                external_anchors: Some("anchor-1".to_string()),
            },
            "2026-05-24T14:00:00Z",
        )
        .unwrap();

        let nodes = get_governance_nodes(&db, Some("lane-fields"), None).unwrap();
        assert_eq!(nodes.len(), 1);
        let n = &nodes[0];
        assert_eq!(n.state, "done");
        assert_eq!(n.summary, Some("updated summary".to_string()));
        assert_eq!(n.evidence_refs, Some("evidence-1".to_string()));
        assert_eq!(n.artifact_refs, Some("artifact-1".to_string()));
        assert_eq!(n.reported_git_head, Some("abc123".to_string()));
        assert_eq!(n.suggested_next, Some("review".to_string()));
        assert_eq!(n.forbidden_scope, Some("production".to_string()));
        assert_eq!(n.external_anchors, Some("anchor-1".to_string()));
    }

    #[test]
    fn delete_nonexistent_lane_and_nodes_is_safe() {
        let db = test_db();
        let deleted_nodes = delete_governance_nodes_by_lane(&db, "no-lane").unwrap();
        assert_eq!(deleted_nodes, 0);
        let deleted_lane = delete_governance_lane(&db, "no-lane").unwrap();
        assert_eq!(deleted_lane, 0);
    }

    #[test]
    fn insert_lane_without_description() {
        let db = test_db();
        let now = "2026-05-24T12:00:00Z";
        let lane = NewGovernanceLane {
            lane_id: "lane-nodesc".to_string(),
            title: "No Description".to_string(),
            description: None,
        };
        insert_governance_lane(&db, &lane, now).unwrap();
        let found = get_governance_lane_by_id(&db, "lane-nodesc")
            .unwrap()
            .unwrap();
        assert_eq!(found.description, None);
    }
}
