use serde_json::{json, Value};
use std::path::Path;

use crate::contracts::{versioned_project_payload, MCP_STATS_V1, MCP_UNUSED_FILES_V1};
use crate::core::file_classification::{
    classify_file_path, FilePathClassification, FilePathClassificationFilter,
};
use crate::core::stats::ProjectSummary;
use crate::storage::queries::{StatsEntry, VerificationRun};

use super::super::super::{stats_guidance, unused_guidance};

pub(crate) const DEFAULT_OBSERVATION_PAYLOAD_LIMIT: usize = 50;

#[cfg(test)]
pub(crate) fn stats_payload(
    id: &str,
    summary: &ProjectSummary,
    entries: &[StatsEntry],
    root_path: &Path,
    verification_runs: &[VerificationRun],
) -> Value {
    stats_payload_with_limit(
        id,
        summary,
        entries,
        root_path,
        verification_runs,
        DEFAULT_OBSERVATION_PAYLOAD_LIMIT,
        FilePathClassificationFilter::All,
    )
}

pub(crate) fn stats_payload_with_limit(
    id: &str,
    summary: &ProjectSummary,
    entries: &[StatsEntry],
    root_path: &Path,
    verification_runs: &[VerificationRun],
    limit: usize,
    path_filter: FilePathClassificationFilter,
) -> Value {
    let limit = normalized_observation_limit(limit);
    let filtered_entries: Vec<StatsEntry> = entries
        .iter()
        .filter(|entry| path_filter.matches(classify_file_path(&entry.file_path)))
        .cloned()
        .collect();
    let files: Vec<Value> = entries
        .iter()
        .filter(|entry| path_filter.matches(classify_file_path(&entry.file_path)))
        .take(limit)
        .map(|e| {
            json!({
                "path": e.file_path,
                "size": e.size,
                "file_type": e.file_type,
                "access_count": e.access_count,
                "estimated_duration_ms": e.estimated_duration_ms,
                "modification_count": e.modification_count,
                "last_access_time": e.last_access_time,
                "path_classification": classify_file_path(&e.file_path).as_str(),
            })
        })
        .collect();
    versioned_project_payload(
        MCP_STATS_V1,
        id,
        [
            (
                "summary",
                json!({
                    "total_files": summary.total_files,
                    "accessed": summary.accessed_files,
                    "unused": summary.unused_files,
                }),
            ),
            (
                "result_window",
                observation_result_window(filtered_entries.len(), files.len(), limit, path_filter),
            ),
            ("classification_summary", classification_summary(entries)),
            ("files", json!(files)),
            (
                "guidance",
                stats_guidance(
                    root_path,
                    summary,
                    &filtered_entries,
                    verification_runs,
                    path_filter,
                ),
            ),
        ],
    )
}

#[cfg(test)]
pub(crate) fn unused_files_payload(
    id: &str,
    unused: &[StatsEntry],
    root_path: &Path,
    verification_runs: &[VerificationRun],
) -> Value {
    unused_files_payload_with_limit(
        id,
        unused,
        root_path,
        verification_runs,
        DEFAULT_OBSERVATION_PAYLOAD_LIMIT,
        FilePathClassificationFilter::All,
    )
}

pub(crate) fn unused_files_payload_with_limit(
    id: &str,
    unused: &[StatsEntry],
    root_path: &Path,
    verification_runs: &[VerificationRun],
    limit: usize,
    path_filter: FilePathClassificationFilter,
) -> Value {
    let limit = normalized_observation_limit(limit);
    let filtered_entries: Vec<StatsEntry> = unused
        .iter()
        .filter(|entry| path_filter.matches(classify_file_path(&entry.file_path)))
        .cloned()
        .collect();
    let files: Vec<Value> = unused
        .iter()
        .filter(|entry| path_filter.matches(classify_file_path(&entry.file_path)))
        .take(limit)
        .map(|e| {
            json!({
                "path": e.file_path,
                "size": e.size,
                "file_type": e.file_type,
                "path_classification": classify_file_path(&e.file_path).as_str(),
            })
        })
        .collect();
    let mut fields: Vec<(&str, Value)> = vec![
        ("unused_count", json!(unused.len())),
        (
            "result_window",
            observation_result_window(filtered_entries.len(), files.len(), limit, path_filter),
        ),
        ("classification_summary", classification_summary(unused)),
        ("files", json!(files)),
        (
            "guidance",
            unused_guidance(root_path, &filtered_entries, verification_runs, path_filter),
        ),
    ];
    if path_filter != FilePathClassificationFilter::All {
        fields.push(("filtered_unused_count", json!(filtered_entries.len())));
    }
    versioned_project_payload(MCP_UNUSED_FILES_V1, id, fields)
}

pub(super) fn normalized_observation_limit(limit: usize) -> usize {
    if limit == 0 {
        DEFAULT_OBSERVATION_PAYLOAD_LIMIT
    } else {
        limit
    }
}

pub(super) fn observation_result_window(
    total_count: usize,
    returned_count: usize,
    limit: usize,
    path_filter: FilePathClassificationFilter,
) -> Value {
    json!({
        "total_count": total_count,
        "returned_count": returned_count,
        "limit": limit,
        "truncated": returned_count < total_count,
        "path_classification": path_filter.as_str(),
    })
}

pub(super) fn classification_summary(entries: &[StatsEntry]) -> Value {
    let mut source_files = 0;
    let mut infrastructure_files = 0;
    let mut backup_files = 0;
    let mut project_files = 0;

    for entry in entries {
        match classify_file_path(&entry.file_path) {
            FilePathClassification::Source => source_files += 1,
            FilePathClassification::Infrastructure => infrastructure_files += 1,
            FilePathClassification::Backup => backup_files += 1,
            FilePathClassification::Project => project_files += 1,
        }
    }

    json!({
        "source_files": source_files,
        "infrastructure_files": infrastructure_files,
        "backup_files": backup_files,
        "project_files": project_files,
    })
}
