use crate::error::Result;
use crate::storage::database::Database;
use rusqlite::params;

pub fn count_verification_runs(db: &Database) -> Result<i64> {
    db.query_row("SELECT COUNT(*) FROM verification_runs", params![], |row| {
        row.get(0)
    })
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
