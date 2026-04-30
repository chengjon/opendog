use serde_json::{json, Value};
use std::path::Path;

use crate::contracts::{versioned_project_payload, MCP_STATS_V1, MCP_UNUSED_FILES_V1};
use crate::core::report::{SnapshotComparison, TimeWindowReport, UsageTrendReport};
use crate::core::retention::ProjectDataCleanupResult;
use crate::core::stats::ProjectSummary;
use crate::storage::queries::{StatsEntry, VerificationRun};

use super::super::{stats_guidance, tool_guidance, unused_guidance};

pub(crate) fn stats_payload(
    id: &str,
    summary: &ProjectSummary,
    entries: &[StatsEntry],
    root_path: &Path,
    verification_runs: &[VerificationRun],
) -> Value {
    let files: Vec<Value> = entries
        .iter()
        .map(|e| {
            json!({
                "path": e.file_path,
                "size": e.size,
                "file_type": e.file_type,
                "access_count": e.access_count,
                "estimated_duration_ms": e.estimated_duration_ms,
                "modification_count": e.modification_count,
                "last_access_time": e.last_access_time,
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
            ("files", json!(files)),
            (
                "guidance",
                stats_guidance(root_path, summary, entries, verification_runs),
            ),
        ],
    )
}

pub(crate) fn unused_files_payload(
    id: &str,
    unused: &[StatsEntry],
    root_path: &Path,
    verification_runs: &[VerificationRun],
) -> Value {
    let files: Vec<Value> = unused
        .iter()
        .map(|e| {
            json!({
                "path": e.file_path,
                "size": e.size,
                "file_type": e.file_type,
            })
        })
        .collect();
    versioned_project_payload(
        MCP_UNUSED_FILES_V1,
        id,
        [
            ("unused_count", json!(files.len())),
            ("files", json!(files)),
            (
                "guidance",
                unused_guidance(root_path, unused, verification_runs),
            ),
        ],
    )
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
