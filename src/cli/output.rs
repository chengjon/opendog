use crate::config::ProjectInfo;
use crate::core::snapshot::SnapshotResult;
use crate::core::stats::ProjectSummary;
use crate::storage::queries::StatsEntry;

pub fn print_created(info: &ProjectInfo) {
    println!("Project '{}' created.", info.id);
    println!("  Root: {}", info.root_path.display());
    println!("  DB:   {}", info.db_path.display());
}

pub fn print_snapshot_result(id: &str, result: &SnapshotResult) {
    println!("Snapshot for project '{}':", id);
    println!("  Total files:  {}", result.total_files);
    println!("  New files:    {}", result.new_files);
    println!("  Removed:      {}", result.removed_files);
}

pub fn print_stats(id: &str, summary: &ProjectSummary, entries: &[StatsEntry]) {
    println!(
        "Project '{}' — {} files | {} accessed | {} unused",
        id, summary.total_files, summary.accessed_files, summary.unused_files
    );
    println!();
    if entries.is_empty() {
        println!("  No files in snapshot. Run 'opendog snapshot --id {}' first.", id);
        return;
    }

    println!(
        "  {:40} {:>8} {:>12} {:>8} {}",
        "PATH", "ACCESSES", "DURATION(ms)", "MODS", "LAST ACCESS"
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

pub fn print_unused(id: &str, unused: &[StatsEntry]) {
    println!("Unused files for project '{}' ({} files):", id, unused.len());
    println!();
    for e in unused.iter().take(100) {
        println!("  {} ({}, {} bytes)", e.file_path, e.file_type, e.size);
    }
    if unused.len() > 100 {
        println!("  ... and {} more files", unused.len() - 100);
    }
}

pub fn print_project_list(projects: &[ProjectInfo]) {
    if projects.is_empty() {
        println!("No projects registered.");
        return;
    }

    println!(
        "  {:20} {:40} {:10} {}",
        "ID", "ROOT PATH", "STATUS", "CREATED"
    );
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("...{}", &s[s.len() - max + 3..])
    }
}
