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
    let patterns = [" | ", "&& ", "|| ", "2>/dev/null", "> /dev/null"];
    patterns.iter().any(|p| command.contains(p))
}

fn detect_suspicious_pass_signals(stdout_tail: &str, stderr_tail: &str) -> Vec<String> {
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

    #[test]
    fn record_verification_rejects_invalid_kind() {
        let db = test_db();
        let err = record_verification_result(
            &db,
            RecordVerificationInput {
                kind: "deploy".to_string(),
                status: "passed".to_string(),
                command: "true".to_string(),
                exit_code: Some(0),
                summary: None,
                source: "mcp".to_string(),
                started_at: None,
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("kind must be one of: test, lint, build"));
        assert!(err.to_string().contains("deploy"));
    }

    #[test]
    fn execute_verification_rejects_invalid_kind() {
        let db = test_db();
        let dir = tempfile::tempdir().unwrap();
        let err = execute_verification_command(
            &db,
            dir.path(),
            ExecuteVerificationInput {
                kind: "security".to_string(),
                command: "true".to_string(),
                source: "mcp".to_string(),
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("kind must be one of: test, lint, build"));
    }

    #[test]
    fn execute_verification_records_failure_status_on_nonzero_exit() {
        let db = test_db();
        let dir = tempfile::tempdir().unwrap();
        let result = execute_verification_command(
            &db,
            dir.path(),
            ExecuteVerificationInput {
                kind: "lint".to_string(),
                command: "exit 1".to_string(),
                source: "mcp".to_string(),
            },
        )
        .unwrap();

        assert_eq!(result.run.status, "failed");
        assert_eq!(result.run.exit_code, Some(1));
    }

    // --- truncate_tail tests ---

    #[test]
    fn truncate_tail_empty_input_returns_empty_string() {
        assert_eq!(truncate_tail(b"", 100), "");
    }

    #[test]
    fn truncate_tail_short_input_within_limit_returns_trimmed_input() {
        assert_eq!(truncate_tail(b"hello world", 100), "hello world");
    }

    #[test]
    fn truncate_tail_long_input_truncates_to_max_chars_from_end() {
        let input = "abcdefghijklmnopqrstuvwxyz".as_bytes();
        assert_eq!(truncate_tail(input, 5), "vwxyz");
    }

    #[test]
    fn truncate_tail_trims_whitespace_before_truncation() {
        assert_eq!(truncate_tail(b"  hello  ", 100), "hello");
    }

    #[test]
    fn truncate_tail_handles_non_utf8_gracefully() {
        // Invalid UTF-8 sequence: 0x80 is a continuation byte without a leading byte.
        let input = &[0x80, 0x80];
        let result = truncate_tail(input, 100);
        // String::from_utf8_lossy replaces invalid sequences with the replacement char.
        assert!(result.contains('\u{fffd}'));
    }

    #[test]
    fn truncate_tail_exactly_max_chars_returned_in_full() {
        let input = "abcde".as_bytes();
        assert_eq!(truncate_tail(input, 5), "abcde");
    }

    // --- summarize_execution tests ---

    #[test]
    fn summarize_execution_stderr_present_returns_last_nonempty_line() {
        let result = summarize_execution("stdout line\n", "err line A\nerr line B\n", false);
        assert_eq!(result, Some("err line B".to_string()));
    }

    #[test]
    fn summarize_execution_no_stderr_stdout_present_returns_last_nonempty_line() {
        let result = summarize_execution("out A\nout B\n", "", true);
        assert_eq!(result, Some("out B".to_string()));
    }

    #[test]
    fn summarize_execution_empty_both_success_returns_succeeded_message() {
        let result = summarize_execution("", "", true);
        assert_eq!(result, Some("Verification command succeeded.".to_string()));
    }

    #[test]
    fn summarize_execution_empty_both_failure_returns_failed_message() {
        let result = summarize_execution("", "", false);
        assert_eq!(result, Some("Verification command failed.".to_string()));
    }

    #[test]
    fn summarize_execution_trailing_empty_lines_ignored() {
        let result = summarize_execution("line1\n\n\n", "", true);
        assert_eq!(result, Some("line1".to_string()));
    }

    #[test]
    fn summarize_execution_single_nonempty_line_in_stderr() {
        let result = summarize_execution("stdout stuff\n", "only err line\n", false);
        assert_eq!(result, Some("only err line".to_string()));
    }

    // --- validate_kind tests ---

    #[test]
    fn validate_kind_test_is_valid() {
        assert!(validate_kind("test").is_ok());
    }

    #[test]
    fn validate_kind_lint_is_valid() {
        assert!(validate_kind("lint").is_ok());
    }

    #[test]
    fn validate_kind_build_is_valid() {
        assert!(validate_kind("build").is_ok());
    }

    #[test]
    fn validate_kind_deploy_is_rejected() {
        let err = validate_kind("deploy").unwrap_err();
        assert!(err.to_string().contains("kind must be one of: test, lint, build"));
        assert!(err.to_string().contains("deploy"));
    }

    #[test]
    fn validate_kind_empty_string_is_rejected() {
        let err = validate_kind("").unwrap_err();
        assert!(err.to_string().contains("kind must be one of: test, lint, build"));
    }

    #[test]
    fn validate_kind_uppercase_is_rejected() {
        let err = validate_kind("TEST").unwrap_err();
        assert!(err.to_string().contains("TEST"));
    }

    // --- pipeline detection tests ---

    #[test]
    fn detect_pipeline_operators_finds_pipe() {
        assert!(command_contains_pipeline_operators("npx vue-tsc --noEmit 2>&1 | tail -30"));
    }

    #[test]
    fn detect_pipeline_operators_finds_double_ampersand() {
        assert!(command_contains_pipeline_operators("cargo test && echo ok"));
    }

    #[test]
    fn detect_pipeline_operators_finds_double_pipe() {
        assert!(command_contains_pipeline_operators("cargo test || true"));
    }

    #[test]
    fn detect_pipeline_operators_finds_redirect_to_dev_null() {
        assert!(command_contains_pipeline_operators("cargo test 2>/dev/null"));
    }

    #[test]
    fn detect_pipeline_operators_clean_command_returns_false() {
        assert!(!command_contains_pipeline_operators("cargo test"));
        assert!(!command_contains_pipeline_operators("npx vue-tsc --noEmit"));
        assert!(!command_contains_pipeline_operators("pytest --co -q"));
    }

    // --- suspicious pass signal tests ---

    #[test]
    fn detect_suspicious_pass_signals_error_ts() {
        let signals = detect_suspicious_pass_signals(
            "src/App.vue(10,5): error TS2304: Cannot find name 'NonBlankString'",
            "",
        );
        assert!(signals.iter().any(|s| s.contains("TypeScript error")));
    }

    #[test]
    fn detect_suspicious_pass_signals_traceback() {
        let signals = detect_suspicious_pass_signals("", "Traceback (most recent call last):");
        assert!(signals.iter().any(|s| s.contains("traceback")));
    }

    #[test]
    fn detect_suspicious_pass_signals_failed() {
        let signals = detect_suspicious_pass_signals("3 tests FAILED out of 10", "");
        assert!(signals.iter().any(|s| s.contains("FAILED keyword")));
    }

    #[test]
    fn detect_suspicious_pass_signals_clean_output() {
        let signals = detect_suspicious_pass_signals("all tests passed", "");
        assert!(signals.is_empty());
    }
}
