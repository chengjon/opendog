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
mod tests {
    use super::*;
    use crate::storage::queries;
    use crate::storage::queries::SnapshotEntry;
    use rusqlite::params;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open_project(&db_path).unwrap();
        Box::leak(Box::new(dir));
        db
    }

    fn insert_snapshot(db: &Database, paths: &[&str]) {
        let entries: Vec<SnapshotEntry> = paths
            .iter()
            .map(|&path| SnapshotEntry {
                path: path.to_string(),
                size: 100,
                mtime: 0,
                file_type: "rs".to_string(),
                scan_timestamp: "1".to_string(),
            })
            .collect();
        queries::insert_snapshot_batch(db, &entries).unwrap();
    }

    fn insert_stat(db: &Database, path: &str, access_count: i64) {
        db.execute(
            "INSERT INTO file_stats (file_path, access_count, estimated_duration_ms, modification_count, first_seen_time, last_updated)
             VALUES (?1, ?2, 0, 0, '1', '1')",
            params![path, access_count],
        )
        .unwrap();
    }

    #[test]
    fn csv_export_uses_deterministic_header() {
        let csv = render_csv_export(&[StatsEntry {
            file_path: "src/main.rs".to_string(),
            size: 12,
            file_type: "rs".to_string(),
            access_count: 3,
            estimated_duration_ms: 99,
            modification_count: 1,
            last_access_time: Some("1".to_string()),
            first_seen_time: Some("1".to_string()),
        }]);
        assert_eq!(
            csv.lines().next().unwrap_or_default(),
            "file_path,file_type,size,access_count,estimated_duration_ms,modification_count,last_access_time,first_seen_time"
        );
    }

    #[test]
    fn export_rows_supports_stats_unused_and_core_views() {
        let db = test_db();
        insert_snapshot(&db, &["used.rs", "unused.rs", "core.rs"]);
        insert_stat(&db, "used.rs", 1);
        insert_stat(&db, "core.rs", 10);

        assert_eq!(export_rows(&db, ExportView::Stats, 5).unwrap().len(), 3);
        assert_eq!(export_rows(&db, ExportView::Unused, 5).unwrap().len(), 1);
        assert_eq!(export_rows(&db, ExportView::Core, 5).unwrap().len(), 1);
    }

    #[test]
    fn json_export_includes_portable_contract_fields() {
        let export = build_portable_export(
            "demo",
            ExportFormat::Json,
            ExportView::Stats,
            ProjectSummary {
                total_files: 1,
                accessed_files: 1,
                unused_files: 0,
            },
            vec![StatsEntry {
                file_path: "src/main.rs".to_string(),
                size: 12,
                file_type: "rs".to_string(),
                access_count: 3,
                estimated_duration_ms: 99,
                modification_count: 1,
                last_access_time: Some("1".to_string()),
                first_seen_time: Some("1".to_string()),
            }],
        );
        let value: serde_json::Value =
            serde_json::from_str(&render_json_export(&export).unwrap()).unwrap();
        assert_eq!(value["schema_version"], PORTABLE_PROJECT_EXPORT_V1);
        assert_eq!(value["project_id"], "demo");
        assert_eq!(value["format"], "json");
        assert_eq!(value["view"], "stats");
        assert_eq!(value["row_count"], 1);
    }
}
