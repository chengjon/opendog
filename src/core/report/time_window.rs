use crate::error::Result;
use crate::storage::database::Database;

use super::{
    now_secs, window_bounds, ReportWindow, TimeWindowFile, TimeWindowReport, TimeWindowSummary,
};
use std::collections::HashMap;

pub fn get_time_window_report(
    db: &Database,
    window: ReportWindow,
    limit: usize,
) -> Result<TimeWindowReport> {
    get_time_window_report_at(db, window, now_secs(), limit)
}

pub fn get_time_window_report_at(
    db: &Database,
    window: ReportWindow,
    end_ts: i64,
    limit: usize,
) -> Result<TimeWindowReport> {
    let (start_ts, end_ts) = window_bounds(window, end_ts);
    let summary = TimeWindowSummary {
        total_sightings: count_rows_between(
            db,
            "SELECT COUNT(*) FROM file_sightings WHERE CAST(seen_at AS INTEGER) BETWEEN ?1 AND ?2",
            start_ts,
            end_ts,
        )?,
        unique_files_accessed: count_rows_between(
            db,
            "SELECT COUNT(DISTINCT file_path) FROM file_sightings WHERE CAST(seen_at AS INTEGER) BETWEEN ?1 AND ?2",
            start_ts,
            end_ts,
        )?,
        unique_processes: count_rows_between(
            db,
            "SELECT COUNT(DISTINCT process_name) FROM file_sightings WHERE CAST(seen_at AS INTEGER) BETWEEN ?1 AND ?2",
            start_ts,
            end_ts,
        )?,
        modification_events: count_rows_between(
            db,
            "SELECT COUNT(*) FROM file_events WHERE event_type = 'modify' AND CAST(event_time AS INTEGER) BETWEEN ?1 AND ?2",
            start_ts,
            end_ts,
        )?,
        modified_files: count_rows_between(
            db,
            "SELECT COUNT(DISTINCT file_path) FROM file_events WHERE event_type = 'modify' AND CAST(event_time AS INTEGER) BETWEEN ?1 AND ?2",
            start_ts,
            end_ts,
        )?,
    };

    let mut files: HashMap<String, TimeWindowFile> = HashMap::new();
    for (file_path, access_count, last_seen_at) in access_counts(db, start_ts, end_ts)? {
        files.insert(
            file_path.clone(),
            TimeWindowFile {
                file_path,
                access_count,
                modification_count: 0,
                last_seen_at: Some(last_seen_at),
                last_modified_at: None,
            },
        );
    }

    for (file_path, modification_count, last_modified_at) in
        modification_counts(db, start_ts, end_ts)?
    {
        let entry = files
            .entry(file_path.clone())
            .or_insert_with(|| TimeWindowFile {
                file_path,
                access_count: 0,
                modification_count: 0,
                last_seen_at: None,
                last_modified_at: None,
            });
        entry.modification_count = modification_count;
        entry.last_modified_at = Some(last_modified_at);
    }

    let mut files: Vec<TimeWindowFile> = files.into_values().collect();
    files.sort_by(|left, right| {
        right
            .access_count
            .cmp(&left.access_count)
            .then_with(|| right.modification_count.cmp(&left.modification_count))
            .then_with(|| left.file_path.cmp(&right.file_path))
    });
    files.truncate(limit.max(1));

    Ok(TimeWindowReport {
        window: window.as_str().to_string(),
        start_time: start_ts.to_string(),
        end_time: end_ts.to_string(),
        summary,
        files,
    })
}

fn count_rows_between(db: &Database, sql: &str, start_ts: i64, end_ts: i64) -> Result<i64> {
    db.query_row(sql, rusqlite::params![start_ts, end_ts], |row| row.get(0))
}

fn access_counts(db: &Database, start_ts: i64, end_ts: i64) -> Result<Vec<(String, i64, String)>> {
    db.prepare_and_query(
        "SELECT file_path, COUNT(*) AS access_count, MAX(CAST(seen_at AS INTEGER)) AS last_seen_at
         FROM file_sightings
         WHERE CAST(seen_at AS INTEGER) BETWEEN ?1 AND ?2
         GROUP BY file_path
         ORDER BY access_count DESC, file_path",
        rusqlite::params![start_ts, end_ts],
        |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, i64>(2)?.to_string())),
    )
}

fn modification_counts(
    db: &Database,
    start_ts: i64,
    end_ts: i64,
) -> Result<Vec<(String, i64, String)>> {
    db.prepare_and_query(
        "SELECT file_path, COUNT(*) AS modification_count, MAX(CAST(event_time AS INTEGER)) AS last_modified_at
         FROM file_events
         WHERE event_type = 'modify' AND CAST(event_time AS INTEGER) BETWEEN ?1 AND ?2
         GROUP BY file_path
         ORDER BY modification_count DESC, file_path",
        rusqlite::params![start_ts, end_ts],
        |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, i64>(2)?.to_string())),
    )
}
