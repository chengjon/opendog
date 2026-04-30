use crate::error::Result;
use crate::storage::database::Database;
use rusqlite::params;

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
