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

#[cfg(test)]
mod tests {
    use super::*;

    fn base_request() -> ProjectDataCleanupRequest {
        ProjectDataCleanupRequest {
            scope: CleanupScope::All,
            older_than_days: Some(30),
            keep_snapshot_runs: Some(1),
            vacuum: false,
            dry_run: false,
        }
    }

    #[test]
    fn valid_all_scope_passes() {
        assert!(validate_request(&base_request()).is_ok());
    }

    #[test]
    fn vacuum_plus_dry_run_rejected() {
        let mut req = base_request();
        req.vacuum = true;
        req.dry_run = true;
        let err = validate_request(&req).unwrap_err();
        assert!(err.to_string().contains("vacuum cannot be combined with dry_run"));
    }

    #[test]
    fn negative_older_than_days_rejected() {
        let mut req = base_request();
        req.older_than_days = Some(-1);
        let err = validate_request(&req).unwrap_err();
        assert!(err.to_string().contains("older_than_days must be >= 0"));
    }

    #[test]
    fn activity_scope_requires_older_than_days() {
        let mut req = base_request();
        req.scope = CleanupScope::Activity;
        req.older_than_days = None;
        let err = validate_request(&req).unwrap_err();
        assert!(err.to_string().contains("older_than_days"));
    }

    #[test]
    fn verification_scope_requires_older_than_days() {
        let mut req = base_request();
        req.scope = CleanupScope::Verification;
        req.older_than_days = None;
        let err = validate_request(&req).unwrap_err();
        assert!(err.to_string().contains("older_than_days"));
    }

    #[test]
    fn snapshots_scope_requires_keep_snapshot_runs() {
        let mut req = base_request();
        req.scope = CleanupScope::Snapshots;
        req.keep_snapshot_runs = None;
        let err = validate_request(&req).unwrap_err();
        assert!(err.to_string().contains("keep_snapshot_runs"));
    }

    #[test]
    fn all_scope_accepts_either_parameter() {
        let mut req = base_request();
        req.older_than_days = Some(7);
        req.keep_snapshot_runs = None;
        assert!(validate_request(&req).is_ok());

        req.older_than_days = None;
        req.keep_snapshot_runs = Some(3);
        assert!(validate_request(&req).is_ok());
    }
}
