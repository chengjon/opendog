use crate::core::report::{SnapshotComparison, TimeWindowReport, UsageTrendReport};

use super::truncate;

pub(super) fn print_time_window_report(id: &str, report: &TimeWindowReport) {
    println!(
        "Project '{}' — window={} sightings={} files={} processes={} modifications={}",
        id,
        report.window,
        report.summary.total_sightings,
        report.summary.unique_files_accessed,
        report.summary.unique_processes,
        report.summary.modification_events,
    );
    println!("  Range: {} .. {}", report.start_time, report.end_time);
    println!();

    if report.files.is_empty() {
        println!("  No activity recorded in this window.");
        return;
    }

    println!(
        "  {:40} {:>8} {:>8} {:>12} LAST MODIFY",
        "PATH", "ACCESSES", "MODS", "LAST SEEN"
    );
    println!("{}", "─".repeat(96));
    for entry in &report.files {
        println!(
            "  {:40} {:>8} {:>8} {:>12} {}",
            truncate(&entry.file_path, 40),
            entry.access_count,
            entry.modification_count,
            entry.last_seen_at.as_deref().unwrap_or("-"),
            entry.last_modified_at.as_deref().unwrap_or("-"),
        );
    }
}

pub(super) fn print_snapshot_comparison(id: &str, comparison: &SnapshotComparison) {
    println!(
        "Project '{}' — snapshot {} -> {}",
        id, comparison.base_run.run_id, comparison.head_run.run_id
    );
    println!(
        "  Base: files={} captured_at={}",
        comparison.base_run.file_count, comparison.base_run.captured_at
    );
    println!(
        "  Head: files={} captured_at={}",
        comparison.head_run.file_count, comparison.head_run.captured_at
    );
    println!(
        "  Summary: +{}  -{}  ~{}  ={}",
        comparison.summary.added_files,
        comparison.summary.removed_files,
        comparison.summary.modified_files,
        comparison.summary.unchanged_files,
    );
    println!();

    if comparison.changes.is_empty() {
        println!("  No changed files in the returned diff set.");
        return;
    }

    for change in &comparison.changes {
        println!("  {:8} {}", change.change_type, change.file_path);
    }
}

pub(super) fn print_usage_trends(id: &str, report: &UsageTrendReport) {
    println!(
        "Project '{}' — trend window={} bucket={} tracked_files={}",
        id, report.window, report.summary.bucket_size, report.summary.tracked_files
    );
    println!("  Range: {} .. {}", report.start_time, report.end_time);
    println!(
        "  Totals: access={} modifications={}",
        report.summary.total_access_count, report.summary.total_modification_count
    );
    println!();

    if report.files.is_empty() {
        println!("  No trend data recorded in this window.");
        return;
    }

    println!(
        "  {:40} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "PATH", "TOTAL", "MODS", "CUR", "PREV", "DELTA"
    );
    println!("{}", "─".repeat(90));
    for entry in &report.files {
        println!(
            "  {:40} {:>8} {:>8} {:>8} {:>8} {:>8}",
            truncate(&entry.file_path, 40),
            entry.total_access_count,
            entry.total_modification_count,
            entry.current_bucket_access_count,
            entry.previous_bucket_access_count,
            entry.delta_access_count,
        );
    }
}
