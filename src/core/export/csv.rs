use crate::storage::queries::StatsEntry;

pub(super) const CSV_COLUMNS: &[&str] = &[
    "file_path",
    "file_type",
    "size",
    "access_count",
    "estimated_duration_ms",
    "modification_count",
    "last_access_time",
    "first_seen_time",
];

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

pub(super) fn escape_csv_field(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
