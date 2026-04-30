use crate::error::{OpenDogError, Result};

use super::{CleanupScope, ProjectDataCleanupRequest};

pub(super) fn validate_request(request: &ProjectDataCleanupRequest) -> Result<()> {
    if request.vacuum && request.dry_run {
        return Err(OpenDogError::InvalidInput(
            "vacuum cannot be combined with dry_run; remove dry_run to compact the database"
                .to_string(),
        ));
    }

    if let Some(days) = request.older_than_days {
        if days < 0 {
            return Err(OpenDogError::InvalidInput(
                "older_than_days must be >= 0".to_string(),
            ));
        }
    }

    match request.scope {
        CleanupScope::Activity | CleanupScope::Verification => {
            if request.older_than_days.is_none() {
                return Err(OpenDogError::InvalidInput(
                    "cleanup requires older_than_days for activity or verification scope"
                        .to_string(),
                ));
            }
        }
        CleanupScope::Snapshots => {
            if request.keep_snapshot_runs.is_none() {
                return Err(OpenDogError::InvalidInput(
                    "cleanup requires keep_snapshot_runs for snapshots scope".to_string(),
                ));
            }
        }
        CleanupScope::All => {
            if request.older_than_days.is_none() && request.keep_snapshot_runs.is_none() {
                return Err(OpenDogError::InvalidInput(
                    "cleanup requires older_than_days and/or keep_snapshot_runs for all scope"
                        .to_string(),
                ));
            }
        }
    }

    Ok(())
}
