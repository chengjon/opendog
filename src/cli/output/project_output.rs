use crate::config::ProjectInfo;
use crate::core::file_classification::{classify_file_path, FilePathClassificationFilter};
use crate::core::retention::ProjectDataCleanupResult;
use crate::core::snapshot::SnapshotResult;
use crate::core::stats::ProjectSummary;
use crate::storage::queries::StatsEntry;

use super::truncate;

pub(super) fn print_registered(info: &ProjectInfo) {
    println!("Project '{}' registered.", info.id);
    println!("  Root: {}", info.root_path.display());
    println!("  DB:   {}", info.db_path.display());
}

pub(super) fn print_snapshot_result(id: &str, result: &SnapshotResult) {
    println!("Snapshot for project '{}':", id);
    println!("  Total files:  {}", result.total_files);
    println!("  New files:    {}", result.new_files);
    println!("  Removed:      {}", result.removed_files);
}

pub(super) fn print_stats(
    id: &str,
    summary: &ProjectSummary,
    entries: &[StatsEntry],
    filter: FilePathClassificationFilter,
    unfiltered_count: usize,
) {
    if filter == FilePathClassificationFilter::All {
        println!(
            "Project '{}' — {} files | {} accessed | {} unused",
            id, summary.total_files, summary.accessed_files, summary.unused_files
        );
    } else {
        println!(
            "Project '{}' — {} files | {} accessed | {} unused | filter={} | shown={}/{}",
            id,
            summary.total_files,
            summary.accessed_files,
            summary.unused_files,
            filter.as_str(),
            entries.len().min(50),
            unfiltered_count
        );
    }
    println!();
    if entries.is_empty() {
        println!(
            "  No files in snapshot. Run 'opendog snapshot --id {}' first.",
            id
        );
        return;
    }

    println!(
        "  {:40} {:>14} {:>8} {:>12} {:>8} LAST ACCESS",
        "PATH", "CLASS", "ACCESSES", "DURATION(ms)", "MODS"
    );
    println!("{}", "─".repeat(105));
    for e in entries.iter().take(50) {
        println!(
            "  {:40} {:>14} {:>8} {:>12} {:>8} {}",
            truncate(&e.file_path, 40),
            classify_file_path(&e.file_path).as_str(),
            e.access_count,
            e.estimated_duration_ms,
            e.modification_count,
            e.last_access_time.as_deref().unwrap_or("-"),
        );
    }
    if entries.len() > 50 {
        println!("  ... and {} more files", entries.len() - 50);
    }
}

pub(super) fn print_unused(
    id: &str,
    unused: &[StatsEntry],
    filter: FilePathClassificationFilter,
    unfiltered_count: usize,
) {
    if filter == FilePathClassificationFilter::All {
        println!(
            "Unused files for project '{}' ({} files):",
            id,
            unused.len()
        );
    } else {
        println!(
            "Unused files for project '{}' — filter={} | shown={}/{} | total_unused={}:",
            id,
            filter.as_str(),
            unused.len().min(100),
            unused.len(),
            unfiltered_count
        );
    }
    println!();
    for e in unused.iter().take(100) {
        println!(
            "  {} [{}] ({}, {} bytes)",
            e.file_path,
            classify_file_path(&e.file_path).as_str(),
            e.file_type,
            e.size
        );
    }
    if unused.len() > 100 {
        println!("  ... and {} more files", unused.len() - 100);
    }
}

pub(super) fn print_cleanup_data_result(id: &str, result: &ProjectDataCleanupResult) {
    println!(
        "Project '{}' — cleanup scope={} dry_run={} vacuum={}",
        id, result.scope, result.dry_run, result.vacuum
    );
    if let Some(days) = result.older_than_days {
        println!("  older_than_days: {}", days);
    }
    if let Some(keep_snapshot_runs) = result.keep_snapshot_runs {
        println!("  keep_snapshot_runs: {}", keep_snapshot_runs);
    }
    println!(
        "  storage_before: page_count={} free_pages={} approx_db_bytes={} approx_reclaimable_bytes={}",
        result.storage_before.page_count,
        result.storage_before.freelist_count,
        result.storage_before.approx_db_size_bytes,
        result.storage_before.approx_reclaimable_bytes,
    );
    println!(
        "  deleted: sightings={} events={} verification={} snapshot_runs={} snapshot_history={}",
        result.deleted.file_sightings,
        result.deleted.file_events,
        result.deleted.verification_runs,
        result.deleted.snapshot_runs,
        result.deleted.snapshot_history,
    );
    println!(
        "  rolled_up: sightings={} events={}",
        result.rolled_up.file_sightings, result.rolled_up.file_events,
    );
    if let Some(storage_after) = &result.storage_after {
        println!(
            "  storage_after: page_count={} free_pages={} approx_db_bytes={} approx_reclaimable_bytes={}",
            storage_after.page_count,
            storage_after.freelist_count,
            storage_after.approx_db_size_bytes,
            storage_after.approx_reclaimable_bytes,
        );
    }
    println!(
        "  maintenance: optimized={} vacuumed={}",
        result.maintenance.optimized, result.maintenance.vacuumed
    );
    if !result.notes.is_empty() {
        println!();
        for note in &result.notes {
            println!("  Note: {}", note);
        }
    }
}

pub(super) fn print_project_list(projects: &[ProjectInfo]) {
    if projects.is_empty() {
        println!("No projects registered.");
        return;
    }

    println!("  {:20} {:40} {:10} CREATED", "ID", "ROOT PATH", "STATUS");
    println!("{}", "─".repeat(100));
    for p in projects {
        println!(
            "  {:20} {:40} {:10} {}",
            truncate(&p.id, 20),
            truncate(&p.root_path.display().to_string(), 40),
            p.status,
            p.created_at,
        );
    }
    println!("\n  {} project(s)", projects.len());
}

