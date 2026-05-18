use crate::storage::queries::{StatsEntry, VerificationRun};
use std::path::PathBuf;

use super::ProjectGuidanceState;

pub(super) fn stats_entry(path: &str, access_count: i64, modification_count: i64) -> StatsEntry {
    StatsEntry {
        file_path: path.to_string(),
        size: 42,
        file_type: path.rsplit('.').next().unwrap_or("").to_string(),
        access_count,
        estimated_duration_ms: 0,
        modification_count,
        last_access_time: Some("1".to_string()),
        first_seen_time: None,
    }
}

pub(super) fn unused_stats_entry(path: &str) -> StatsEntry {
    StatsEntry {
        file_path: path.to_string(),
        size: 11,
        file_type: path.rsplit('.').next().unwrap_or("").to_string(),
        access_count: 0,
        estimated_duration_ms: 0,
        modification_count: 0,
        last_access_time: None,
        first_seen_time: None,
    }
}

pub(super) fn verification_run(
    id: i64,
    kind: &str,
    status: &str,
    command: &str,
    exit_code: Option<i64>,
    finished_at: String,
) -> VerificationRun {
    VerificationRun {
        id,
        kind: kind.to_string(),
        status: status.to_string(),
        command: command.to_string(),
        exit_code,
        summary: None,
        source: "cli".to_string(),
        started_at: None,
        finished_at,
    }
}

pub(super) fn project_state(
    id: &str,
    total_files: i64,
    accessed_files: i64,
    unused_files: i64,
) -> ProjectGuidanceState {
    ProjectGuidanceState {
        id: id.to_string(),
        status: "monitoring".to_string(),
        root_path: PathBuf::from(format!("/tmp/{id}")),
        total_files,
        accessed_files,
        unused_files,
        latest_snapshot_captured_at: Some(super::fresh_ts()),
        latest_activity_at: Some(super::fresh_ts()),
        latest_verification_at: Some(super::fresh_ts()),
    }
}
