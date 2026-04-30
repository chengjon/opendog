use rmcp::handler::server::wrapper::Json;
use serde_json::Value;
use std::path::Path;

use crate::core::export::{self, ExportFormat, ExportView};
use crate::core::retention::{self, CleanupScope, ProjectDataCleanupRequest};
use crate::core::stats;
use crate::error::OpenDogError;

use super::{
    cleanup_project_data_payload, error_json_for, export_project_evidence_payload, OpenDogServer,
    MCP_CLEANUP_PROJECT_DATA_V1, MCP_EXPORT_PROJECT_EVIDENCE_V1,
};

pub(super) fn handle_export_project_evidence(
    server: &OpenDogServer,
    id: &str,
    format: String,
    view: Option<String>,
    output_path: String,
    min_access_count: Option<i64>,
) -> Json<Value> {
    let format = match ExportFormat::parse(&format) {
        Ok(value) => value,
        Err(e) => return error_json_for(MCP_EXPORT_PROJECT_EVIDENCE_V1, Some(id), &e),
    };
    let view_name = view.unwrap_or_else(|| "stats".to_string());
    let view = match ExportView::parse(&view_name) {
        Ok(value) => value,
        Err(e) => return error_json_for(MCP_EXPORT_PROJECT_EVIDENCE_V1, Some(id), &e),
    };

    let result = (|| -> crate::error::Result<Value> {
        let (db, _) = server.get_project(id)?;
        let summary = stats::get_summary(&db)?;
        let rows = export::export_rows(&db, view, min_access_count.unwrap_or(5))?;
        let artifact = export::build_portable_export(id, format, view, summary, rows.clone());
        let content = match format {
            ExportFormat::Json => export::render_json_export(&artifact)?,
            ExportFormat::Csv => export::render_csv_export(&rows),
        };
        let bytes_written = export::write_export_file(Path::new(&output_path), &content)?;
        Ok(export_project_evidence_payload(
            MCP_EXPORT_PROJECT_EVIDENCE_V1,
            id,
            format.as_str(),
            view.as_str(),
            &output_path,
            bytes_written,
            artifact.row_count,
            &artifact.summary,
            &content,
        ))
    })();

    match result {
        Ok(payload) => Json(payload),
        Err(e) => error_json_for(MCP_EXPORT_PROJECT_EVIDENCE_V1, Some(id), &e),
    }
}

pub(super) fn handle_cleanup_project_data(
    server: &OpenDogServer,
    id: &str,
    scope: String,
    older_than_days: Option<i64>,
    keep_snapshot_runs: Option<usize>,
    vacuum: Option<bool>,
    dry_run: Option<bool>,
) -> Json<Value> {
    let scope = match CleanupScope::parse(&scope) {
        Ok(scope) => scope,
        Err(e) => return error_json_for(MCP_CLEANUP_PROJECT_DATA_V1, Some(id), &e),
    };
    let request = ProjectDataCleanupRequest {
        scope,
        older_than_days,
        keep_snapshot_runs,
        vacuum: vacuum.unwrap_or(false),
        dry_run: dry_run.unwrap_or(true),
    };

    match crate::control::DaemonClient::new().cleanup_project_data(id, request.clone()) {
        Ok(result) => {
            return Json(cleanup_project_data_payload(
                MCP_CLEANUP_PROJECT_DATA_V1,
                id,
                &result,
            ));
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return error_json_for(MCP_CLEANUP_PROJECT_DATA_V1, Some(id), &e),
    }

    let result = (|| -> crate::error::Result<_> {
        let (db, _) = server.get_project(id)?;
        retention::cleanup_project_data(&db, &request)
    })();
    match result {
        Ok(result) => Json(cleanup_project_data_payload(
            MCP_CLEANUP_PROJECT_DATA_V1,
            id,
            &result,
        )),
        Err(e) => error_json_for(MCP_CLEANUP_PROJECT_DATA_V1, Some(id), &e),
    }
}
