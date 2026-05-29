use crate::contracts::PORTABLE_PROJECT_EXPORT_V1;
use crate::core::stats;
use crate::core::stats::ProjectSummary;
use crate::error::{OpenDogError, Result};
use crate::storage::database::Database;
use crate::storage::queries::StatsEntry;
use serde::{Deserialize, Serialize};
use std::path::Path;

const CSV_COLUMNS: &[&str] = &[
    "file_path",
    "file_type",
    "size",
    "access_count",
    "estimated_duration_ms",
    "modification_count",
    "last_access_time",
    "first_seen_time",
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportView {
    Stats,
    Unused,
    Core,
}

impl ExportView {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "stats" => Ok(Self::Stats),
            "unused" => Ok(Self::Unused),
            "core" => Ok(Self::Core),
            _ => Err(OpenDogError::InvalidInput(format!(
                "view must be one of: stats, unused, core; got '{}'",
                value
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stats => "stats",
            Self::Unused => "unused",
            Self::Core => "core",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Csv,
}

impl ExportFormat {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            _ => Err(OpenDogError::InvalidInput(format!(
                "format must be one of: json, csv; got '{}'",
                value
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortableProjectExport {
    pub schema_version: String,
    pub project_id: String,
    pub format: String,
    pub view: String,
    pub generated_at: String,
    pub summary: ProjectSummary,
    pub row_count: usize,
    pub rows: Vec<StatsEntry>,
}

pub fn export_rows(
    db: &Database,
    view: ExportView,
    min_access_count: i64,
) -> Result<Vec<StatsEntry>> {
    match view {
        ExportView::Stats => stats::get_stats(db),
        ExportView::Unused => stats::get_unused_files(db),
        ExportView::Core => stats::get_core_files(db, min_access_count),
    }
}

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

pub fn render_csv_export(rows: &[StatsEntry]) -> String {
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push(CSV_COLUMNS.join(","));
    for row in rows {
        lines.push(
            [
                escape_csv_field(&row.file_path),
                escape_csv_field(&row.file_type),
                row.size.to_string(),
                row.access_count.to_string(),
                row.estimated_duration_ms.to_string(),
                row.modification_count.to_string(),
                escape_csv_field(row.last_access_time.as_deref().unwrap_or("")),
                escape_csv_field(row.first_seen_time.as_deref().unwrap_or("")),
            ]
            .join(","),
        );
    }
    lines.join("\n")
}

pub fn write_export_file(path: &Path, content: &str) -> Result<u64> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(content.len() as u64)
}

fn escape_csv_field(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn now_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}

#[cfg(test)]
mod tests;
