use serde_json::{json, Value};

use crate::storage::queries::{StatsEntry, VerificationRun};

use super::{now_unix_secs, ProjectGuidanceState};

const FRESHNESS_RECENT_MAX_AGE_SECS: i64 = 24 * 60 * 60;
const FRESHNESS_AGING_MAX_AGE_SECS: i64 = 7 * 24 * 60 * 60;

fn parse_unix_timestamp(value: &str) -> Option<i64> {
    value.parse::<i64>().ok().filter(|ts| *ts >= 0)
}

fn max_timestamp_string<'a, I>(values: I) -> Option<String>
where
    I: Iterator<Item = &'a str>,
{
    values
        .filter_map(|value| parse_unix_timestamp(value).map(|ts| (ts, value)))
        .max_by_key(|(ts, _)| *ts)
        .map(|(_, value)| value.to_string())
}

pub(super) fn latest_activity_timestamp(entries: &[StatsEntry]) -> Option<String> {
    max_timestamp_string(
        entries
            .iter()
            .filter_map(|entry| entry.last_access_time.as_deref()),
    )
}

pub(super) fn latest_verification_timestamp(runs: &[VerificationRun]) -> Option<String> {
    max_timestamp_string(runs.iter().map(|run| run.finished_at.as_str()))
}

fn freshness_status(
    timestamp: Option<&str>,
    available: bool,
    now_secs: i64,
) -> (&'static str, Option<i64>) {
    if !available {
        return ("missing", None);
    }

    let Some(timestamp) = timestamp else {
        return ("unknown", None);
    };
    let Some(parsed) = parse_unix_timestamp(timestamp) else {
        return ("unknown", None);
    };
    let age_secs = now_secs.saturating_sub(parsed);
    let status = if age_secs <= FRESHNESS_RECENT_MAX_AGE_SECS {
        "fresh"
    } else if age_secs <= FRESHNESS_AGING_MAX_AGE_SECS {
        "aging"
    } else {
        "stale"
    };
    (status, Some(age_secs))
}

pub(super) fn freshness_detail(
    label: &'static str,
    timestamp: Option<&str>,
    available: bool,
    now_secs: i64,
) -> Value {
    let (status, age_seconds) = freshness_status(timestamp, available, now_secs);
    json!({
        "label": label,
        "status": status,
        "observed_at": timestamp,
        "age_seconds": age_seconds,
        "available": available,
    })
}

pub(super) fn verification_is_stale(runs: &[VerificationRun], now_secs: i64) -> bool {
    matches!(
        freshness_status(
            latest_verification_timestamp(runs).as_deref(),
            !runs.is_empty(),
            now_secs
        )
        .0,
        "stale" | "unknown"
    )
}

pub(super) fn snapshot_is_stale(project: &ProjectGuidanceState, now_secs: i64) -> bool {
    matches!(
        freshness_status(
            project.latest_snapshot_captured_at.as_deref(),
            project.total_files > 0,
            now_secs,
        )
        .0,
        "stale" | "unknown"
    )
}

pub(super) fn activity_is_stale(project: &ProjectGuidanceState, now_secs: i64) -> bool {
    matches!(
        freshness_status(
            project.latest_activity_at.as_deref(),
            project.accessed_files > 0,
            now_secs,
        )
        .0,
        "stale" | "unknown"
    )
}

pub(super) fn project_observation_layer(project: &ProjectGuidanceState) -> Value {
    let now_secs = now_unix_secs();
    let snapshot = freshness_detail(
        "snapshot",
        project.latest_snapshot_captured_at.as_deref(),
        project.total_files > 0,
        now_secs,
    );
    let activity = freshness_detail(
        "activity",
        project.latest_activity_at.as_deref(),
        project.accessed_files > 0,
        now_secs,
    );
    let verification = freshness_detail(
        "verification",
        project.latest_verification_at.as_deref(),
        project.latest_verification_at.is_some(),
        now_secs,
    );

    let mut evidence_gaps = Vec::new();
    if project.status != "monitoring" {
        evidence_gaps.push("monitor_not_running");
    }
    match snapshot["status"].as_str().unwrap_or("unknown") {
        "missing" => evidence_gaps.push("snapshot_missing"),
        "stale" | "unknown" => evidence_gaps.push("snapshot_stale"),
        _ => {}
    }
    match activity["status"].as_str().unwrap_or("unknown") {
        "missing" => evidence_gaps.push("activity_missing"),
        "stale" | "unknown" => evidence_gaps.push("activity_stale"),
        _ => {}
    }
    match verification["status"].as_str().unwrap_or("unknown") {
        "missing" => evidence_gaps.push("verification_missing"),
        "stale" | "unknown" => evidence_gaps.push("verification_stale"),
        _ => {}
    }

    let coverage_state = if snapshot["status"] == "missing" {
        "missing_snapshot"
    } else if activity["status"] == "missing" {
        "snapshot_without_activity"
    } else if verification["status"] == "missing" {
        "activity_without_verification"
    } else if evidence_gaps.iter().any(|gap| {
        matches!(
            *gap,
            "snapshot_stale" | "activity_stale" | "verification_stale"
        )
    }) {
        "stale_evidence"
    } else {
        "ready"
    };

    let analysis_state = if snapshot["status"] == "missing" {
        "not_ready"
    } else if activity["status"] == "missing" {
        "insufficient_activity"
    } else if coverage_state == "stale_evidence" {
        "stale"
    } else {
        "ready"
    };

    json!({
        "status": "available",
        "analysis_state": analysis_state,
        "coverage_state": coverage_state,
        "snapshot_available": project.total_files > 0,
        "activity_available": project.accessed_files > 0,
        "verification_available": project.latest_verification_at.is_some(),
        "monitoring_active": project.status == "monitoring",
        "total_files": project.total_files,
        "accessed_files": project.accessed_files,
        "unused_files": project.unused_files,
        "freshness": {
            "snapshot": snapshot,
            "activity": activity,
            "verification": verification,
        },
        "evidence_gaps": evidence_gaps,
    })
}
