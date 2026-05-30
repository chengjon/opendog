use std::path::Path;

use crate::contracts::CLI_EXPORT_PROJECT_EVIDENCE_V1;
use crate::core::export::{self, ExportFormat, ExportView};
use crate::core::project::ProjectManager;
use crate::core::stats;
use crate::error::OpenDogError;
use crate::mcp::export_project_evidence_payload;

pub(in crate::cli) fn cmd_export(
    pm: &ProjectManager,
    id: &str,
    format: &str,
    view: &str,
    output_path: &str,
    min_access_count: i64,
) -> Result<(), OpenDogError> {
    let format = ExportFormat::parse(format)?;
    let view = ExportView::parse(view)?;
    let db = pm.open_project_db(id)?;
    let summary = stats::get_summary(&db)?;
    let rows = export::export_rows(&db, view, min_access_count)?;
    let artifact = export::build_portable_export(id, format, view, summary, rows.clone());
    let content = match format {
        ExportFormat::Json => export::render_json_export(&artifact)?,
        ExportFormat::Csv => export::render_csv_export(&rows),
    };

    let bytes_written = export::write_export_file(Path::new(output_path), &content)?;
    let payload = export_project_evidence_payload(
        CLI_EXPORT_PROJECT_EVIDENCE_V1,
        &artifact,
        output_path,
        bytes_written,
        &content,
    );

    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}
