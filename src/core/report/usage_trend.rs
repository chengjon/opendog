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
        BucketCountQuery {
            table: "file_sightings",
            time_column: "seen_at",
            extra_filter: None,
            start_ts,
            end_ts,
            bucket_size,
            limit,
        },
    )?;
    let modify_buckets = bucket_counts(
        db,
        BucketCountQuery {
            table: "file_events",
            time_column: "event_time",
            extra_filter: Some("event_type = 'modify'"),
            start_ts,
            end_ts,
            bucket_size,
            limit,
        },
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
            .find(|bucket| bucket.bucket_start == bucket_start)
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
            .find(|bucket| bucket.bucket_start == bucket_start)
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

struct BucketCountQuery<'a> {
    table: &'a str,
    time_column: &'a str,
    extra_filter: Option<&'a str>,
    start_ts: i64,
    end_ts: i64,
    bucket_size: i64,
    limit: usize,
}

fn bucket_counts(db: &Database, query: BucketCountQuery<'_>) -> Result<Vec<(String, i64, i64)>> {
    let filter = query
        .extra_filter
        .map(|clause| format!("{} AND ", clause))
        .unwrap_or_default();
    let table = query.table;
    let time_column = query.time_column;
    let sql = format!(
        "SELECT file_path,
                (?1 + ((CAST({time_column} AS INTEGER) - ?1) / ?3) * ?3) AS bucket_start,
                COUNT(*) AS bucket_count
         FROM {table}
         WHERE {filter}CAST({time_column} AS INTEGER) BETWEEN ?1 AND ?2
         GROUP BY file_path, bucket_start
         ORDER BY file_path, bucket_start
         LIMIT ?4"
    );
    db.prepare_and_query(
        &sql,
        rusqlite::params![
            query.start_ts,
            query.end_ts,
            query.bucket_size,
            query.limit as i64
        ],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
}

fn build_empty_buckets(start_ts: i64, bucket_count: usize, bucket_size: i64) -> Vec<TrendBucket> {
    (0..bucket_count)
        .map(|index| TrendBucket {
            bucket_start: start_ts + index as i64 * bucket_size,
            access_count: 0,
            modification_count: 0,
        })
        .collect()
}

fn bucket_access_count(buckets: &[TrendBucket], bucket_start: i64) -> i64 {
    buckets
        .iter()
        .find(|bucket| bucket.bucket_start == bucket_start)
        .map(|bucket| bucket.access_count)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_empty_buckets_generates_correct_count() {
        let buckets = build_empty_buckets(1000, 5, 60);
        assert_eq!(buckets.len(), 5);
    }

    #[test]
    fn build_empty_buckets_starts_at_start_ts() {
        let buckets = build_empty_buckets(1000, 3, 60);
        assert_eq!(buckets[0].bucket_start, 1000);
    }

    #[test]
    fn build_empty_buckets_spaces_by_bucket_size() {
        let buckets = build_empty_buckets(0, 4, 100);
        let starts: Vec<i64> = buckets.iter().map(|b| b.bucket_start).collect();
        assert_eq!(starts, vec![0, 100, 200, 300]);
    }

    #[test]
    fn build_empty_buckets_all_counts_zero() {
        let buckets = build_empty_buckets(0, 10, 30);
        for bucket in &buckets {
            assert_eq!(bucket.access_count, 0);
            assert_eq!(bucket.modification_count, 0);
        }
    }

    #[test]
    fn build_empty_buckets_single_bucket() {
        let buckets = build_empty_buckets(500, 1, 200);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].bucket_start, 500);
    }

    #[test]
    fn build_empty_buckets_zero_buckets() {
        let buckets = build_empty_buckets(0, 0, 60);
        assert!(buckets.is_empty());
    }

    #[test]
    fn bucket_access_count_returns_matching_count() {
        let buckets = vec![
            TrendBucket {
                bucket_start: 0,
                access_count: 5,
                modification_count: 0,
            },
            TrendBucket {
                bucket_start: 60,
                access_count: 10,
                modification_count: 0,
            },
            TrendBucket {
                bucket_start: 120,
                access_count: 3,
                modification_count: 0,
            },
        ];
        assert_eq!(bucket_access_count(&buckets, 60), 10);
    }

    #[test]
    fn bucket_access_count_returns_zero_for_missing() {
        let buckets = vec![TrendBucket {
            bucket_start: 0,
            access_count: 5,
            modification_count: 0,
        }];
        assert_eq!(bucket_access_count(&buckets, 999), 0);
    }

    #[test]
    fn bucket_access_count_empty_slice() {
        let buckets: Vec<TrendBucket> = vec![];
        assert_eq!(bucket_access_count(&buckets, 0), 0);
    }

    #[test]
    fn bucket_access_count_first_bucket() {
        let buckets = vec![
            TrendBucket {
                bucket_start: 100,
                access_count: 7,
                modification_count: 0,
            },
            TrendBucket {
                bucket_start: 200,
                access_count: 8,
                modification_count: 0,
            },
        ];
        assert_eq!(bucket_access_count(&buckets, 100), 7);
    }
}
