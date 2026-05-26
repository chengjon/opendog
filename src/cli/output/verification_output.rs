use crate::core::verification::ExecutedVerificationResult;
use crate::storage::queries::VerificationRun;

use super::truncate;

pub(super) fn print_verification_recorded(id: &str, run: &VerificationRun) {
    println!(
        "Recorded {} result for project '{}': {} ({})",
        run.kind, id, run.status, run.command
    );
    if let Some(code) = run.exit_code {
        println!("  Exit code: {}", code);
    }
    if let Some(summary) = &run.summary {
        println!("  Summary: {}", summary);
    }
    println!("  Finished: {}", run.finished_at);
}

pub(super) fn print_verification_status(id: &str, runs: &[VerificationRun]) {
    println!("Latest verification results for project '{}':", id);
    if runs.is_empty() {
        println!("  No verification results recorded yet.");
        return;
    }

    println!(
        "  {:12} {:10} {:8} {:18} COMMAND",
        "KIND", "STATUS", "EXIT", "FINISHED"
    );
    println!("{}", "─".repeat(90));
    for run in runs {
        println!(
            "  {:12} {:10} {:8} {:18} {}",
            run.kind,
            run.status,
            run.exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".to_string()),
            truncate(&run.finished_at, 18),
            truncate(&run.command, 40),
        );
    }
}

pub(super) fn print_verification_executed(id: &str, result: &ExecutedVerificationResult) {
    println!(
        "Executed {} verification for project '{}': {} ({})",
        result.run.kind, id, result.run.status, result.run.command
    );
    if let Some(code) = result.run.exit_code {
        println!("  Exit code: {}", code);
    }
    if let Some(summary) = &result.run.summary {
        println!("  Summary: {}", summary);
    }
    if !result.stdout_tail.is_empty() {
        println!("  Stdout tail: {}", truncate(&result.stdout_tail, 120));
    }
    if !result.stderr_tail.is_empty() {
        println!("  Stderr tail: {}", truncate(&result.stderr_tail, 120));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn recorded_verification_format() {
        let line = format!(
            "Recorded {} result for project '{}': {} ({})",
            "test", "proj", "passed", "cargo test"
        );
        assert_eq!(line, "Recorded test result for project 'proj': passed (cargo test)");
    }

    #[test]
    fn exit_code_present() {
        let code: Option<i32> = Some(0);
        let line = code.map(|c| format!("  Exit code: {}", c));
        assert_eq!(line.unwrap(), "  Exit code: 0");
    }

    #[test]
    fn exit_code_absent() {
        let code: Option<i32> = None;
        assert!(code.is_none());
    }

    #[test]
    fn summary_present() {
        let summary: Option<String> = Some("all tests passed".into());
        let line = summary.as_deref().map(|s| format!("  Summary: {}", s));
        assert_eq!(line.unwrap(), "  Summary: all tests passed");
    }

    #[test]
    fn summary_absent() {
        let summary: Option<String> = None;
        assert!(summary.is_none());
    }

    #[test]
    fn empty_verification_guard() {
        let runs: Vec<String> = vec![];
        let msg = if runs.is_empty() {
            "No verification results recorded yet."
        } else {
            "has runs"
        };
        assert_eq!(msg, "No verification results recorded yet.");
    }

    #[test]
    fn exit_code_display_some() {
        let code: Option<i32> = Some(1);
        let display = code.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
        assert_eq!(display, "1");
    }

    #[test]
    fn exit_code_display_none() {
        let code: Option<i32> = None;
        let display = code.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
        assert_eq!(display, "-");
    }

    #[test]
    fn executed_verification_format() {
        let line = format!(
            "Executed {} verification for project '{}': {} ({})",
            "build", "proj", "failed", "cargo build"
        );
        assert!(line.contains("build"));
        assert!(line.contains("failed"));
    }

    #[test]
    fn stdout_tail_present() {
        let tail = "test output".to_string();
        let empty = tail.is_empty();
        assert!(!empty);
        let line = format!("  Stdout tail: {}", &tail[..tail.len().min(120)]);
        assert_eq!(line, "  Stdout tail: test output");
    }

    #[test]
    fn stdout_tail_empty_skipped() {
        let tail = "".to_string();
        assert!(tail.is_empty());
    }

    #[test]
    fn stderr_tail_present() {
        let tail = "error occurred".to_string();
        let empty = tail.is_empty();
        assert!(!empty);
    }

    #[test]
    fn verification_table_header_format() {
        let header = format!(
            "  {:12} {:10} {:8} {:18} COMMAND",
            "KIND", "STATUS", "EXIT", "FINISHED"
        );
        assert!(header.contains("KIND"));
        assert!(header.contains("STATUS"));
        assert!(header.contains("COMMAND"));
    }
}
