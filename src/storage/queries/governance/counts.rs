use crate::error::Result;
use crate::storage::database::Database;
use rusqlite::params;

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
