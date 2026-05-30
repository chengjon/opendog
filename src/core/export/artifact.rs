use super::types::{ExportFormat, ExportView, PortableProjectExport};
use crate::contracts::PORTABLE_PROJECT_EXPORT_V1;
use crate::core::stats::ProjectSummary;
use crate::error::Result;
use crate::storage::queries::StatsEntry;

pub fn build_portable_export(
    project_id: &str,
    format: ExportFormat,
    view: ExportView,
    summary: ProjectSummary,
    rows: Vec<StatsEntry>,
) -> PortableProjectExport {
    PortableProjectExport {
        schema_version: PORTABLE_PROJECT_EXPORT_V1.to_string(),
        project_id: project_id.to_string(),
        format: format.as_str().to_string(),
        view: view.as_str().to_string(),
        generated_at: now_iso(),
        row_count: rows.len(),
        summary,
        rows,
    }
}

pub fn render_json_export(export: &PortableProjectExport) -> Result<String> {
    Ok(serde_json::to_string_pretty(export)?)
}

fn now_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}
