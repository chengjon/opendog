use super::*;

impl MonitorController {
    pub fn get_stats(&self, id: &str) -> Result<(ProjectSummary, Vec<StatsEntry>)> {
        self.with_project_db(id, |db| {
            let summary = stats::get_summary(db)?;
            let entries = stats::get_stats(db)?;
            Ok((summary, entries))
        })
    }

    pub fn get_unused_files(&self, id: &str) -> Result<Vec<StatsEntry>> {
        self.with_project_db(id, stats::get_unused_files)
    }

    pub fn get_time_window_report(
        &self,
        id: &str,
        window: ReportWindow,
        limit: usize,
    ) -> Result<TimeWindowReport> {
        self.with_project_db(id, |db| report::get_time_window_report(db, window, limit))
    }

    pub fn compare_snapshots(
        &self,
        id: &str,
        base_run_id: Option<i64>,
        head_run_id: Option<i64>,
        limit: usize,
    ) -> Result<SnapshotComparison> {
        self.with_project_db(id, |db| match (base_run_id, head_run_id) {
            (None, None) => report::compare_latest_snapshots(db, limit),
            (Some(base), Some(head)) => report::compare_snapshot_runs(db, base, head, limit),
            _ => Err(OpenDogError::InvalidInput(
                "base_run_id and head_run_id must be provided together".to_string(),
            )),
        })
    }

    pub fn get_usage_trends(
        &self,
        id: &str,
        window: ReportWindow,
        limit: usize,
    ) -> Result<UsageTrendReport> {
        self.with_project_db(id, |db| report::get_usage_trend_report(db, window, limit))
    }

    pub fn cleanup_project_data(
        &self,
        id: &str,
        request: ProjectDataCleanupRequest,
    ) -> Result<ProjectDataCleanupResult> {
        self.with_project_db(id, |db| retention::cleanup_project_data(db, &request))
    }
}
