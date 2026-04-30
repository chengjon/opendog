use crate::error::Result;
use crate::storage::database::Database;
use crate::storage::queries;

use super::{
    collect_storage_metrics, now_secs, validate_request, CleanupCountBreakdown,
    CleanupMaintenanceStatus, CleanupScope, ProjectDataCleanupRequest, ProjectDataCleanupResult,
};

pub fn cleanup_project_data(
    db: &Database,
    request: &ProjectDataCleanupRequest,
) -> Result<ProjectDataCleanupResult> {
    cleanup_project_data_at(db, request, now_secs())
}

pub fn cleanup_project_data_at(
    db: &Database,
    request: &ProjectDataCleanupRequest,
    now_ts: i64,
) -> Result<ProjectDataCleanupResult> {
    validate_request(request)?;

    let cutoff_ts = request.older_than_days.map(|days| {
        now_ts
            .saturating_sub(days.saturating_mul(24 * 60 * 60))
            .max(0)
    });
    let storage_before = collect_storage_metrics(db)?;
    let mut deleted = CleanupCountBreakdown::default();
    let mut maintenance = CleanupMaintenanceStatus::default();
    let mut notes = Vec::new();

    if matches!(request.scope, CleanupScope::Activity | CleanupScope::All) {
        if let Some(cutoff_ts) = cutoff_ts {
            deleted.file_sightings = if request.dry_run {
                queries::count_file_sightings_before(db, cutoff_ts)?
            } else {
                queries::delete_file_sightings_before(db, cutoff_ts)? as i64
            };
            deleted.file_events = if request.dry_run {
                queries::count_file_events_before(db, cutoff_ts)?
            } else {
                queries::delete_file_events_before(db, cutoff_ts)? as i64
            };
            notes.push(
                "activity cleanup only removes raw sightings and events; aggregate file_stats are preserved"
                    .to_string(),
            );
        }
    }

    if matches!(
        request.scope,
        CleanupScope::Verification | CleanupScope::All
    ) {
        if let Some(cutoff_ts) = cutoff_ts {
            deleted.verification_runs = if request.dry_run {
                queries::count_verification_runs_before(db, cutoff_ts)?
            } else {
                queries::delete_verification_runs_before(db, cutoff_ts)? as i64
            };
        }
    }

    if matches!(request.scope, CleanupScope::Snapshots | CleanupScope::All) {
        if let Some(keep_latest) = request.keep_snapshot_runs {
            let run_ids = queries::list_snapshot_run_ids_to_prune(db, keep_latest)?;
            deleted.snapshot_runs = run_ids.len() as i64;
            deleted.snapshot_history = queries::count_snapshot_history_for_runs(db, &run_ids)?;
            if !request.dry_run {
                queries::delete_snapshot_history_for_runs(db, &run_ids)?;
                queries::delete_snapshot_runs_by_ids(db, &run_ids)?;
            }
            notes.push(
                "snapshot cleanup only prunes historical snapshot_runs and snapshot_history; current snapshot baseline is preserved"
                    .to_string(),
            );
            if keep_latest < 2 {
                notes.push(
                    "keeping fewer than 2 snapshot runs can temporarily disable snapshot comparison until more snapshots are taken"
                        .to_string(),
                );
            }
        }
    }

    if deleted == CleanupCountBreakdown::default() {
        notes.push("no matching retained rows found for the requested cleanup scope".to_string());
    }

    if !request.dry_run && (deleted != CleanupCountBreakdown::default() || request.vacuum) {
        db.execute_batch("PRAGMA optimize;")?;
        maintenance.optimized = true;
        notes.push(
            "ran PRAGMA optimize after cleanup so SQLite can refresh lightweight planner hints"
                .to_string(),
        );
    }
    if !request.dry_run && request.vacuum {
        db.execute_batch("VACUUM;")?;
        maintenance.vacuumed = true;
        notes.push(
            "ran VACUUM to rebuild the project database file and reclaim unused pages".to_string(),
        );
    }

    let storage_after = if request.dry_run {
        None
    } else {
        Some(collect_storage_metrics(db)?)
    };

    Ok(ProjectDataCleanupResult {
        scope: request.scope.as_str().to_string(),
        dry_run: request.dry_run,
        older_than_days: request.older_than_days,
        keep_snapshot_runs: request.keep_snapshot_runs,
        vacuum: request.vacuum,
        deleted,
        storage_before,
        storage_after,
        maintenance,
        notes,
    })
}
