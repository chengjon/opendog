use serde_json::{json, Value};
use std::path::Path;

use crate::contracts::{versioned_project_payload, MCP_STATS_V1, MCP_UNUSED_FILES_V1};
use crate::core::file_classification::{
    classify_file_path, FilePathClassification, FilePathClassificationFilter,
};
use crate::core::report::{SnapshotComparison, TimeWindowReport, UsageTrendReport};
use crate::core::retention::ProjectDataCleanupResult;
use crate::core::stats::ProjectSummary;
use crate::storage::queries::{StatsEntry, VerificationRun};

use super::super::{stats_guidance, tool_guidance, unused_guidance};

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

fn normalized_observation_limit(limit: usize) -> usize {
    if limit == 0 {
        DEFAULT_OBSERVATION_PAYLOAD_LIMIT
    } else {
        limit
    }
}

fn observation_result_window(
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

fn classification_summary(entries: &[StatsEntry]) -> Value {
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

pub(crate) fn time_window_report_payload(
    schema_version: &str,
    id: &str,
    report: &TimeWindowReport,
) -> Value {
    versioned_project_payload(
        schema_version,
        id,
        [
            ("window", json!(report.window)),
            (
                "range",
                json!({
                    "start_time": report.start_time,
                    "end_time": report.end_time,
                }),
            ),
            ("summary", json!(report.summary)),
            ("files", json!(report.files)),
            (
                "guidance",
                tool_guidance(
                    "Use time-window reports to understand recent activity concentration before choosing hotspot review or cleanup work.",
                    &[
                        "Compare 24h versus 7d output to distinguish fresh spikes from sustained activity",
                        "Use snapshot comparison before deletion or broad refactor decisions",
                    ],
                    &["compare_snapshots", "get_usage_trends", "get_stats"],
                    Some(
                        "Time-window analytics are evidence summaries, not proof that untouched files are safe to remove.",
                    ),
                ),
            ),
        ],
    )
}

pub(crate) fn snapshot_comparison_payload(
    schema_version: &str,
    id: &str,
    comparison: &SnapshotComparison,
) -> Value {
    versioned_project_payload(
        schema_version,
        id,
        [
            ("base_run", json!(comparison.base_run)),
            ("head_run", json!(comparison.head_run)),
            ("summary", json!(comparison.summary)),
            ("changes", json!(comparison.changes)),
            (
                "guidance",
                tool_guidance(
                    "Use snapshot comparison to verify what changed between scan baselines before cleanup or regression investigation.",
                    &[
                        "Review added, removed, and modified files before drawing conclusions from a single current snapshot",
                        "Pair snapshot comparison with git diff and verification when changes may be risky",
                    ],
                    &["take_snapshot", "get_time_window_report", "get_usage_trends"],
                    Some(
                        "Snapshot comparison reflects filesystem state between scans; confirm code intent with shell diff and tests before acting.",
                    ),
                ),
            ),
        ],
    )
}

pub(crate) fn usage_trends_payload(
    schema_version: &str,
    id: &str,
    report: &UsageTrendReport,
) -> Value {
    versioned_project_payload(
        schema_version,
        id,
        [
            ("window", json!(report.window)),
            (
                "range",
                json!({
                    "start_time": report.start_time,
                    "end_time": report.end_time,
                }),
            ),
            ("summary", json!(report.summary)),
            ("files", json!(report.files)),
            (
                "guidance",
                tool_guidance(
                    "Use usage trends to separate stable core files from short-lived spikes and cooling modules.",
                    &[
                        "Look at delta_access_count to spot rising or cooling files",
                        "Use time-window stats for the latest summary and snapshot comparison for structural file-set changes",
                    ],
                    &["get_time_window_report", "compare_snapshots", "get_unused_files"],
                    Some(
                        "Trend buckets are derived from recorded sightings and modify events only; they do not infer semantic importance on their own.",
                    ),
                ),
            ),
        ],
    )
}

pub(crate) fn cleanup_project_data_payload(
    schema_version: &str,
    id: &str,
    result: &ProjectDataCleanupResult,
) -> Value {
    versioned_project_payload(
        schema_version,
        id,
        [
            ("scope", json!(result.scope)),
            ("dry_run", json!(result.dry_run)),
            ("older_than_days", json!(result.older_than_days)),
            ("keep_snapshot_runs", json!(result.keep_snapshot_runs)),
            ("vacuum", json!(result.vacuum)),
            ("deleted", json!(result.deleted)),
            ("storage_before", json!(result.storage_before)),
            ("storage_after", json!(result.storage_after)),
            ("maintenance", json!(result.maintenance)),
            ("notes", json!(result.notes)),
            (
                "guidance",
                tool_guidance(
                    "Selective cleanup only removes retained OPENDOG evidence; it does not delete source files.",
                    &[
                        "Use dry_run first when pruning shared or long-lived project evidence",
                        "Keep at least 2 snapshot runs when snapshot comparison should remain immediately available",
                        "Use vacuum only after large cleanup batches because it rewrites the SQLite file for that project",
                    ],
                    &["get_time_window_report", "compare_snapshots", "get_verification_status"],
                    Some(
                        "Cleanup removes OPENDOG-retained evidence only; validate whether you still need historical trends before deleting raw activity or verification rows, and reserve VACUUM for explicit space-reclaim passes.",
                    ),
                ),
            ),
        ],
    )
}
