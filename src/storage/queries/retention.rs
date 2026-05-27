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

pub fn count_verification_runs(db: &Database) -> Result<i64> {
    db.query_row("SELECT COUNT(*) FROM verification_runs", params![], |row| {
        row.get(0)
    })
}

pub fn count_activity_daily_rollups(db: &Database) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*) FROM activity_daily_rollups",
        params![],
        |row| row.get(0),
    )
}

pub fn count_verification_runs_before(db: &Database, cutoff_ts: i64) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*) FROM verification_runs WHERE CAST(finished_at AS INTEGER) < ?1",
        params![cutoff_ts],
        |row| row.get(0),
    )
}

pub fn delete_verification_runs_before(db: &Database, cutoff_ts: i64) -> Result<usize> {
    db.execute(
        "DELETE FROM verification_runs WHERE CAST(finished_at AS INTEGER) < ?1",
        params![cutoff_ts],
    )
}

pub fn list_snapshot_run_ids_to_prune(db: &Database, keep_latest: usize) -> Result<Vec<i64>> {
    db.prepare_and_query(
        "SELECT id
         FROM snapshot_runs
         ORDER BY CAST(captured_at AS INTEGER) DESC, id DESC
         LIMIT -1 OFFSET ?1",
        params![keep_latest as i64],
        |row| row.get(0),
    )
}

pub fn count_snapshot_history_for_runs(db: &Database, run_ids: &[i64]) -> Result<i64> {
    if run_ids.is_empty() {
        return Ok(0);
    }
    let (sql, params_vec) = in_clause_sql(
        "SELECT COUNT(*) FROM snapshot_history WHERE run_id IN ({})",
        run_ids,
    );
    db.query_row(&sql, rusqlite::params_from_iter(params_vec), |row| {
        row.get(0)
    })
}

pub fn delete_snapshot_history_for_runs(db: &Database, run_ids: &[i64]) -> Result<usize> {
    if run_ids.is_empty() {
        return Ok(0);
    }
    let (sql, params_vec) =
        in_clause_sql("DELETE FROM snapshot_history WHERE run_id IN ({})", run_ids);
    Ok(db
        .conn()
        .execute(&sql, rusqlite::params_from_iter(params_vec))?)
}

pub fn delete_snapshot_runs_by_ids(db: &Database, run_ids: &[i64]) -> Result<usize> {
    if run_ids.is_empty() {
        return Ok(0);
    }
    let (sql, params_vec) = in_clause_sql("DELETE FROM snapshot_runs WHERE id IN ({})", run_ids);
    Ok(db
        .conn()
        .execute(&sql, rusqlite::params_from_iter(params_vec))?)
}

fn in_clause_sql(prefix: &str, ids: &[i64]) -> (String, Vec<i64>) {
    let placeholders: Vec<String> = (1..=ids.len()).map(|index| format!("?{}", index)).collect();
    (prefix.replace("{}", &placeholders.join(", ")), ids.to_vec())
}

