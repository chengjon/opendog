use crate::error::Result;
use crate::storage::database::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityRollupCounts {
    pub file_sightings: i64,
    pub file_events: i64,
}

pub fn count_file_sightings(db: &Database) -> Result<i64> {
    db.query_row("SELECT COUNT(*) FROM file_sightings", params![], |row| {
        row.get(0)
    })
}

pub fn count_file_sightings_before(db: &Database, cutoff_ts: i64) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*) FROM file_sightings WHERE CAST(seen_at AS INTEGER) < ?1",
        params![cutoff_ts],
        |row| row.get(0),
    )
}

pub fn delete_file_sightings_before(db: &Database, cutoff_ts: i64) -> Result<usize> {
    db.execute(
        "DELETE FROM file_sightings WHERE CAST(seen_at AS INTEGER) < ?1",
        params![cutoff_ts],
    )
}

pub fn count_file_events(db: &Database) -> Result<i64> {
    db.query_row("SELECT COUNT(*) FROM file_events", params![], |row| {
        row.get(0)
    })
}

pub fn count_file_events_before(db: &Database, cutoff_ts: i64) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*) FROM file_events WHERE CAST(event_time AS INTEGER) < ?1",
        params![cutoff_ts],
        |row| row.get(0),
    )
}

pub fn delete_file_events_before(db: &Database, cutoff_ts: i64) -> Result<usize> {
    db.execute(
        "DELETE FROM file_events WHERE CAST(event_time AS INTEGER) < ?1",
        params![cutoff_ts],
    )
}

pub fn rollup_file_activity_before(
    db: &Database,
    cutoff_ts: i64,
    updated_at: &str,
) -> Result<ActivityRollupCounts> {
    let counts = ActivityRollupCounts {
        file_sightings: count_unrolled_file_sightings_before(db, cutoff_ts)?,
        file_events: count_unrolled_file_events_before(db, cutoff_ts)?,
    };
    if counts.file_sightings > 0 {
        rollup_file_sightings_before(db, cutoff_ts, updated_at)?;
    }
    if counts.file_events > 0 {
        rollup_file_events_before(db, cutoff_ts, updated_at)?;
    }
    Ok(counts)
}

fn count_unrolled_file_sightings_before(db: &Database, cutoff_ts: i64) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*)
         FROM file_sightings AS s
         LEFT JOIN activity_daily_rollups AS r
           ON r.day_start = (CAST(CAST(s.seen_at AS INTEGER) / 86400 AS INTEGER) * 86400)
          AND r.source_table = 'file_sightings'
          AND r.activity = 'seen'
         WHERE CAST(s.seen_at AS INTEGER) < ?1
           AND s.id > COALESCE(r.max_source_id, 0)",
        params![cutoff_ts],
        |row| row.get(0),
    )
}

fn count_unrolled_file_events_before(db: &Database, cutoff_ts: i64) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*)
         FROM file_events AS e
         LEFT JOIN activity_daily_rollups AS r
           ON r.day_start = (CAST(CAST(e.event_time AS INTEGER) / 86400 AS INTEGER) * 86400)
          AND r.source_table = 'file_events'
          AND r.activity = e.event_type
         WHERE CAST(e.event_time AS INTEGER) < ?1
           AND e.id > COALESCE(r.max_source_id, 0)",
        params![cutoff_ts],
        |row| row.get(0),
    )
}

fn rollup_file_sightings_before(db: &Database, cutoff_ts: i64, updated_at: &str) -> Result<usize> {
    db.execute(
        "INSERT INTO activity_daily_rollups (
             day_start, source_table, activity, row_count, max_source_id, updated_at
         )
         SELECT
             day_start,
             'file_sightings',
             'seen',
             COUNT(*),
             MAX(id),
             ?2
         FROM (
             SELECT
                 s.id,
                 (CAST(CAST(s.seen_at AS INTEGER) / 86400 AS INTEGER) * 86400) AS day_start
             FROM file_sightings AS s
             LEFT JOIN activity_daily_rollups AS r
               ON r.day_start = (CAST(CAST(s.seen_at AS INTEGER) / 86400 AS INTEGER) * 86400)
              AND r.source_table = 'file_sightings'
              AND r.activity = 'seen'
             WHERE CAST(s.seen_at AS INTEGER) < ?1
               AND s.id > COALESCE(r.max_source_id, 0)
         ) AS pending
         WHERE true
         GROUP BY day_start
         ON CONFLICT(day_start, source_table, activity) DO UPDATE SET
             row_count = activity_daily_rollups.row_count + excluded.row_count,
             max_source_id = MAX(activity_daily_rollups.max_source_id, excluded.max_source_id),
             updated_at = excluded.updated_at",
        params![cutoff_ts, updated_at],
    )
}

fn rollup_file_events_before(db: &Database, cutoff_ts: i64, updated_at: &str) -> Result<usize> {
    db.execute(
        "INSERT INTO activity_daily_rollups (
             day_start, source_table, activity, row_count, max_source_id, updated_at
         )
         SELECT
             day_start,
             'file_events',
             activity,
             COUNT(*),
             MAX(id),
             ?2
         FROM (
             SELECT
                 e.id,
                 e.event_type AS activity,
                 (CAST(CAST(e.event_time AS INTEGER) / 86400 AS INTEGER) * 86400) AS day_start
             FROM file_events AS e
             LEFT JOIN activity_daily_rollups AS r
               ON r.day_start = (CAST(CAST(e.event_time AS INTEGER) / 86400 AS INTEGER) * 86400)
              AND r.source_table = 'file_events'
              AND r.activity = e.event_type
             WHERE CAST(e.event_time AS INTEGER) < ?1
               AND e.id > COALESCE(r.max_source_id, 0)
         ) AS pending
         WHERE true
         GROUP BY day_start, activity
         ON CONFLICT(day_start, source_table, activity) DO UPDATE SET
             row_count = activity_daily_rollups.row_count + excluded.row_count,
             max_source_id = MAX(activity_daily_rollups.max_source_id, excluded.max_source_id),
             updated_at = excluded.updated_at",
        params![cutoff_ts, updated_at],
    )
}

pub fn count_activity_daily_rollups(db: &Database) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*) FROM activity_daily_rollups",
        params![],
        |row| row.get(0),
    )
}
