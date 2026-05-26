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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::orphan::types::{
        DeletionPlanInput, DeletionPlanVerification, OrphanSubjectKind,
    };
    use serde_json;

    fn make_config() -> ProjectConfig {
        ProjectConfig {
            ignore_patterns: vec![],
            process_whitelist: vec![],
        }
    }

    #[test]
    fn empty_targets_returns_invalid_input_error() {
        let dir = tempfile::tempdir().expect("tempdir creation failed");
        let config = make_config();
        let input = DeletionPlanInput {
            targets: vec![],
            external_reports: vec![],
            required_project_verification_commands: vec![],
            max_age_secs: None,
        };
        let result = verify_deletion_plan(dir.path(), &config, input);
        let err = result.expect_err("expected error for empty targets");
        match err {
            OpenDogError::InvalidInput(msg) => {
                assert!(
                    msg.contains("targets cannot be empty"),
                    "error message should mention empty targets, got: {msg}"
                );
            }
            other => panic!("expected InvalidInput, got: {other}"),
        }
    }

    #[test]
    fn deletion_plan_verification_serializes_all_fields() {
        let verification = DeletionPlanVerification {
            status: "ready".to_string(),
            safe_to_plan_deletion: true,
            blocked_targets: vec![],
            review_required_targets: vec![],
            remove_candidates: vec![],
            required_project_verification_commands: vec!["cargo test".to_string()],
            evidence_gaps: vec!["gap one".to_string()],
        };
        let json = serde_json::to_value(&verification).expect("serialization failed");
        assert_eq!(json["status"], "ready");
        assert_eq!(json["safe_to_plan_deletion"], true);
        assert!(json["blocked_targets"].as_array().unwrap().is_empty());
        assert!(json["review_required_targets"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(json["remove_candidates"].as_array().unwrap().is_empty());
        assert_eq!(
            json["required_project_verification_commands"][0],
            "cargo test"
        );
        assert_eq!(json["evidence_gaps"][0], "gap one");
    }

    #[test]
    fn deletion_plan_input_manual_default_has_expected_fields() {
        let input = DeletionPlanInput {
            targets: vec![],
            external_reports: vec![],
            required_project_verification_commands: vec![],
            max_age_secs: None,
        };
        assert!(input.targets.is_empty());
        assert!(input.external_reports.is_empty());
        assert!(input.required_project_verification_commands.is_empty());
        assert!(input.max_age_secs.is_none());
    }

    #[test]
    fn deletion_plan_input_with_all_fields_serializes() {
        let input = DeletionPlanInput {
            targets: vec![super::super::types::OrphanSubject {
                subject_kind: OrphanSubjectKind::File,
                subject: "dead.rs".to_string(),
                path: Some("src/dead.rs".to_string()),
                display_name: None,
            }],
            external_reports: vec![],
            required_project_verification_commands: vec!["cargo test".to_string()],
            max_age_secs: Some(3600),
        };
        let json = serde_json::to_value(&input).expect("serialization failed");
        assert_eq!(json["targets"].as_array().unwrap().len(), 1);
        assert_eq!(json["targets"][0]["subject_kind"], "file");
        assert_eq!(json["targets"][0]["subject"], "dead.rs");
        assert_eq!(
            json["required_project_verification_commands"][0],
            "cargo test"
        );
        assert_eq!(json["max_age_secs"], 3600);
    }
}
