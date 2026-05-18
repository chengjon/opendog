use super::*;
use crate::core::orphan::{
    ClassifiedOrphanCandidate, DeletionPlanVerification, OrphanClassification, OrphanScanSummary,
    OrphanSubject, OrphanSubjectKind, ScanOrphansResult, ScannerHealth, ScannerHealthEntry,
};

fn subject(path: &str) -> OrphanSubject {
    OrphanSubject {
        subject_kind: OrphanSubjectKind::File,
        subject: path.to_string(),
        path: Some(path.to_string()),
        display_name: None,
    }
}

#[test]
fn orphan_scan_payload_has_versioned_contract() {
    let value = orphan_scan_payload(
        "demo",
        &ScanOrphansResult {
            status: "ok".to_string(),
            scan_run_id: None,
            scanner_health: vec![ScannerHealthEntry {
                scanner: "entrypoint_scanner".to_string(),
                health: ScannerHealth::Passed,
                warnings: Vec::new(),
                errors: Vec::new(),
            }],
            summary: OrphanScanSummary {
                total_candidates: 1,
                remove_candidate_count: 0,
                review_required_count: 0,
                blocked_count: 1,
            },
            candidates: vec![ClassifiedOrphanCandidate {
                subject: subject("src/api/old.py"),
                classification: OrphanClassification::Blocked,
                confidence: 0.9,
                reasons: vec!["referenced by entrypoint".to_string()],
                vetoes: vec!["entrypoint veto".to_string()],
                evidence: Vec::new(),
            }],
            warnings: Vec::new(),
            recommended_next_actions: vec!["review candidate".to_string()],
        },
    );

    assert_eq!(value["schema_version"], MCP_ORPHAN_SCAN_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["status"], "ok");
    assert!(value["scan_run_id"].is_null());
    assert!(value["scanner_health"].is_array());
    assert!(value["candidates"].is_array());
    assert_eq!(value["summary"]["blocked_count"], 1);
}

#[test]
fn orphan_deletion_plan_payload_has_versioned_contract() {
    let value = orphan_deletion_plan_payload(
        "demo",
        &DeletionPlanVerification {
            status: "blocked".to_string(),
            safe_to_plan_deletion: false,
            blocked_targets: Vec::new(),
            review_required_targets: Vec::new(),
            remove_candidates: Vec::new(),
            required_project_verification_commands: vec!["cargo test".to_string()],
            evidence_gaps: vec!["additional evidence required".to_string()],
        },
    );

    assert_eq!(value["schema_version"], MCP_ORPHAN_DELETION_PLAN_V1);
    assert_eq!(value["project_id"], "demo");
    assert_eq!(value["status"], "blocked");
    assert_eq!(value["safe_to_plan_deletion"], false);
    assert!(value["required_project_verification_commands"].is_array());
    assert!(value["evidence_gaps"].is_array());
}
