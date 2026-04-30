use crate::error::{OpenDogError, Result};
use crate::storage::database::Database;
use crate::storage::queries::{self, HistoricalSnapshotEntry, SnapshotRunRecord};
use std::collections::BTreeMap;

use super::{
    SnapshotComparison, SnapshotComparisonSummary, SnapshotDiffEntry, SnapshotFileVersion,
    SnapshotRunInfo,
};

pub fn compare_latest_snapshots(db: &Database, limit: usize) -> Result<SnapshotComparison> {
    let runs = queries::list_snapshot_runs(db, 2)?;
    if runs.len() < 2 {
        return Err(OpenDogError::InvalidInput(
            "at least two snapshot runs are required for comparison".to_string(),
        ));
    }
    compare_snapshot_runs(db, runs[1].id, runs[0].id, limit)
}

pub fn compare_snapshot_runs(
    db: &Database,
    base_run_id: i64,
    head_run_id: i64,
    limit: usize,
) -> Result<SnapshotComparison> {
    if base_run_id == head_run_id {
        return Err(OpenDogError::InvalidInput(
            "base_run_id and head_run_id must differ".to_string(),
        ));
    }

    let base_run = queries::get_snapshot_run(db, base_run_id)?.ok_or_else(|| {
        OpenDogError::InvalidInput(format!("snapshot run {} not found", base_run_id))
    })?;
    let head_run = queries::get_snapshot_run(db, head_run_id)?.ok_or_else(|| {
        OpenDogError::InvalidInput(format!("snapshot run {} not found", head_run_id))
    })?;
    let base_entries = history_map(queries::get_snapshot_history_entries(db, base_run_id)?);
    let head_entries = history_map(queries::get_snapshot_history_entries(db, head_run_id)?);

    let mut all_paths = BTreeMap::new();
    for path in base_entries.keys() {
        all_paths.insert(path.clone(), ());
    }
    for path in head_entries.keys() {
        all_paths.insert(path.clone(), ());
    }

    let mut summary = SnapshotComparisonSummary {
        added_files: 0,
        removed_files: 0,
        modified_files: 0,
        unchanged_files: 0,
    };
    let mut changes = Vec::new();

    for path in all_paths.into_keys() {
        match (base_entries.get(&path), head_entries.get(&path)) {
            (None, Some(after)) => {
                summary.added_files += 1;
                changes.push(SnapshotDiffEntry {
                    file_path: path,
                    change_type: "added".to_string(),
                    before: None,
                    after: Some(after.clone()),
                });
            }
            (Some(before), None) => {
                summary.removed_files += 1;
                changes.push(SnapshotDiffEntry {
                    file_path: path,
                    change_type: "removed".to_string(),
                    before: Some(before.clone()),
                    after: None,
                });
            }
            (Some(before), Some(after)) if before != after => {
                summary.modified_files += 1;
                changes.push(SnapshotDiffEntry {
                    file_path: path,
                    change_type: "modified".to_string(),
                    before: Some(before.clone()),
                    after: Some(after.clone()),
                });
            }
            (Some(_), Some(_)) => {
                summary.unchanged_files += 1;
            }
            (None, None) => {}
        }
    }

    changes.truncate(limit.max(1));

    Ok(SnapshotComparison {
        base_run: snapshot_run_info(base_run),
        head_run: snapshot_run_info(head_run),
        summary,
        changes,
    })
}

fn history_map(entries: Vec<HistoricalSnapshotEntry>) -> BTreeMap<String, SnapshotFileVersion> {
    entries
        .into_iter()
        .map(|entry| {
            (
                entry.path,
                SnapshotFileVersion {
                    size: entry.size,
                    mtime: entry.mtime,
                    file_type: entry.file_type,
                },
            )
        })
        .collect()
}

fn snapshot_run_info(run: SnapshotRunRecord) -> SnapshotRunInfo {
    SnapshotRunInfo {
        run_id: run.id,
        captured_at: run.captured_at,
        file_count: run.file_count,
    }
}
