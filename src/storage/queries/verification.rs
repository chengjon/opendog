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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("verification_test.db");
        let db = Database::open_project(&db_path).unwrap();
        Box::leak(Box::new(dir));
        db
    }

    fn insert_run(db: &Database, kind: &str, status: &str, command: &str, finished_at: &str) {
        insert_verification_run(
            db,
            &NewVerificationRun {
                kind: kind.to_string(),
                status: status.to_string(),
                command: command.to_string(),
                exit_code: None,
                summary: None,
                source: "test".to_string(),
                started_at: None,
                finished_at: finished_at.to_string(),
            },
        )
        .unwrap();
    }

    #[test]
    fn insert_and_read_verification_run() {
        let db = test_db();
        insert_run(&db, "test", "passed", "cargo test", "1000");
        let runs = get_latest_verification_runs(&db).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].kind, "test");
        assert_eq!(runs[0].status, "passed");
        assert_eq!(runs[0].command, "cargo test");
    }

    #[test]
    fn latest_returns_most_recent_per_kind() {
        let db = test_db();
        insert_run(&db, "test", "passed", "cargo test v1", "1000");
        insert_run(&db, "test", "failed", "cargo test v2", "2000");
        insert_run(&db, "lint", "passed", "cargo clippy", "1500");
        let runs = get_latest_verification_runs(&db).unwrap();
        assert_eq!(runs.len(), 2);
        let test_run = runs.iter().find(|r| r.kind == "test").unwrap();
        assert_eq!(test_run.status, "failed"); // latest
        assert_eq!(test_run.finished_at, "2000");
        let lint_run = runs.iter().find(|r| r.kind == "lint").unwrap();
        assert_eq!(lint_run.status, "passed");
    }
}
