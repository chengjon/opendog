mod classification;
mod deletion_plan;
mod evidence;
mod path_rules;
mod scan;
mod scanner_contract;
mod types;

pub use self::classification::classify_subject;
pub use self::deletion_plan::verify_deletion_plan;
pub use self::scan::scan_project_orphans;
pub use self::scanner_contract::validate_required_scanners;
pub use self::types::{
    ClassificationOptions, ClassifiedOrphanCandidate, DeletionPlanInput, DeletionPlanVerification,
    EvidencePolarity, EvidenceSignal, ExternalScannerReport, OrphanClassification,
    OrphanScanSummary, OrphanSubject, OrphanSubjectKind, ScanOrphansInput, ScanOrphansResult,
    ScannerHealth, ScannerHealthEntry,
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn file_subject(path: &str) -> OrphanSubject {
        OrphanSubject {
            subject_kind: OrphanSubjectKind::File,
            subject: path.to_string(),
            path: Some(path.to_string()),
            display_name: None,
        }
    }

    fn scanner_health(scanner: &str, health: ScannerHealth) -> ScannerHealthEntry {
        ScannerHealthEntry {
            scanner: scanner.to_string(),
            health,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn signal(
        subject: &OrphanSubject,
        signal_kind: &str,
        polarity: EvidencePolarity,
        confidence: f64,
    ) -> EvidenceSignal {
        EvidenceSignal {
            source: signal_kind.to_string(),
            source_kind: "rust_internal".to_string(),
            signal_kind: signal_kind.to_string(),
            polarity,
            confidence,
            observed_at: None,
            subject: subject.clone(),
            detail: json!({}),
        }
    }

    #[test]
    fn veto_signal_blocks_candidate() {
        let subject = file_subject("src/api/old.py");
        let result = classify_subject(
            &subject,
            vec![scanner_health("entrypoint_scanner", ScannerHealth::Passed)],
            vec![signal(&subject, "entrypoint", EvidencePolarity::Veto, 0.95)],
            &ClassificationOptions::default(),
        )
        .unwrap();

        assert_eq!(result.classification, OrphanClassification::Blocked);
        assert!(result.vetoes.iter().any(|item| item.contains("entrypoint")));
    }

    #[test]
    fn missing_required_scanner_caps_at_review_required() {
        let subject = file_subject("src/api/old.py");
        let result = classify_subject(
            &subject,
            vec![scanner_health("candidate_collector", ScannerHealth::Passed)],
            vec![signal(
                &subject,
                "candidate_collector",
                EvidencePolarity::SupportsUnused,
                0.95,
            )],
            &ClassificationOptions {
                required_scanners: vec![
                    "candidate_collector".to_string(),
                    "entrypoint_scanner".to_string(),
                ],
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(result.classification, OrphanClassification::ReviewRequired);
        assert!(result
            .reasons
            .iter()
            .any(|item| item.contains("entrypoint_scanner")));
    }

    #[test]
    fn all_required_scanners_with_unused_evidence_can_remove_candidate() {
        let subject = file_subject("src/api/old.py");
        let result = classify_subject(
            &subject,
            vec![
                scanner_health("candidate_collector", ScannerHealth::Passed),
                scanner_health("entrypoint_scanner", ScannerHealth::Passed),
                scanner_health("docs_ownership_gate", ScannerHealth::Passed),
            ],
            vec![
                signal(
                    &subject,
                    "candidate_collector",
                    EvidencePolarity::SupportsUnused,
                    0.95,
                ),
                signal(
                    &subject,
                    "entrypoint",
                    EvidencePolarity::SupportsUnused,
                    0.90,
                ),
                signal(
                    &subject,
                    "docs_owner",
                    EvidencePolarity::SupportsUnused,
                    0.85,
                ),
            ],
            &ClassificationOptions::default(),
        )
        .unwrap();

        assert_eq!(result.classification, OrphanClassification::RemoveCandidate);
    }

    #[test]
    fn unknown_signal_kind_is_informational() {
        let subject = file_subject("src/api/old.py");
        let mut evidence = signal(
            &subject,
            "custom_scanner",
            EvidencePolarity::SupportsUsed,
            1.0,
        );
        evidence.signal_kind = "unknown_future_signal".to_string();

        let result = classify_subject(
            &subject,
            vec![
                scanner_health("candidate_collector", ScannerHealth::Passed),
                scanner_health("entrypoint_scanner", ScannerHealth::Passed),
                scanner_health("docs_ownership_gate", ScannerHealth::Passed),
            ],
            vec![
                signal(
                    &subject,
                    "candidate_collector",
                    EvidencePolarity::SupportsUnused,
                    0.95,
                ),
                signal(
                    &subject,
                    "entrypoint",
                    EvidencePolarity::SupportsUnused,
                    0.90,
                ),
                signal(
                    &subject,
                    "docs_owner",
                    EvidencePolarity::SupportsUnused,
                    0.85,
                ),
                evidence,
            ],
            &ClassificationOptions::default(),
        )
        .unwrap();

        assert_eq!(result.classification, OrphanClassification::RemoveCandidate);
    }

    #[test]
    fn empty_required_scanners_is_invalid() {
        let required: Vec<String> = Vec::new();
        let error = validate_required_scanners(Some(&required), &[]).unwrap_err();
        assert!(error
            .to_string()
            .contains("required_scanners cannot be empty"));
    }

    #[test]
    fn none_required_scanners_returns_defaults() {
        let scanners = validate_required_scanners(None, &[]).unwrap();
        assert!(!scanners.is_empty());
        assert!(scanners.contains(&"candidate_collector".to_string()));
    }

    #[test]
    fn unknown_scanner_name_is_rejected() {
        let required = vec!["candidate_collector".to_string(), "fantasy_scanner".to_string()];
        let error = validate_required_scanners(Some(&required), &[]).unwrap_err();
        assert!(error.to_string().contains("unknown required scanner 'fantasy_scanner'"));
    }

    #[test]
    fn deduplicates_repeated_scanner_names() {
        let required = vec![
            "candidate_collector".to_string(),
            "candidate_collector".to_string(),
        ];
        let scanners = validate_required_scanners(Some(&required), &[]).unwrap();
        assert_eq!(scanners.len(), 1);
    }

    #[test]
    fn verify_deletion_plan_rejects_empty_targets() {
        let dir = tempfile::tempdir().unwrap();
        let config = crate::config::ProjectConfig::default();
        let err = verify_deletion_plan(
            dir.path(),
            &config,
            DeletionPlanInput {
                targets: vec![],
                external_reports: vec![],
                required_project_verification_commands: vec!["cargo test".to_string()],
                max_age_secs: None,
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("targets cannot be empty"));
    }

    #[test]
    fn verify_deletion_plan_flags_nonexistent_target_as_blocked() {
        let dir = tempfile::tempdir().unwrap();
        let config = crate::config::ProjectConfig::default();
        let subject = OrphanSubject {
            subject_kind: OrphanSubjectKind::File,
            subject: "nonexistent_file.rs".to_string(),
            path: Some("nonexistent_file.rs".to_string()),
            display_name: None,
        };
        let result = verify_deletion_plan(
            dir.path(),
            &config,
            DeletionPlanInput {
                targets: vec![subject],
                external_reports: vec![ExternalScannerReport {
                    scanner: "external_veto".to_string(),
                    version: "1.0".to_string(),
                    health: ScannerHealth::Passed,
                    started_at: None,
                    finished_at: None,
                    evidence: vec![EvidenceSignal {
                        source: "external_veto".to_string(),
                        source_kind: "external".to_string(),
                        signal_kind: "incoming_ref".to_string(),
                        polarity: EvidencePolarity::Veto,
                        confidence: 0.99,
                        observed_at: None,
                        subject: OrphanSubject {
                            subject_kind: OrphanSubjectKind::File,
                            subject: "nonexistent_file.rs".to_string(),
                            path: Some("nonexistent_file.rs".to_string()),
                            display_name: None,
                        },
                        detail: json!({"reason": "still referenced"}),
                    }],
                    warnings: vec![],
                    errors: vec![],
                }],
                required_project_verification_commands: vec!["cargo test".to_string()],
                max_age_secs: None,
            },
        )
        .unwrap();

        assert!(!result.safe_to_plan_deletion);
        assert_eq!(result.status, "blocked");
        assert!(!result.blocked_targets.is_empty());
    }
}
