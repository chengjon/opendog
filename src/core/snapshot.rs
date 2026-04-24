use crate::config::ProjectConfig;
use crate::error::Result;
use crate::storage::database::Database;
use crate::storage::queries::{self, SnapshotEntry};
use rusqlite::params;
use std::path::Path;
use walkdir::WalkDir;

pub struct SnapshotResult {
    pub total_files: usize,
    pub new_files: usize,
    pub removed_files: usize,
}

pub fn take_snapshot(db: &Database, root: &Path, config: &ProjectConfig) -> Result<SnapshotResult> {
    let scan_timestamp = now_iso();

    let entries = scan_directory(root, config, &scan_timestamp)?;

    let previous_count = queries::count_snapshot(db)? as usize;

    // Remove files that no longer exist (incremental: delete stale entries)
    let removed = remove_missing_entries(db, root, &scan_timestamp)?;

    // Insert current snapshot
    queries::insert_snapshot_batch(db, &entries)?;

    let new_count = entries.len();
    Ok(SnapshotResult {
        total_files: new_count,
        new_files: new_count.saturating_sub(previous_count - removed),
        removed_files: removed,
    })
}

pub fn get_snapshot_count(db: &Database) -> Result<i64> {
    queries::count_snapshot(db)
}

pub fn get_snapshot_paths(db: &Database) -> Result<Vec<String>> {
    queries::get_snapshot_paths(db)
}

fn scan_directory(root: &Path, config: &ProjectConfig, scan_timestamp: &str) -> Result<Vec<SnapshotEntry>> {
    let mut entries = Vec::new();

    for entry in WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // SNAP-04: skip inaccessible files
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let rel_path = match path.strip_prefix(root) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let rel_str = rel_path.to_str().unwrap_or("");
        if should_ignore(rel_str, config) {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue, // SNAP-04: skip permission errors
        };

        let file_type = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        entries.push(SnapshotEntry {
            path: rel_str.to_string(),
            size: metadata.len() as i64,
            mtime: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            file_type,
            scan_timestamp: scan_timestamp.to_string(),
        });
    }

    Ok(entries)
}

fn should_ignore(rel_path: &str, config: &ProjectConfig) -> bool {
    let components: Vec<&str> = std::path::Path::new(rel_path)
        .iter()
        .filter_map(|c| c.to_str())
        .collect();

    for pattern in &config.ignore_patterns {
        // Check if pattern matches any path component (directory-level filter)
        for component in &components {
            if pattern.starts_with('*') {
                // Glob suffix pattern like "*.pyc"
                let suffix = &pattern[1..];
                if component.ends_with(suffix) {
                    return true;
                }
            } else if *component == *pattern {
                return true;
            }
        }
    }
    false
}

fn remove_missing_entries(db: &Database, root: &Path, _scan_timestamp: &str) -> Result<usize> {
    let existing_paths = queries::get_snapshot_paths(db)?;
    if existing_paths.is_empty() {
        return Ok(0);
    }

    let mut stale = Vec::new();
    for rel_path in &existing_paths {
        let full_path = root.join(rel_path);
        if !full_path.exists() {
            stale.push(rel_path.clone());
        }
    }

    if stale.is_empty() {
        return Ok(0);
    }

    // Delete stale entries individually
    let mut count = 0usize;
    for path in &stale {
        count += db.execute("DELETE FROM snapshot WHERE path = ?1", params![path])?;
    }
    Ok(count)
}

fn now_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}
