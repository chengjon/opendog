use super::types::ObservationHints;
use crate::storage::database::Database;
use crate::storage::queries::{self, get_data_risk_cache};

pub(super) fn compute_observation_hints(db: &Database) -> ObservationHints {
    // Snapshot freshness — check if any snapshot data exists
    let snapshot_freshness = if let Ok(entries) = queries::get_snapshot_paths(db) {
        if !entries.is_empty() {
            "fresh"
        } else {
            "unknown"
        }
    } else {
        "unknown"
    };

    // Verification status
    let verification_status = match queries::get_latest_verification_runs(db) {
        Ok(runs) if runs.iter().all(|r| r.status == "passed") => "passed",
        Ok(runs) if runs.is_empty() => "not_recorded",
        _ => "failed",
    };

    // Unused files count
    let unused_files = queries::count_unused(db).unwrap_or(0) as usize;

    // Data risk candidates — read from cache populated by data-risk detection
    let data_risk_candidates: usize = get_data_risk_cache(db)
        .ok()
        .flatten()
        .map(|c| c.mock_candidate_count + c.hardcoded_candidate_count)
        .unwrap_or(0);

    ObservationHints {
        snapshot_freshness: snapshot_freshness.to_string(),
        verification_status: verification_status.to_string(),
        data_risk_candidates,
        unused_files,
    }
}
