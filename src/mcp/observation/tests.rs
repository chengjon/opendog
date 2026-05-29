use super::*;
use crate::mcp::guidance_payload::ProjectGuidanceState;
use crate::storage::queries::{StatsEntry, VerificationRun};
use std::path::PathBuf;

const NOW: i64 = 1_700_000_000;

fn make_stats_entry(last_access: Option<&str>) -> StatsEntry {
    StatsEntry {
        file_path: "src/main.rs".to_string(),
        size: 1024,
        file_type: "rs".to_string(),
        access_count: 5,
        estimated_duration_ms: 500,
        modification_count: 2,
        last_access_time: last_access.map(|s| s.to_string()),
        first_seen_time: Some("1600000000".to_string()),
    }
}

fn make_verification_run(finished_at: &str) -> VerificationRun {
    VerificationRun {
        id: 1,
        kind: "test".to_string(),
        status: "passed".to_string(),
        command: "cargo test".to_string(),
        exit_code: Some(0),
        summary: Some("all passed".to_string()),
        source: "cli".to_string(),
        started_at: Some("1600000000".to_string()),
        finished_at: finished_at.to_string(),
    }
}

fn make_project(
    total_files: i64,
    accessed_files: i64,
    latest_snapshot_at: Option<&str>,
    latest_activity_at: Option<&str>,
    latest_verification_at: Option<&str>,
) -> ProjectGuidanceState {
    ProjectGuidanceState {
        id: "test-project".to_string(),
        status: "monitoring".to_string(),
        root_path: PathBuf::from("/tmp/test-project"),
        total_files,
        accessed_files,
        unused_files: 0,
        latest_snapshot_captured_at: latest_snapshot_at.map(|s| s.to_string()),
        latest_activity_at: latest_activity_at.map(|s| s.to_string()),
        latest_verification_at: latest_verification_at.map(|s| s.to_string()),
    }
}

// ---- parse_unix_timestamp ----

#[test]
fn parse_unix_timestamp_valid() {
    assert_eq!(parse_unix_timestamp("1700000000"), Some(1_700_000_000));
    assert_eq!(parse_unix_timestamp("0"), Some(0));
}

#[test]
fn parse_unix_timestamp_invalid_string() {
    assert_eq!(parse_unix_timestamp("not-a-number"), None);
    assert_eq!(parse_unix_timestamp("abc123"), None);
}

#[test]
fn parse_unix_timestamp_empty() {
    assert_eq!(parse_unix_timestamp(""), None);
}

#[test]
fn parse_unix_timestamp_negative() {
    assert_eq!(parse_unix_timestamp("-1"), None);
}

// ---- max_timestamp_string ----

#[test]
fn max_timestamp_multiple_values() {
    let values = vec!["1699900000", "1700000000", "1699950000"];
    let result = max_timestamp_string(values.into_iter());
    assert_eq!(result.as_deref(), Some("1700000000"));
}

#[test]
fn max_timestamp_single_value() {
    let values = vec!["1700000000"];
    let result = max_timestamp_string(values.into_iter());
    assert_eq!(result.as_deref(), Some("1700000000"));
}

#[test]
fn max_timestamp_empty_iterator() {
    let values: Vec<&str> = vec![];
    let result = max_timestamp_string(values.into_iter());
    assert!(result.is_none());
}

#[test]
fn max_timestamp_skips_invalid() {
    let values = vec!["invalid", "1700000000", "bad"];
    let result = max_timestamp_string(values.into_iter());
    assert_eq!(result.as_deref(), Some("1700000000"));
}

// ---- latest_activity_timestamp ----

#[test]
fn latest_activity_with_entries() {
    let entries = vec![
        make_stats_entry(Some("1699900000")),
        make_stats_entry(Some("1700000000")),
        make_stats_entry(Some("1699950000")),
    ];
    let result = latest_activity_timestamp(&entries);
    assert_eq!(result.as_deref(), Some("1700000000"));
}

#[test]
fn latest_activity_empty_entries() {
    let entries: Vec<StatsEntry> = vec![];
    let result = latest_activity_timestamp(&entries);
    assert!(result.is_none());
}

#[test]
fn latest_activity_none_timestamps() {
    let entries = vec![make_stats_entry(None)];
    let result = latest_activity_timestamp(&entries);
    assert!(result.is_none());
}

// ---- latest_verification_timestamp ----

#[test]
fn latest_verification_with_runs() {
    let runs = vec![
        make_verification_run("1699900000"),
        make_verification_run("1700000000"),
    ];
    let result = latest_verification_timestamp(&runs);
    assert_eq!(result.as_deref(), Some("1700000000"));
}

#[test]
fn latest_verification_empty_runs() {
    let runs: Vec<VerificationRun> = vec![];
    let result = latest_verification_timestamp(&runs);
    assert!(result.is_none());
}

// ---- freshness_status ----

#[test]
fn freshness_status_missing_when_not_available() {
    let (status, age) = freshness_status(Some("1700000000"), false, NOW);
    assert_eq!(status, "missing");
    assert!(age.is_none());
}