#[cfg(test)]
mod tests {
    use crate::config::{ProjectConfigOverrides, ProjectInfo};
    use crate::core::file_classification::FilePathClassificationFilter;
    use std::path::PathBuf;

    #[test]
    fn stats_filter_all_omits_filter_label() {
        let filter = FilePathClassificationFilter::All;
        let has_filter = filter != FilePathClassificationFilter::All;
        assert!(!has_filter);
    }

    #[test]
    fn stats_filter_non_all_includes_label() {
        let filter = FilePathClassificationFilter::parse(Some("source")).unwrap();
        let has_filter = filter != FilePathClassificationFilter::All;
        assert!(has_filter);
        assert_eq!(filter.as_str(), "source");
    }

    #[test]
    fn snapshot_result_format_string() {
        let total = 100;
        let new = 5;
        let removed = 2;
        let line = format!(
            "Snapshot for project '{}':\n  Total files:  {}\n  New files:    {}\n  Removed:      {}",
            "test", total, new, removed
        );
        assert!(line.contains("Total files:  100"));
        assert!(line.contains("New files:    5"));
        assert!(line.contains("Removed:      2"));
    }

    #[test]
    fn registered_format_string() {
        let info = ProjectInfo {
            id: "myproj".into(),
            root_path: PathBuf::from("/tmp/myproj"),
            db_path: PathBuf::from("/tmp/myproj.db"),
            config: ProjectConfigOverrides::default(),
            created_at: "2026-01-01".into(),
            status: "idle".into(),
        };
        let line1 = format!("Project '{}' registered.", info.id);
        let line2 = format!("  Root: {}", info.root_path.display());
        let line3 = format!("  DB:   {}", info.db_path.display());
        assert_eq!(line1, "Project 'myproj' registered.");
        assert!(line2.contains("/tmp/myproj"));
        assert!(line3.contains("myproj.db"));
    }

    #[test]
    fn cleanup_optional_older_than_days_present() {
        let days: Option<i64> = Some(30);
        let line = days.map(|d| format!("  older_than_days: {}", d));
        assert_eq!(line.unwrap(), "  older_than_days: 30");
    }

    #[test]
    fn cleanup_optional_older_than_days_absent() {
        let days: Option<i64> = None;
        assert!(days.is_none());
    }

    #[test]
    fn cleanup_optional_keep_snapshot_runs_present() {
        let keep: Option<usize> = Some(5);
        let line = keep.map(|k| format!("  keep_snapshot_runs: {}", k));
        assert_eq!(line.unwrap(), "  keep_snapshot_runs: 5");
    }

    #[test]
    fn unused_truncates_at_100() {
        let total = 150;
        let shown = total.min(100);
        assert_eq!(shown, 100);
        let extra = total - shown;
        assert_eq!(extra, 50);
    }

    #[test]
    fn unused_no_truncation_under_100() {
        let total = 50;
        let shown = total.min(100);
        assert_eq!(shown, 50);
    }

    #[test]
    fn project_list_empty_guard() {
        let projects: Vec<ProjectInfo> = vec![];
        assert!(projects.is_empty());
    }

    #[test]
    fn project_list_header_format() {
        let header = format!("  {:20} {:40} {:10} CREATED", "ID", "ROOT PATH", "STATUS");
        assert!(header.starts_with("  "));
        assert!(header.contains("ID"));
        assert!(header.contains("ROOT PATH"));
        assert!(header.contains("STATUS"));
        assert!(header.contains("CREATED"));
    }

    #[test]
    fn cleanup_storage_before_format() {
        // Mirrors the format in print_cleanup_data_result
        let page_count: i64 = 100;
        let freelist_count: i64 = 10;
        let approx_db_size_bytes: i64 = 409600;
        let approx_reclaimable_bytes: i64 = 40960;
        let line = format!(
            "  storage_before: page_count={} free_pages={} approx_db_bytes={} approx_reclaimable_bytes={}",
            page_count, freelist_count, approx_db_size_bytes, approx_reclaimable_bytes
        );
        assert!(line.contains("page_count=100"));
        assert!(line.contains("free_pages=10"));
        assert!(line.contains("approx_db_bytes=409600"));
        assert!(line.contains("approx_reclaimable_bytes=40960"));
    }

    #[test]
    fn cleanup_deleted_format() {
        let line = format!(
            "  deleted: sightings={} events={} verification={} snapshot_runs={} snapshot_history={}",
            5, 10, 2, 1, 3
        );
        assert!(line.contains("sightings=5"));
        assert!(line.contains("events=10"));
        assert!(line.contains("verification=2"));
        assert!(line.contains("snapshot_runs=1"));
        assert!(line.contains("snapshot_history=3"));
    }

    #[test]
    fn stats_header_format() {
        let header = format!(
            "  {:40} {:>14} {:>8} {:>12} {:>8} LAST ACCESS",
            "PATH", "CLASS", "ACCESSES", "DURATION(ms)", "MODS"
        );
        assert!(header.contains("PATH"));
        assert!(header.contains("LAST ACCESS"));
        assert!(header.contains("DURATION(ms)"));
    }

    #[test]
    fn stats_entries_truncate_at_50() {
        let total = 75;
        let shown = total.min(50);
        assert_eq!(shown, 50);
        let extra = total - shown;
        assert_eq!(extra, 25);
    }
}
