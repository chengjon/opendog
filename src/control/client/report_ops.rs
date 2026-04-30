use crate::core::report::{ReportWindow, SnapshotComparison, TimeWindowReport, UsageTrendReport};
use crate::core::stats::ProjectSummary;
use crate::error::{OpenDogError, Result};
use crate::storage::queries::StatsEntry;

use super::{ControlRequest, ControlResponse, DaemonClient};

impl DaemonClient {
    pub fn get_stats(&self, id: &str) -> Result<(ProjectSummary, Vec<StatsEntry>)> {
        match self.send(ControlRequest::GetStats { id: id.to_string() })? {
            ControlResponse::Stats {
                summary, entries, ..
            } => Ok((summary, entries)),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon stats response: {:?}",
                response
            ))),
        }
    }

    pub fn get_unused_files(&self, id: &str) -> Result<Vec<StatsEntry>> {
        match self.send(ControlRequest::GetUnusedFiles { id: id.to_string() })? {
            ControlResponse::UnusedFiles { entries, .. } => Ok(entries),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon unused response: {:?}",
                response
            ))),
        }
    }

    pub fn get_time_window_report(
        &self,
        id: &str,
        window: ReportWindow,
        limit: usize,
    ) -> Result<TimeWindowReport> {
        match self.send(ControlRequest::GetTimeWindowReport {
            id: id.to_string(),
            window: window.as_str().to_string(),
            limit,
        })? {
            ControlResponse::TimeWindowReport { report, .. } => Ok(report),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon time-window report response: {:?}",
                response
            ))),
        }
    }

    pub fn compare_snapshots(
        &self,
        id: &str,
        base_run_id: Option<i64>,
        head_run_id: Option<i64>,
        limit: usize,
    ) -> Result<SnapshotComparison> {
        match self.send(ControlRequest::CompareSnapshots {
            id: id.to_string(),
            base_run_id,
            head_run_id,
            limit,
        })? {
            ControlResponse::SnapshotComparison { comparison, .. } => Ok(comparison),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon snapshot comparison response: {:?}",
                response
            ))),
        }
    }

    pub fn get_usage_trends(
        &self,
        id: &str,
        window: ReportWindow,
        limit: usize,
    ) -> Result<UsageTrendReport> {
        match self.send(ControlRequest::GetUsageTrends {
            id: id.to_string(),
            window: window.as_str().to_string(),
            limit,
        })? {
            ControlResponse::UsageTrends { report, .. } => Ok(report),
            ControlResponse::Error { message } => Err(OpenDogError::RemoteControl(message)),
            response => Err(OpenDogError::RemoteControl(format!(
                "Unexpected daemon usage-trends response: {:?}",
                response
            ))),
        }
    }
}
