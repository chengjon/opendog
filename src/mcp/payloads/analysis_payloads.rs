use serde_json::{json, Value};

use crate::contracts::versioned_project_payload;
#[cfg(test)]
use crate::core::file_classification::FilePathClassificationFilter;
use crate::core::report::{
    ActivityRollupReport, SnapshotComparison, TimeWindowReport, UsageTrendReport,
};
use crate::core::retention::ProjectDataCleanupResult;

use super::super::tool_guidance;

mod observation;

#[cfg(test)]
use observation::{
    classification_summary, normalized_observation_limit, observation_result_window,
};
#[cfg(test)]
pub(crate) use observation::{stats_payload, unused_files_payload};
pub(crate) use observation::{
    stats_payload_with_limit, unused_files_payload_with_limit, DEFAULT_OBSERVATION_PAYLOAD_LIMIT,
};

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

pub(crate) fn activity_rollups_payload(
    schema_version: &str,
    id: &str,
    report: &ActivityRollupReport,
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
            ("days", json!(report.days)),
            (
                "guidance",
                tool_guidance(
                    "Use activity rollups to inspect long-lived usage volume after raw activity rows have been compacted.",
                    &[
                        "Rollups preserve daily counts, not per-file or per-process raw detail",
                        "Use usage trends for recent file-level detail before retention cleanup removes raw rows",
                    ],
                    &["get_usage_trends", "get_time_window_report", "cleanup-data"],
                    Some(
                        "Activity rollups are safe for historical volume checks but cannot reconstruct deleted raw activity rows.",
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
            ("rolled_up", json!(result.rolled_up)),
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
                        "Activity cleanup preserves daily rollup counts before deleting raw activity rows",
                        "Keep at least 2 snapshot runs when snapshot comparison should remain immediately available",
                        "Use vacuum only after large cleanup batches because it rewrites the SQLite file for that project",
                    ],
                    &["get_time_window_report", "compare_snapshots", "get_verification_status"],
                    Some(
                        "Cleanup removes OPENDOG-retained evidence only; activity cleanup preserves daily counts but removes raw per-row detail, and VACUUM should be reserved for explicit space-reclaim passes.",
                    ),
                ),
            ),
        ],
    )
}

#[cfg(test)]
mod tests;
