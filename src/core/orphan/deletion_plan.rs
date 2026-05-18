use crate::config::ProjectConfig;
use crate::error::{OpenDogError, Result};
use std::path::Path;

use super::scan::scan_project_orphans;
use super::types::{
    DeletionPlanInput, DeletionPlanVerification, OrphanClassification, ScanOrphansInput,
};

pub fn verify_deletion_plan(
    root: &Path,
    config: &ProjectConfig,
    input: DeletionPlanInput,
) -> Result<DeletionPlanVerification> {
    if input.targets.is_empty() {
        return Err(OpenDogError::InvalidInput(
            "targets cannot be empty".to_string(),
        ));
    }

    let scan = scan_project_orphans(
        root,
        config,
        ScanOrphansInput {
            subjects: Some(input.targets),
            external_reports: input.external_reports,
            max_age_secs: input.max_age_secs,
            ..Default::default()
        },
    )?;

    let mut blocked_targets = Vec::new();
    let mut review_required_targets = Vec::new();
    let mut remove_candidates = Vec::new();
    for candidate in scan.candidates {
        match candidate.classification {
            OrphanClassification::Blocked => blocked_targets.push(candidate),
            OrphanClassification::ReviewRequired => review_required_targets.push(candidate),
            OrphanClassification::RemoveCandidate => remove_candidates.push(candidate),
        }
    }

    let mut evidence_gaps = Vec::new();
    if !blocked_targets.is_empty() {
        evidence_gaps.push("One or more targets are blocked by veto evidence.".to_string());
    }
    if !review_required_targets.is_empty() {
        evidence_gaps.push("One or more targets require additional review evidence.".to_string());
    }
    if input.required_project_verification_commands.is_empty() {
        evidence_gaps.push("No project verification commands were supplied.".to_string());
    }

    let safe_to_plan_deletion = blocked_targets.is_empty()
        && review_required_targets.is_empty()
        && !remove_candidates.is_empty()
        && !input.required_project_verification_commands.is_empty();

    Ok(DeletionPlanVerification {
        status: if safe_to_plan_deletion {
            "ready".to_string()
        } else {
            "blocked".to_string()
        },
        safe_to_plan_deletion,
        blocked_targets,
        review_required_targets,
        remove_candidates,
        required_project_verification_commands: input.required_project_verification_commands,
        evidence_gaps,
    })
}