pub fn count_snapshot_runs(db: &Database) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*) FROM snapshot_runs",
        rusqlite::params![],
        |row| row.get(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;
    use rusqlite::params;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("retention_test.db");
        let db = Database::open_project(&db_path).unwrap();
        Box::leak(Box::new(dir));
        db
    }

    #[test]
    fn count_and_delete_file_sightings_by_cutoff() {
        let db = test_db();
        let cutoff = 5000i64;
        db.execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES ('a.rs', 'codex', 1, '4000')",
            params![],
        )
        .unwrap();
        db.execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES ('b.rs', 'codex', 2, '6000')",
            params![],
        )
        .unwrap();
        assert_eq!(count_file_sightings_before(&db, cutoff).unwrap(), 1);
        assert_eq!(delete_file_sightings_before(&db, cutoff).unwrap(), 1);
        assert_eq!(count_file_sightings_before(&db, cutoff).unwrap(), 0);
    }

    #[test]
    fn count_and_delete_file_events_by_cutoff() {
        let db = test_db();
        db.execute(
            "INSERT INTO file_events (file_path, event_type, event_time) VALUES ('a.rs', 'modify', '100')",
            params![],
        )
        .unwrap();
        db.execute(
            "INSERT INTO file_events (file_path, event_type, event_time) VALUES ('b.rs', 'modify', '900')",
            params![],
        )
        .unwrap();
        assert_eq!(count_file_events_before(&db, 500).unwrap(), 1);
        assert_eq!(delete_file_events_before(&db, 500).unwrap(), 1);
    }

    #[test]
    fn count_and_delete_verification_runs_by_cutoff() {
        let db = test_db();
        db.execute(
            "INSERT INTO verification_runs (kind, status, command, source, finished_at) VALUES ('test', 'passed', 'c', 't', '100')",
            params![],
        )
        .unwrap();
        db.execute(
            "INSERT INTO verification_runs (kind, status, command, source, finished_at) VALUES ('lint', 'passed', 'c', 't', '900')",
            params![],
        )
        .unwrap();
        assert_eq!(count_verification_runs_before(&db, 500).unwrap(), 1);
        assert_eq!(delete_verification_runs_before(&db, 500).unwrap(), 1);
    }

    #[test]
    fn snapshot_run_pruning_keeps_latest() {
        let db = test_db();
        for ts in ["100", "200", "300"] {
            db.execute(
                "INSERT INTO snapshot_runs (captured_at, file_count) VALUES (?1, 1)",
                params![ts],
            )
            .unwrap();
        }
        // Keep latest 1, prune the rest
        let to_prune = list_snapshot_run_ids_to_prune(&db, 1).unwrap();
        assert_eq!(to_prune.len(), 2);

        // Count and delete history
        let history_count = count_snapshot_history_for_runs(&db, &to_prune).unwrap();
        assert_eq!(history_count, 0); // no history rows seeded

        let deleted_runs = delete_snapshot_runs_by_ids(&db, &to_prune).unwrap();
        assert_eq!(deleted_runs, 2);
    }

    #[test]
    fn empty_run_ids_is_noop() {
        let db = test_db();
        assert_eq!(count_snapshot_history_for_runs(&db, &[]).unwrap(), 0);
        assert_eq!(delete_snapshot_history_for_runs(&db, &[]).unwrap(), 0);
        assert_eq!(delete_snapshot_runs_by_ids(&db, &[]).unwrap(), 0);
    }

    #[test]
    fn in_clause_sql_generates_placeholders() {
        let (sql, ids) = in_clause_sql("SELECT * FROM t WHERE id IN ({})", &[10, 20, 30]);
        assert_eq!(sql, "SELECT * FROM t WHERE id IN (?1, ?2, ?3)");
        assert_eq!(ids, vec![10, 20, 30]);
    }

    #[test]
    fn count_snapshot_runs_returns_total_count() {
        let db = test_db();
        assert_eq!(count_snapshot_runs(&db).unwrap(), 0);
        for ts in ["100", "200", "300"] {
            db.execute(
                "INSERT INTO snapshot_runs (captured_at, file_count) VALUES (?1, 1)",
                params![ts],
            )
            .unwrap();
        }
        assert_eq!(count_snapshot_runs(&db).unwrap(), 3);
    }

    #[test]
    fn count_activity_daily_rollups_returns_total_count() {
        let db = test_db();
        assert_eq!(count_activity_daily_rollups(&db).unwrap(), 0);
        db.execute(
            "INSERT INTO activity_daily_rollups
             (day_start, source_table, activity, row_count, max_source_id, updated_at)
             VALUES (86400, 'file_events', 'modify', 2, 10, '200')",
            params![],
        )
        .unwrap();
        assert_eq!(count_activity_daily_rollups(&db).unwrap(), 1);
    }

    #[test]
    fn rollup_file_activity_before_preserves_daily_counts_idempotently() {
        let db = test_db();
        let cutoff = 200_000i64;
        let first_day = 86_400i64;
        db.execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES ('a.rs', 'codex', 1, ?1)",
            params![(first_day + 1).to_string()],
        )
        .unwrap();
        db.execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES ('b.rs', 'codex', 2, ?1)",
            params![(first_day + 2).to_string()],
        )
        .unwrap();
        db.execute(
            "INSERT INTO file_events (file_path, event_type, event_time) VALUES ('a.rs', 'modify', ?1)",
            params![(first_day + 3).to_string()],
        )
        .unwrap();
        db.execute(
            "INSERT INTO file_events (file_path, event_type, event_time) VALUES ('new.rs', 'modify', ?1)",
            params![(cutoff + 1).to_string()],
        )
        .unwrap();

        let counts = rollup_file_activity_before(&db, cutoff, "200000").unwrap();

        assert_eq!(counts.file_sightings, 2);
        assert_eq!(counts.file_events, 1);
        let sighting_rollup: i64 = db
            .query_row(
                "SELECT row_count FROM activity_daily_rollups
                 WHERE day_start = ?1 AND source_table = 'file_sightings' AND activity = 'seen'",
                params![first_day],
                |row| row.get(0),
            )
            .unwrap();
        let modify_rollup: i64 = db
            .query_row(
                "SELECT row_count FROM activity_daily_rollups
                 WHERE day_start = ?1 AND source_table = 'file_events' AND activity = 'modify'",
                params![first_day],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(sighting_rollup, 2);
        assert_eq!(modify_rollup, 1);

        let second = rollup_file_activity_before(&db, cutoff, "200001").unwrap();
        assert_eq!(second, ActivityRollupCounts::default());
    }
}
