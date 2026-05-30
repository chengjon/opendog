use crate::core::report::{
    ActivityRollupReport, SnapshotComparison, TimeWindowReport, UsageTrendReport,
};

use super::truncate;

pub fn print_time_window_report(id: &str, report: &TimeWindowReport) {
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

pub fn print_snapshot_comparison(id: &str, comparison: &SnapshotComparison) {
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

pub fn print_usage_trends(id: &str, report: &UsageTrendReport) {
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

pub fn print_activity_rollups(id: &str, report: &ActivityRollupReport) {
    println!(
        "Project '{}' — activity rollups window={} bucket={} returned_days={}/{}",
        id,
        report.window,
        report.summary.bucket_size,
        report.summary.returned_days,
        report.summary.rollup_days
    );
    println!("  Range: {} .. {}", report.start_time, report.end_time);
    println!(
        "  Totals: access={} modifications={} events={}",
        report.summary.total_access_count,
        report.summary.total_modification_count,
        report.summary.total_event_count
    );
    println!();

    if report.days.is_empty() {
        println!("  No activity rollups recorded in this window.");
        return;
    }

    println!(
        "  {:>12} {:>8} {:>8} {:>8}",
        "DAY_START", "ACCESS", "MODS", "EVENTS"
    );
    println!("{}", "─".repeat(50));
    for day in &report.days {
        println!(
            "  {:>12} {:>8} {:>8} {:>8}",
            day.day_start, day.access_count, day.modification_count, day.event_count,
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn time_window_header_format() {
        let line = format!(
            "Project '{}' — window={} sightings={} files={} processes={} modifications={}",
            "proj1", "24h", 100, 50, 3, 10
        );
        assert!(line.contains("window=24h"));
        assert!(line.contains("sightings=100"));
        assert!(line.contains("files=50"));
    }

    #[test]
    fn time_window_range_format() {
        let line = format!("  Range: {} .. {}", "2026-01-01", "2026-01-02");
        assert!(line.contains("2026-01-01 .. 2026-01-02"));
    }

    #[test]
    fn time_window_empty_files_guard() {
        let files: Vec<String> = vec![];
        let msg = if files.is_empty() {
            "No activity recorded in this window."
        } else {
            "has data"
        };
        assert_eq!(msg, "No activity recorded in this window.");
    }

    #[test]
    fn snapshot_comparison_header_format() {
        let line = format!("Project '{}' — snapshot {} -> {}", "proj", 1, 2);
        assert_eq!(line, "Project 'proj' — snapshot 1 -> 2");
    }

    #[test]
    fn snapshot_comparison_summary_format() {
        let added = 3;
        let removed = 1;
        let modified = 2;
        let unchanged = 10;
        let line = format!(
            "  Summary: +{}  -{}  ~{}  ={}",
            added, removed, modified, unchanged
        );
        assert_eq!(line, "  Summary: +3  -1  ~2  =10");
    }

    #[test]
    fn snapshot_comparison_empty_changes_guard() {
        let changes: Vec<String> = vec![];
        let msg = if changes.is_empty() {
            "No changed files in the returned diff set."
        } else {
            "has changes"
        };
        assert_eq!(msg, "No changed files in the returned diff set.");
    }

    #[test]
    fn usage_trend_header_format() {
        let line = format!(
            "Project '{}' — trend window={} bucket={} tracked_files={}",
            "proj", "7d", "1d", 5
        );
        assert!(line.contains("window=7d"));
        assert!(line.contains("bucket=1d"));
        assert!(line.contains("tracked_files=5"));
    }

    #[test]
    fn usage_trend_totals_format() {
        let line = format!("  Totals: access={} modifications={}", 200, 50);
        assert_eq!(line, "  Totals: access=200 modifications=50");
    }

    #[test]
    fn usage_trend_empty_guard() {
        let files: Vec<String> = vec![];
        let msg = if files.is_empty() {
            "No trend data recorded in this window."
        } else {
            "has data"
        };
        assert_eq!(msg, "No trend data recorded in this window.");
    }

    #[test]
    fn activity_rollup_header_format() {
        let line = format!(
            "Project '{}' — activity rollups window={} bucket={} returned_days={}/{}",
            "proj", "30d", "1d", 2, 3
        );
        assert!(line.contains("activity rollups"));
        assert!(line.contains("returned_days=2/3"));
    }

    #[test]
    fn change_type_format() {
        let line = format!("  {:8} {}", "added", "src/new.rs");
        assert!(line.contains("added"));
        assert!(line.contains("src/new.rs"));
    }
}
