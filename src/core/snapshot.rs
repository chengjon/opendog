use crate::config::{should_ignore_path, ProjectConfig};
use crate::error::Result;
use crate::storage::database::Database;
use crate::storage::queries::{self, SnapshotEntry};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotResult {
    pub total_files: usize,
    pub new_files: usize,
    pub removed_files: usize,
}

pub fn take_snapshot(db: &Database, root: &Path, config: &ProjectConfig) -> Result<SnapshotResult> {
    let scan_timestamp = now_iso();

    let entries = scan_directory(root, config, &scan_timestamp)?;
    queries::insert_snapshot_history(db, &scan_timestamp, &entries)?;

    let previous_count = queries::count_snapshot(db)? as usize;

    // Remove files that no longer exist (incremental: delete stale entries)
    let removed = remove_missing_entries(db, root, &scan_timestamp)?;

    // Insert current snapshot
    queries::insert_snapshot_batch(db, &entries)?;

    let new_count = entries.len();
    Ok(SnapshotResult {
        total_files: new_count,
        new_files: new_count.saturating_sub(previous_count.saturating_sub(removed)),
        removed_files: removed,
    })
}

pub fn get_snapshot_count(db: &Database) -> Result<i64> {
    queries::count_snapshot(db)
}

pub fn get_snapshot_paths(db: &Database) -> Result<Vec<String>> {
    queries::get_snapshot_paths(db)
}

fn scan_directory(
    root: &Path,
    config: &ProjectConfig,
    scan_timestamp: &str,
) -> Result<Vec<SnapshotEntry>> {
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
    should_ignore_path(rel_path, config)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProjectConfig;

    #[test]
    fn now_iso_returns_unix_timestamp_string() {
        let result = now_iso();
        // Should be a non-empty string of digits (unix epoch seconds)
        assert!(!result.is_empty());
        assert!(result.chars().all(|c| c.is_ascii_digit()));
        let secs: u64 = result.parse().expect("now_iso should return a valid number");
        // Sanity: should be after year 2020 (~1577836800) and before year 2100 (~4102444800)
        assert!(secs > 1_577_836_800);
        assert!(secs < 4_102_444_800);
    }

    #[test]
    fn should_ignore_matches_configured_pattern() {
        let config = ProjectConfig {
            ignore_patterns: vec!["node_modules".to_string(), "*.pyc".to_string()],
            process_whitelist: vec![],
        };
        assert!(should_ignore("src/node_modules/pkg/index.js", &config));
        assert!(should_ignore("build/output.pyc", &config));
    }

    #[test]
    fn should_ignore_does_not_match_unconfigured_path() {
        let config = ProjectConfig {
            ignore_patterns: vec!["node_modules".to_string()],
            process_whitelist: vec![],
        };
        assert!(!should_ignore("src/main.rs", &config));
        assert!(!should_ignore("lib/core/mod.rs", &config));
    }

    #[test]
    fn should_ignore_with_empty_patterns_matches_nothing() {
        let config = ProjectConfig {
            ignore_patterns: vec![],
            process_whitelist: vec![],
        };
        assert!(!should_ignore("any/path.rs", &config));
    }

    #[test]
    fn should_ignore_with_default_config() {
        let config = ProjectConfig::default();
        // Default config includes "node_modules", ".git", "target", etc.
        assert!(should_ignore("node_modules/pkg/index.js", &config));
        assert!(should_ignore("target/debug/app", &config));
        assert!(!should_ignore("src/main.rs", &config));
    }

    #[test]
    fn should_ignore_normalizes_backslashes() {
        let config = ProjectConfig {
            ignore_patterns: vec!["node_modules".to_string()],
            process_whitelist: vec![],
        };
        // Backslash normalization is handled by should_ignore_path
        assert!(should_ignore("src\\node_modules\\pkg", &config));
    }
}
