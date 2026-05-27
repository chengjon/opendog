use crate::error::Result;
use crate::storage::database::Database;

use super::{
    now_secs, window_bounds, ActivityRollupDay, ActivityRollupReport, ActivityRollupSummary,
    ReportWindow,
};

const DAY_SECS: i64 = 24 * 60 * 60;

pub fn get_activity_rollup_report(
    db: &Database,
    window: ReportWindow,
    limit: usize,
) -> Result<ActivityRollupReport> {
    get_activity_rollup_report_at(db, window, now_secs(), limit)
}

pub fn get_activity_rollup_report_at(
    db: &Database,
    window: ReportWindow,
    end_ts: i64,
    limit: usize,
) -> Result<ActivityRollupReport> {
    let (start_ts, end_ts) = window_bounds(window, end_ts);
    let start_day = day_start(start_ts);
    let end_day = day_start(end_ts);
    let bucket_count = ((end_day - start_day) / DAY_SECS + 1).max(0) as usize;
    let mut all_days = activity_rollup_days(db, start_day, end_day)?;
    let rollup_days = all_days.len();
    let total_access_count = all_days.iter().map(|day| day.access_count).sum();
    let total_modification_count = all_days.iter().map(|day| day.modification_count).sum();
    let total_event_count = all_days.iter().map(|day| day.event_count).sum();
    let limit = limit.max(1);
    let truncated = rollup_days > limit;
    all_days.truncate(limit);

    let summary = ActivityRollupSummary {
        bucket_size: "1d".to_string(),
        bucket_count,
        total_access_count,
        total_modification_count,
        total_event_count,
        rollup_days,
        returned_days: all_days.len(),
        truncated,
    };

    Ok(ActivityRollupReport {
        window: window.as_str().to_string(),
        start_time: start_ts.to_string(),
        end_time: end_ts.to_string(),
        summary,
        days: all_days,
    })
}

fn activity_rollup_days(
    db: &Database,
    start_day: i64,
    end_day: i64,
) -> Result<Vec<ActivityRollupDay>> {
    db.prepare_and_query(
        "SELECT
             day_start,
             COALESCE(SUM(CASE WHEN source_table = 'file_sightings' THEN row_count ELSE 0 END), 0) AS access_count,
             COALESCE(SUM(CASE WHEN source_table = 'file_events' AND activity = 'modify' THEN row_count ELSE 0 END), 0) AS modification_count,
             COALESCE(SUM(CASE WHEN source_table = 'file_events' THEN row_count ELSE 0 END), 0) AS event_count
         FROM activity_daily_rollups
         WHERE day_start BETWEEN ?1 AND ?2
         GROUP BY day_start
         ORDER BY day_start ASC",
        rusqlite::params![start_day, end_day],
        |row| {
            Ok(ActivityRollupDay {
                day_start: row.get(0)?,
                access_count: row.get(1)?,
                modification_count: row.get(2)?,
                event_count: row.get(3)?,
            })
        },
    )
}

fn day_start(ts: i64) -> i64 {
    ts.div_euclid(DAY_SECS) * DAY_SECS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_start_floors_positive_timestamp() {
        assert_eq!(day_start(DAY_SECS + 123), DAY_SECS);
    }
}
