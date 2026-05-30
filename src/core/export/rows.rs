use super::types::ExportView;
use crate::core::stats;
use crate::error::Result;
use crate::storage::database::Database;
use crate::storage::queries::StatsEntry;

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
