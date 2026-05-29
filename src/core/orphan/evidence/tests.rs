use super::*;
use crate::core::orphan::types::OrphanSubjectKind;
use serde_json::json;

fn file_subject(name: &str) -> OrphanSubject {
    OrphanSubject {
        subject_kind: OrphanSubjectKind::File,
        subject: name.to_string(),
        path: Some(name.to_string()),
        display_name: None,
    }
}

#[test]
fn signal_matches_subject_by_subject_field() {
    let signal_subject = file_subject("foo.rs");
    let query_subject = OrphanSubject {
        subject_kind: OrphanSubjectKind::File,
        subject: "foo.rs".to_string(),
        path: None, // different path
        display_name: None,
    };
    let signal = EvidenceSignal {
        source: "t".into(),
        source_kind: "t".into(),
        signal_kind: "t".into(),
        polarity: EvidencePolarity::SupportsUnused,
        confidence: 0.5,
        observed_at: None,
        subject: signal_subject,
        detail: json!(null),
    };
    assert!(signal_matches_subject(&signal, &query_subject));
}

#[test]
fn signal_matches_subject_by_normalized_path() {
    let signal_subject = OrphanSubject {
        subject_kind: OrphanSubjectKind::File,
        subject: "unrelated".into(),
        path: Some("src/foo.rs".into()),
        display_name: None,
    };
    let query_subject = OrphanSubject {
        subject_kind: OrphanSubjectKind::File,
        subject: "different".into(),
        path: Some("src/foo.rs".into()),
        display_name: None,
    };
    let signal = EvidenceSignal {
        source: "t".into(),
        source_kind: "t".into(),
        signal_kind: "t".into(),
        polarity: EvidencePolarity::SupportsUnused,
        confidence: 0.5,
        observed_at: None,
        subject: signal_subject,
        detail: json!(null),
    };
    assert!(signal_matches_subject(&signal, &query_subject));
}

#[test]
fn signal_does_not_match_unrelated_subject() {
    let signal_subject = file_subject("a.rs");
    let query_subject = file_subject("b.rs");
    let signal = EvidenceSignal {
        source: "t".into(),
        source_kind: "t".into(),
        signal_kind: "t".into(),
        polarity: EvidencePolarity::SupportsUnused,
        confidence: 0.5,
        observed_at: None,
        subject: signal_subject,
        detail: json!(null),
    };
    assert!(!signal_matches_subject(&signal, &query_subject));
}

#[test]
fn candidate_collector_produces_unused_evidence() {
    let subjects = vec![file_subject("dead.rs"), file_subject("old.py")];
    let evidence = candidate_collector_evidence(&subjects);
    assert_eq!(evidence.len(), 2);
    assert_eq!(evidence[0].polarity, EvidencePolarity::SupportsUnused);
    assert_eq!(evidence[0].signal_kind, "candidate_collector");
}

#[test]
fn subject_match_tokens_includes_variants() {
    let subject = OrphanSubject {
        subject_kind: OrphanSubjectKind::Module,
        subject: "utils".into(),
        path: Some("src/utils.py".into()),
        display_name: None,
    };
    let tokens = subject_match_tokens(&subject);
    assert!(tokens.contains(&"utils".to_string()));
    assert!(tokens.contains(&"src/utils.py".to_string()));
    assert!(tokens.contains(&"src/utils".to_string())); // trimmed .py
}

#[test]
fn subject_text_matches_finds_literal() {
    let subject = file_subject("main.rs");
    assert!(subject_text_matches(
        "mod main;\n// main.rs is core\n",
        &subject
    ));
    assert!(!subject_text_matches(
        "completely unrelated content",
        &subject
    ));
}

#[test]
fn summarize_candidates_counts_by_classification() {
    use crate::core::orphan::types::OrphanClassification;
    let candidates = vec![
        ClassifiedOrphanCandidate {
            subject: file_subject("a.rs"),
            classification: OrphanClassification::RemoveCandidate,
            confidence: 0.9,
            reasons: vec![],
            vetoes: vec![],
            evidence: vec![],
        },
        ClassifiedOrphanCandidate {
            subject: file_subject("b.rs"),
            classification: OrphanClassification::RemoveCandidate,
            confidence: 0.8,
            reasons: vec![],
            vetoes: vec![],
            evidence: vec![],
        },
        ClassifiedOrphanCandidate {
            subject: file_subject("c.rs"),
            classification: OrphanClassification::Blocked,
            confidence: 0.9,
            reasons: vec![],
            vetoes: vec![],
            evidence: vec![],
        },
    ];
    let summary = summarize_candidates(&candidates);
    assert_eq!(summary.total_candidates, 3);
    assert_eq!(summary.remove_candidate_count, 2);
    assert_eq!(summary.review_required_count, 0);
    assert_eq!(summary.blocked_count, 1);
}

#[test]
fn external_report_evidence_and_health_extraction() {
    let subject = file_subject("target.rs");
    let reports = vec![ExternalScannerReport {
        scanner: "ext".into(),
        version: "1.0".into(),
        health: ScannerHealth::Passed,
        started_at: None,
        finished_at: None,
        evidence: vec![EvidenceSignal {
            source: "ext".into(),
            source_kind: "external".into(),
            signal_kind: "incoming_ref".into(),
            polarity: EvidencePolarity::SupportsUsed,
            confidence: 0.95,
            observed_at: None,
            subject,
            detail: json!("found"),
        }],
        warnings: vec![],
        errors: vec![],
    }];
    let health = scanner_health_from_external_reports(&reports);
    assert_eq!(health.len(), 1);
    assert_eq!(health[0].scanner, "ext");

    let evidence = evidence_from_external_reports(&reports);
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].signal_kind, "incoming_ref");
}
