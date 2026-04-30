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
