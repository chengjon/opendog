use rmcp::handler::server::wrapper::Json;
use serde_json::Value;

use crate::control::DaemonClient;
use crate::core::report::{self, ReportWindow};
use crate::core::stats;
use crate::error::OpenDogError;

use super::{
    error_json_for, latest_verification_runs_for_project, snapshot_comparison_payload,
    stats_payload, time_window_report_payload, unused_files_payload, usage_trends_payload,
    OpenDogServer, MCP_SNAPSHOT_COMPARE_V1, MCP_STATS_V1, MCP_TIME_WINDOW_REPORT_V1,
    MCP_UNUSED_FILES_V1, MCP_USAGE_TRENDS_V1,
};

pub(super) fn handle_get_stats(server: &OpenDogServer, id: &str) -> Json<Value> {
    match DaemonClient::new().get_stats(id) {
        Ok((summary, entries)) => {
            let (root_path, verification_runs) = server
                .get_project(id)
                .map(|(db, info)| (info.root_path, latest_verification_runs_for_project(&db)))
                .unwrap_or_else(|_| (std::path::PathBuf::from("."), Vec::new()));
            return Json(stats_payload(
                id,
                &summary,
                &entries,
                &root_path,
                &verification_runs,
            ));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_STATS_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, info) = server.get_project(id)?;
        let summary = stats::get_summary(&db)?;
        let entries = stats::get_stats(&db)?;
        let verification_runs = latest_verification_runs_for_project(&db);
        Ok((summary, entries, info.root_path, verification_runs))
    })();
    match result {
        Ok((summary, entries, root_path, verification_runs)) => Json(stats_payload(
            id,
            &summary,
            &entries,
            &root_path,
            &verification_runs,
        )),
        Err(e) => error_json_for(MCP_STATS_V1, Some(id), &e),
    }
}

pub(super) fn handle_get_unused_files(server: &OpenDogServer, id: &str) -> Json<Value> {
    match DaemonClient::new().get_unused_files(id) {
        Ok(unused) => {
            let (root_path, verification_runs) = server
                .get_project(id)
                .map(|(db, info)| (info.root_path, latest_verification_runs_for_project(&db)))
                .unwrap_or_else(|_| (std::path::PathBuf::from("."), Vec::new()));
            return Json(unused_files_payload(
                id,
                &unused,
                &root_path,
                &verification_runs,
            ));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_UNUSED_FILES_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, info) = server.get_project(id)?;
        let unused = stats::get_unused_files(&db)?;
        let verification_runs = latest_verification_runs_for_project(&db);
        Ok((unused, info.root_path, verification_runs))
    })();
    match result {
        Ok((unused, root_path, verification_runs)) => Json(unused_files_payload(
            id,
            &unused,
            &root_path,
            &verification_runs,
        )),
        Err(e) => error_json_for(MCP_UNUSED_FILES_V1, Some(id), &e),
    }
}

pub(super) fn handle_get_time_window_report(
    server: &OpenDogServer,
    id: &str,
    window: Option<String>,
    limit: Option<usize>,
) -> Json<Value> {
    let window_name = window.unwrap_or_else(|| "24h".to_string());
    let window = match ReportWindow::parse(&window_name) {
        Ok(window) => window,
        Err(e) => return error_json_for(MCP_TIME_WINDOW_REPORT_V1, Some(id), &e),
    };
    let limit = limit.unwrap_or(10).max(1);

    match DaemonClient::new().get_time_window_report(id, window, limit) {
        Ok(report) => {
            return Json(time_window_report_payload(
                MCP_TIME_WINDOW_REPORT_V1,
                id,
                &report,
            ));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_TIME_WINDOW_REPORT_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, _) = server.get_project(id)?;
        report::get_time_window_report(&db, window, limit)
    })();
    match result {
        Ok(report) => Json(time_window_report_payload(
            MCP_TIME_WINDOW_REPORT_V1,
            id,
            &report,
        )),
        Err(e) => error_json_for(MCP_TIME_WINDOW_REPORT_V1, Some(id), &e),
    }
}

pub(super) fn handle_compare_snapshots(
    server: &OpenDogServer,
    id: &str,
    base_run_id: Option<i64>,
    head_run_id: Option<i64>,
    limit: Option<usize>,
) -> Json<Value> {
    let limit = limit.unwrap_or(20).max(1);

    match DaemonClient::new().compare_snapshots(id, base_run_id, head_run_id, limit) {
        Ok(comparison) => {
            return Json(snapshot_comparison_payload(
                MCP_SNAPSHOT_COMPARE_V1,
                id,
                &comparison,
            ));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_SNAPSHOT_COMPARE_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, _) = server.get_project(id)?;
        match (base_run_id, head_run_id) {
            (None, None) => report::compare_latest_snapshots(&db, limit),
            (Some(base_run_id), Some(head_run_id)) => {
                report::compare_snapshot_runs(&db, base_run_id, head_run_id, limit)
            }
            _ => Err(OpenDogError::InvalidInput(
                "base_run_id and head_run_id must be provided together".to_string(),
            )),
        }
    })();
    match result {
        Ok(comparison) => Json(snapshot_comparison_payload(
            MCP_SNAPSHOT_COMPARE_V1,
            id,
            &comparison,
        )),
        Err(e) => error_json_for(MCP_SNAPSHOT_COMPARE_V1, Some(id), &e),
    }
}

pub(super) fn handle_get_usage_trends(
    server: &OpenDogServer,
    id: &str,
    window: Option<String>,
    limit: Option<usize>,
) -> Json<Value> {
    let window_name = window.unwrap_or_else(|| "7d".to_string());
    let window = match ReportWindow::parse(&window_name) {
        Ok(window) => window,
        Err(e) => return error_json_for(MCP_USAGE_TRENDS_V1, Some(id), &e),
    };
    let limit = limit.unwrap_or(10).max(1);

    match DaemonClient::new().get_usage_trends(id, window, limit) {
        Ok(report) => return Json(usage_trends_payload(MCP_USAGE_TRENDS_V1, id, &report)),
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_USAGE_TRENDS_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, _) = server.get_project(id)?;
        report::get_usage_trend_report(&db, window, limit)
    })();
    match result {
        Ok(report) => Json(usage_trends_payload(MCP_USAGE_TRENDS_V1, id, &report)),
        Err(e) => error_json_for(MCP_USAGE_TRENDS_V1, Some(id), &e),
    }
}
