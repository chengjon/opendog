use crate::config::ProjectInfo;
use crate::core::retention::ProjectDataCleanupResult;
use crate::core::snapshot::SnapshotResult;
use crate::core::stats::ProjectSummary;
use crate::storage::queries::StatsEntry;

use super::truncate;

pub(super) fn print_created(info: &ProjectInfo) {
    println!("Project '{}' created.", info.id);
    println!("  Root: {}", info.root_path.display());
    println!("  DB:   {}", info.db_path.display());
}

pub(super) fn print_snapshot_result(id: &str, result: &SnapshotResult) {
    println!("Snapshot for project '{}':", id);
    println!("  Total files:  {}", result.total_files);
    println!("  New files:    {}", result.new_files);
    println!("  Removed:      {}", result.removed_files);
}

pub(super) fn print_stats(id: &str, summary: &ProjectSummary, entries: &[StatsEntry]) {
    println!(
        "Project '{}' — {} files | {} accessed | {} unused",
        id, summary.total_files, summary.accessed_files, summary.unused_files
    );
    println!();
    if entries.is_empty() {
        println!(
            "  No files in snapshot. Run 'opendog snapshot --id {}' first.",
            id
        );
        return;
    }

    println!(
        "  {:40} {:>8} {:>12} {:>8} LAST ACCESS",
        "PATH", "ACCESSES", "DURATION(ms)", "MODS"
    );
    println!("{}", "─".repeat(90));
    for e in entries.iter().take(50) {
        println!(
            "  {:40} {:>8} {:>12} {:>8} {}",
            truncate(&e.file_path, 40),
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

pub(super) fn print_unused(id: &str, unused: &[StatsEntry]) {
    println!(
        "Unused files for project '{}' ({} files):",
        id,
        unused.len()
    );
    println!();
    for e in unused.iter().take(100) {
        println!("  {} ({}, {} bytes)", e.file_path, e.file_type, e.size);
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
