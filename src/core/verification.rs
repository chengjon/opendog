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
    pub pipeline_operators_detected: bool,
    pub suspicious_pass_signals: Vec<String>,
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
    // Decode only the tail portion to avoid allocating the full string for large outputs.
    let cow = String::from_utf8_lossy(text);
    let trimmed = cow.trim();

    // Advance to the start of the tail without collecting all chars.
    let total_chars = trimmed.chars().count();
    if total_chars <= max_chars {
        return trimmed.to_string();
    }

    let skip = total_chars - max_chars;
    trimmed.chars().skip(skip).collect()
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

pub fn command_contains_pipeline_operators(command: &str) -> bool {
    let patterns = ["|", "&&", "||", "2>/dev/null", "> /dev/null", ">/dev/null"];
    patterns.iter().any(|p| command.contains(p))
}

pub fn detect_suspicious_pass_signals(stdout_tail: &str, stderr_tail: &str) -> Vec<String> {
    let error_patterns = [
        ("error TS", "TypeScript error in passed output"),
        ("FAILED", "FAILED keyword in passed output"),
        ("Traceback", "Python traceback in passed output"),
        ("Error:", "Error: keyword in passed output"),
        ("panic!", "Rust panic in passed output"),
    ];
    let combined = format!("{}\n{}", stdout_tail, stderr_tail);
    let combined_lower = combined.to_ascii_lowercase();
    let mut signals = Vec::new();
    for (pattern, label) in &error_patterns {
        if combined_lower.contains(&pattern.to_ascii_lowercase()) {
            signals.push(label.to_string());
        }
    }
    signals
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
        .ok_or_else(|| OpenDogError::VerificationRecordMissing(kind.to_string()))?;
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
    let pipeline_operators_detected = command_contains_pipeline_operators(&input.command);
    let suspicious_pass_signals = if success {
        detect_suspicious_pass_signals(&stdout_tail, &stderr_tail)
    } else {
        Vec::new()
    };
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
        pipeline_operators_detected,
        suspicious_pass_signals,
    })
}

#[cfg(test)]
mod tests;