#[test]
fn freshness_status_unknown_when_no_timestamp() {
    let (status, age) = freshness_status(None, true, NOW);
    assert_eq!(status, "unknown");
    assert!(age.is_none());
}

#[test]
fn freshness_status_fresh_when_recent() {
    // Within 24 hours
    let recent_ts = (NOW - 3600).to_string();
    let (status, age) = freshness_status(Some(&recent_ts), true, NOW);
    assert_eq!(status, "fresh");
    assert_eq!(age, Some(3600));
}

#[test]
fn freshness_status_aging_when_slightly_old() {
    // Between 1 day and 7 days
    let ts = (NOW - 2 * 24 * 3600).to_string();
    let (status, age) = freshness_status(Some(&ts), true, NOW);
    assert_eq!(status, "aging");
    assert!(age.unwrap() > 24 * 3600);
    assert!(age.unwrap() <= 7 * 24 * 3600);
}

#[test]
fn freshness_status_stale_when_very_old() {
    // Older than 7 days
    let ts = (NOW - 10 * 24 * 3600).to_string();
    let (status, age) = freshness_status(Some(&ts), true, NOW);
    assert_eq!(status, "stale");
    assert!(age.unwrap() > 7 * 24 * 3600);
}

#[test]
fn freshness_status_unknown_when_invalid_timestamp() {
    let (status, age) = freshness_status(Some("not-a-timestamp"), true, NOW);
    assert_eq!(status, "unknown");
    assert!(age.is_none());
}

#[test]
fn freshness_status_fresh_at_boundary() {
    // Exactly at the 24-hour boundary
    let ts = (NOW - 86400).to_string();
    let (status, _) = freshness_status(Some(&ts), true, NOW);
    assert_eq!(status, "fresh");
}

#[test]
fn freshness_status_aging_at_boundary() {
    // Exactly at the 7-day boundary
    let ts = (NOW - 7 * 86400).to_string();
    let (status, _) = freshness_status(Some(&ts), true, NOW);
    assert_eq!(status, "aging");
}

// ---- freshness_detail ----

#[test]
fn freshness_detail_json_output() {
    let detail = freshness_detail("snapshot", Some("1700000000"), true, NOW);
    assert_eq!(detail["label"], "snapshot");
    assert_eq!(detail["status"], "fresh");
    assert_eq!(detail["observed_at"], "1700000000");
    assert_eq!(detail["available"], true);
    assert!(detail["age_seconds"].is_number());
    assert!(detail["policy"].is_object());
}

#[test]
fn freshness_detail_missing() {
    let detail = freshness_detail("activity", None, false, NOW);
    assert_eq!(detail["status"], "missing");
    assert_eq!(detail["available"], false);
}

// ---- verification_is_stale ----

#[test]
fn verification_stale_when_old() {
    let runs = vec![make_verification_run(&(NOW - 10 * 86400).to_string())];
    assert!(verification_is_stale(&runs, NOW));
}

#[test]
fn verification_not_stale_when_recent() {
    let runs = vec![make_verification_run(&(NOW - 100).to_string())];
    assert!(!verification_is_stale(&runs, NOW));
}

#[test]
fn verification_not_stale_when_empty() {
    // Empty runs means available=false, so freshness_status returns "missing",
    // which does not match "stale" | "unknown".
    let runs: Vec<VerificationRun> = vec![];
    assert!(!verification_is_stale(&runs, NOW));
}

// ---- snapshot_is_stale ----

#[test]
fn snapshot_stale_when_old() {
    let project = make_project(100, 50, Some(&(NOW - 10 * 86400).to_string()), None, None);
    assert!(snapshot_is_stale(&project, NOW));
}

#[test]
fn snapshot_not_stale_when_recent() {
    let project = make_project(100, 50, Some(&(NOW - 100).to_string()), None, None);
    assert!(!snapshot_is_stale(&project, NOW));
}

#[test]
fn snapshot_not_stale_when_no_files() {
    // total_files=0 means available=false, so freshness returns "missing",
    // which does not match "stale" | "unknown".
    let project = make_project(0, 0, None, None, None);
    assert!(!snapshot_is_stale(&project, NOW));
}

#[test]
fn snapshot_stale_when_files_but_no_timestamp() {
    let project = make_project(100, 0, None, None, None);
    assert!(snapshot_is_stale(&project, NOW));
}

// ---- activity_is_stale ----

#[test]
fn activity_stale_when_old() {
    let project = make_project(100, 50, None, Some(&(NOW - 10 * 86400).to_string()), None);
    assert!(activity_is_stale(&project, NOW));
}

#[test]
fn activity_not_stale_when_recent() {
    let project = make_project(100, 50, None, Some(&(NOW - 100).to_string()), None);
    assert!(!activity_is_stale(&project, NOW));
}

#[test]
fn activity_not_stale_when_no_accessed_files() {
    // accessed_files=0 means available=false, so freshness returns "missing",
    // which does not match "stale" | "unknown".
    let project = make_project(100, 0, None, None, None);
    assert!(!activity_is_stale(&project, NOW));
}

#[test]
fn activity_stale_when_accessed_but_no_timestamp() {
    let project = make_project(100, 50, None, None, None);
    assert!(activity_is_stale(&project, NOW));
}
