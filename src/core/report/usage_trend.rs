use crate::error::Result;
use crate::storage::database::Database;

use super::{
    now_secs, window_bounds, FileTrend, ReportWindow, TrendBucket, TrendSummary, UsageTrendReport,
};
use std::collections::HashMap;

pub fn get_usage_trend_report(
    db: &Database,
    window: ReportWindow,
    limit: usize,
) -> Result<UsageTrendReport> {
    get_usage_trend_report_at(db, window, now_secs(), limit)
}

pub fn get_usage_trend_report_at(
    db: &Database,
    window: ReportWindow,
    end_ts: i64,
    limit: usize,
) -> Result<UsageTrendReport> {
    let (start_ts, end_ts) = window_bounds(window, end_ts);
    let bucket_size = window.bucket_size_secs();
    let bucket_count = ((window.duration_secs() + bucket_size - 1) / bucket_size) as usize;

    let access_buckets = bucket_counts(
        db,
        "file_sightings",
        "seen_at",
        None,
        start_ts,
        end_ts,
        bucket_size,
    )?;
    let modify_buckets = bucket_counts(
        db,
        "file_events",
        "event_time",
        Some("event_type = 'modify'"),
        start_ts,
        end_ts,
        bucket_size,
    )?;

    let mut files: HashMap<String, FileTrend> = HashMap::new();
    for (file_path, bucket_start, access_count) in access_buckets {
        let entry = files.entry(file_path.clone()).or_insert_with(|| FileTrend {
            file_path,
            total_access_count: 0,
            total_modification_count: 0,
            current_bucket_access_count: 0,
            previous_bucket_access_count: 0,
            delta_access_count: 0,
            buckets: build_empty_buckets(start_ts, bucket_count, bucket_size),
        });
        entry.total_access_count += access_count;
        if let Some(bucket) = entry
            .buckets
            .iter_mut()
            .find(|bucket| bucket.bucket_start == bucket_start.to_string())
        {
            bucket.access_count = access_count;
        }
    }

    for (file_path, bucket_start, modification_count) in modify_buckets {
        let entry = files.entry(file_path.clone()).or_insert_with(|| FileTrend {
            file_path,
            total_access_count: 0,
            total_modification_count: 0,
            current_bucket_access_count: 0,
            previous_bucket_access_count: 0,
            delta_access_count: 0,
            buckets: build_empty_buckets(start_ts, bucket_count, bucket_size),
        });
        entry.total_modification_count += modification_count;
        if let Some(bucket) = entry
            .buckets
            .iter_mut()
            .find(|bucket| bucket.bucket_start == bucket_start.to_string())
        {
            bucket.modification_count = modification_count;
        }
    }

    let current_bucket_start = start_ts + ((end_ts - start_ts) / bucket_size) * bucket_size;
    let previous_bucket_start = current_bucket_start.saturating_sub(bucket_size);
    let mut files: Vec<FileTrend> = files
        .into_values()
        .map(|mut file| {
            file.current_bucket_access_count =
                bucket_access_count(&file.buckets, current_bucket_start);
            file.previous_bucket_access_count =
                bucket_access_count(&file.buckets, previous_bucket_start);
            file.delta_access_count =
                file.current_bucket_access_count - file.previous_bucket_access_count;
            file
        })
        .collect();
    files.sort_by(|left, right| {
        right
            .total_access_count
            .cmp(&left.total_access_count)
            .then_with(|| right.delta_access_count.cmp(&left.delta_access_count))
            .then_with(|| {
                right
                    .total_modification_count
                    .cmp(&left.total_modification_count)
            })
            .then_with(|| left.file_path.cmp(&right.file_path))
    });
    files.truncate(limit.max(1));

    let summary = TrendSummary {
        bucket_size: window.bucket_size_label().to_string(),
        bucket_count,
        total_access_count: files.iter().map(|file| file.total_access_count).sum(),
        total_modification_count: files.iter().map(|file| file.total_modification_count).sum(),
        tracked_files: files.len(),
    };

    Ok(UsageTrendReport {
        window: window.as_str().to_string(),
        start_time: start_ts.to_string(),
        end_time: end_ts.to_string(),
        summary,
        files,
    })
}

fn bucket_counts(
    db: &Database,
    table: &str,
    time_column: &str,
    extra_filter: Option<&str>,
    start_ts: i64,
    end_ts: i64,
    bucket_size: i64,
) -> Result<Vec<(String, i64, i64)>> {
    let filter = extra_filter
        .map(|clause| format!("{} AND ", clause))
        .unwrap_or_default();
    let sql = format!(
        "SELECT file_path,
                (?1 + ((CAST({time_column} AS INTEGER) - ?1) / ?3) * ?3) AS bucket_start,
                COUNT(*) AS bucket_count
         FROM {table}
         WHERE {filter}CAST({time_column} AS INTEGER) BETWEEN ?1 AND ?2
         GROUP BY file_path, bucket_start
         ORDER BY file_path, bucket_start"
    );
    db.prepare_and_query(
        &sql,
        rusqlite::params![start_ts, end_ts, bucket_size],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
}

fn build_empty_buckets(start_ts: i64, bucket_count: usize, bucket_size: i64) -> Vec<TrendBucket> {
    (0..bucket_count)
        .map(|index| TrendBucket {
            bucket_start: (start_ts + index as i64 * bucket_size).to_string(),
            access_count: 0,
            modification_count: 0,
        })
        .collect()
}

fn bucket_access_count(buckets: &[TrendBucket], bucket_start: i64) -> i64 {
    buckets
        .iter()
        .find(|bucket| bucket.bucket_start == bucket_start.to_string())
        .map(|bucket| bucket.access_count)
        .unwrap_or(0)
}
