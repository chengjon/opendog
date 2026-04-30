use crate::error::{OpenDogError, Result};
use crate::storage::database::Database;
use crate::storage::queries::{self, NewVerificationRun, VerificationRun};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordVerificationInput {
    pub kind: String,
    pub status: String,
    pub command: String,
    pub exit_code: Option<i64>,
    pub summary: Option<String>,
    pub source: String,
    pub started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteVerificationInput {
    pub kind: String,
    pub command: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutedVerificationResult {
    pub run: VerificationRun,
    pub stdout_tail: String,
    pub stderr_tail: String,
}

fn now_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    now.as_secs().to_string()
}

fn validate_kind(kind: &str) -> Result<()> {
    match kind {
        "test" | "lint" | "build" => Ok(()),
        _ => Err(OpenDogError::InvalidVerification(format!(
            "kind must be one of: test, lint, build; got '{}'",
            kind
        ))),
    }
}

fn truncate_tail(text: &[u8], max_chars: usize) -> String {
    let rendered = String::from_utf8_lossy(text).trim().to_string();
    let chars: Vec<char> = rendered.chars().collect();
    if chars.len() <= max_chars {
        rendered
    } else {
        chars[chars.len() - max_chars..].iter().collect()
    }
}

fn summarize_execution(stdout_tail: &str, stderr_tail: &str, success: bool) -> Option<String> {
    let source = if !stderr_tail.is_empty() {
        stderr_tail
    } else {
        stdout_tail
    };
    let line = source
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string());
    match (success, line) {
        (_, Some(line)) if !line.is_empty() => Some(line),
        (_, Some(_)) => None,
        (true, None) => Some("Verification command succeeded.".to_string()),
        (false, None) => Some("Verification command failed.".to_string()),
    }
}

pub fn record_verification_result(
    db: &Database,
    input: RecordVerificationInput,
) -> Result<VerificationRun> {
    validate_kind(&input.kind)?;
    record_verification_result_at(db, input, now_timestamp())
}

fn record_verification_result_at(
    db: &Database,
    input: RecordVerificationInput,
    finished_at: String,
) -> Result<VerificationRun> {
    let kind = input.kind.clone();
    let run = NewVerificationRun {
        kind: input.kind,
        status: input.status,
        command: input.command,
        exit_code: input.exit_code,
        summary: input.summary,
        source: input.source,
        started_at: input.started_at,
        finished_at,
    };
    queries::insert_verification_run(db, &run)?;

    let mut latest = queries::get_latest_verification_runs(db)?;
    let recorded = latest
        .drain(..)
        .find(|r| r.kind == kind)
        .expect("recorded verification run should be queryable immediately");
    Ok(recorded)
}

pub fn get_latest_verification_runs(db: &Database) -> Result<Vec<VerificationRun>> {
    queries::get_latest_verification_runs(db)
}

pub fn execute_verification_command(
    db: &Database,
    root: &Path,
    input: ExecuteVerificationInput,
) -> Result<ExecutedVerificationResult> {
    validate_kind(&input.kind)?;

    let started_at = now_timestamp();
    #[cfg(unix)]
    let output = Command::new("sh")
        .args(["-lc", &input.command])
        .current_dir(root)
        .output()?;

    #[cfg(not(unix))]
    let output = Command::new("cmd")
        .args(["/C", &input.command])
        .current_dir(root)
        .output()?;

    let success = output.status.success();
    let stdout_tail = truncate_tail(&output.stdout, 800);
    let stderr_tail = truncate_tail(&output.stderr, 800);
    let run = record_verification_result_at(
        db,
        RecordVerificationInput {
            kind: input.kind,
            status: if success {
                "passed".to_string()
            } else {
                "failed".to_string()
            },
            command: input.command,
            exit_code: output.status.code().map(i64::from),
            summary: summarize_execution(&stdout_tail, &stderr_tail, success),
            source: input.source,
            started_at: Some(started_at),
        },
        now_timestamp(),
    )?;

    Ok(ExecutedVerificationResult {
        run,
        stdout_tail,
        stderr_tail,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("verification.db");
        let db = Database::open_project(&db_path).unwrap();
        Box::leak(Box::new(dir));
        db
    }

    #[test]
    fn records_and_reads_latest_verification_runs() {
        let db = test_db();
        let first = record_verification_result(
            &db,
            RecordVerificationInput {
                kind: "test".to_string(),
                status: "passed".to_string(),
                command: "cargo test".to_string(),
                exit_code: Some(0),
                summary: Some("all good".to_string()),
                source: "cli".to_string(),
                started_at: None,
            },
        )
        .unwrap();

        assert_eq!(first.kind, "test");
        assert_eq!(first.status, "passed");

        let latest = get_latest_verification_runs(&db).unwrap();
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].command, "cargo test");
    }

    #[test]
    fn executes_verification_command_and_records_result() {
        let db = test_db();
        let dir = tempfile::tempdir().unwrap();

        let result = execute_verification_command(
            &db,
            dir.path(),
            ExecuteVerificationInput {
                kind: "test".to_string(),
                command: "printf success-output".to_string(),
                source: "cli".to_string(),
            },
        )
        .unwrap();

        assert_eq!(result.run.status, "passed");
        assert!(result.stdout_tail.contains("success-output"));
    }
}
