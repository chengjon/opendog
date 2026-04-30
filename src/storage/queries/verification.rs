use crate::error::Result;
use crate::storage::database::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRun {
    pub id: i64,
    pub kind: String,
    pub status: String,
    pub command: String,
    pub exit_code: Option<i64>,
    pub summary: Option<String>,
    pub source: String,
    pub started_at: Option<String>,
    pub finished_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewVerificationRun {
    pub kind: String,
    pub status: String,
    pub command: String,
    pub exit_code: Option<i64>,
    pub summary: Option<String>,
    pub source: String,
    pub started_at: Option<String>,
    pub finished_at: String,
}

pub fn insert_verification_run(db: &Database, run: &NewVerificationRun) -> Result<()> {
    db.execute(
        "INSERT INTO verification_runs (kind, status, command, exit_code, summary, source, started_at, finished_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            run.kind,
            run.status,
            run.command,
            run.exit_code,
            run.summary,
            run.source,
            run.started_at,
            run.finished_at
        ],
    )?;
    Ok(())
}

pub fn get_latest_verification_runs(db: &Database) -> Result<Vec<VerificationRun>> {
    db.prepare_and_query(
        "SELECT vr.id, vr.kind, vr.status, vr.command, vr.exit_code, vr.summary, vr.source, vr.started_at, vr.finished_at
         FROM verification_runs vr
         JOIN (
             SELECT kind, MAX(finished_at) AS max_finished_at
             FROM verification_runs
             GROUP BY kind
         ) latest ON latest.kind = vr.kind AND latest.max_finished_at = vr.finished_at
         ORDER BY vr.finished_at DESC, vr.kind",
        params![],
        |row| {
            Ok(VerificationRun {
                id: row.get(0)?,
                kind: row.get(1)?,
                status: row.get(2)?,
                command: row.get(3)?,
                exit_code: row.get(4)?,
                summary: row.get(5)?,
                source: row.get(6)?,
                started_at: row.get(7)?,
                finished_at: row.get(8)?,
            })
        },
    )
}
