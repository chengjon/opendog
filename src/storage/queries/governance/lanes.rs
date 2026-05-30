use super::types::{GovernanceLane, NewGovernanceLane};
use crate::error::Result;
use crate::storage::database::Database;
use rusqlite::params;

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

pub fn delete_governance_lane(db: &Database, lane_id: &str) -> Result<usize> {
    db.execute(
        "DELETE FROM governance_lanes WHERE lane_id = ?1",
        params![lane_id],
    )
}
