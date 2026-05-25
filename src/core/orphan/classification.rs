use crate::error::Result;
use std::collections::BTreeMap;

use super::evidence::signal_matches_subject;
use super::types::{
    ClassificationOptions, ClassifiedOrphanCandidate, EvidencePolarity, EvidenceSignal,
    OrphanClassification, OrphanSubject, ScannerHealth, ScannerHealthEntry, KNOWN_SIGNAL_KINDS,
};

pub fn classify_subject(
    subject: &OrphanSubject,
    scanner_health: Vec<ScannerHealthEntry>,
    evidence: Vec<EvidenceSignal>,
    options: &ClassificationOptions,
) -> Result<ClassifiedOrphanCandidate> {
    let mut reasons = Vec::new();
    let mut vetoes = Vec::new();
    let mut confidence = 0.0_f64;
    let subject_evidence: Vec<EvidenceSignal> = evidence
        .into_iter()
        .filter(|signal| signal_matches_subject(signal, subject))
        .collect();

    for signal in &subject_evidence {
        if !KNOWN_SIGNAL_KINDS.contains(&signal.signal_kind.as_str()) {
            continue;
        }

        confidence = confidence.max(signal.confidence);
        match signal.polarity {
            EvidencePolarity::Veto => {
                vetoes.push(format!(
                    "{} veto from {}",
                    signal.signal_kind, signal.source
                ));
            }
            EvidencePolarity::SupportsUsed
                if signal.confidence >= options.used_signal_threshold =>
            {
                vetoes.push(format!(
                    "{} used signal from {}",
                    signal.signal_kind, signal.source
                ));
            }
            EvidencePolarity::SupportsUsed
            | EvidencePolarity::SupportsUnused
            | EvidencePolarity::Informational => {}
        }
    }

    if !vetoes.is_empty() {
        return Ok(ClassifiedOrphanCandidate {
            subject: subject.clone(),
            classification: OrphanClassification::Blocked,
            confidence,
            reasons: vec!["A veto or strong used signal references this subject.".to_string()],
            vetoes,
            evidence: subject_evidence,
        });
    }

    let health_by_scanner: BTreeMap<&str, &ScannerHealthEntry> = scanner_health
        .iter()
        .map(|entry| (entry.scanner.as_str(), entry))
        .collect();
    for scanner in &options.required_scanners {
        match health_by_scanner.get(scanner.as_str()) {
            Some(entry)
                if matches!(
                    entry.health,
                    ScannerHealth::Passed | ScannerHealth::PassedWithWarnings
                ) => {}
            Some(entry) => reasons.push(format!(
                "Required scanner '{}' is {:?}.",
                scanner, entry.health
            )),
            None => reasons.push(format!("Required scanner '{}' did not run.", scanner)),
        }
    }

    if let (Some(max_age_secs), Some(now_secs)) = (options.max_age_secs, options.now_secs) {
        for signal in &subject_evidence {
            if signal
                .observed_at
                .is_some_and(|observed| observed.saturating_add(max_age_secs) < now_secs)
            {
                reasons.push(format!(
                    "Evidence from '{}' is older than {} seconds.",
                    signal.source, max_age_secs
                ));
            }
        }
    }

    if !reasons.is_empty() {
        return Ok(ClassifiedOrphanCandidate {
            subject: subject.clone(),
            classification: OrphanClassification::ReviewRequired,
            confidence,
            reasons,
            vetoes,
            evidence: subject_evidence,
        });
    }

    let has_unused_support = subject_evidence.iter().any(|signal| {
        KNOWN_SIGNAL_KINDS.contains(&signal.signal_kind.as_str())
            && signal.polarity == EvidencePolarity::SupportsUnused
    });

    if has_unused_support {
        Ok(ClassifiedOrphanCandidate {
            subject: subject.clone(),
            classification: OrphanClassification::RemoveCandidate,
            confidence,
            reasons: vec!["Required scanners passed and no used signal was found.".to_string()],
            vetoes,
            evidence: subject_evidence,
        })
    } else {
        Ok(ClassifiedOrphanCandidate {
            subject: subject.clone(),
            classification: OrphanClassification::ReviewRequired,
            confidence,
            reasons: vec!["No positive unused evidence was produced.".to_string()],
            vetoes,
            evidence: subject_evidence,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::orphan::types::OrphanSubjectKind;
    use serde_json::json;

    fn file_subject(path: &str) -> OrphanSubject {
        OrphanSubject {
            subject_kind: OrphanSubjectKind::File,
            subject: path.to_string(),
            path: Some(path.to_string()),
            display_name: None,
        }
    }

    fn signal(polarity: EvidencePolarity, kind: &str, subject: &OrphanSubject) -> EvidenceSignal {
        EvidenceSignal {
            source: "test".to_string(),
            source_kind: "test".to_string(),
            signal_kind: kind.to_string(),
            polarity,
            confidence: 0.9,
            observed_at: None,
            subject: subject.clone(),
            detail: json!("test"),
        }
    }

    fn passed_health(scanner: &str) -> ScannerHealthEntry {
        ScannerHealthEntry {
            scanner: scanner.to_string(),
            health: ScannerHealth::Passed,
            warnings: vec![],
            errors: vec![],
        }
    }

    fn default_options() -> ClassificationOptions {
        ClassificationOptions {
            required_scanners: vec!["candidate_collector".to_string()],
            used_signal_threshold: 0.80,
            max_age_secs: None,
            now_secs: None,
        }
    }

    #[test]
    fn veto_signal_classifies_as_blocked() {
        let subject = file_subject("dead.rs");
        let evidence = vec![signal(EvidencePolarity::Veto, "incoming_ref", &subject)];
        let result = classify_subject(&subject, vec![], evidence, &default_options()).unwrap();
        assert_eq!(result.classification, OrphanClassification::Blocked);
        assert!(!result.vetoes.is_empty());
    }

    #[test]
    fn strong_used_signal_classifies_as_blocked() {
        let subject = file_subject("alive.rs");
        let evidence = vec![signal(EvidencePolarity::SupportsUsed, "incoming_ref", &subject)];
        let result = classify_subject(&subject, vec![], evidence, &default_options()).unwrap();
        assert_eq!(result.classification, OrphanClassification::Blocked);
    }

    #[test]
    fn weak_used_signal_does_not_block() {
        let subject = file_subject("maybe.rs");
        let mut s = signal(EvidencePolarity::SupportsUsed, "incoming_ref", &subject);
        s.confidence = 0.50;
        let evidence = vec![s];
        let health = vec![passed_health("candidate_collector")];
        let result = classify_subject(&subject, health, evidence, &default_options()).unwrap();
        assert_eq!(result.classification, OrphanClassification::ReviewRequired);
    }

    #[test]
    fn missing_required_scanner_yields_review_required() {
        let subject = file_subject("uncertain.rs");
        let result = classify_subject(&subject, vec![], vec![], &default_options()).unwrap();
        assert_eq!(result.classification, OrphanClassification::ReviewRequired);
        assert!(result.reasons.iter().any(|r| r.contains("did not run")));
    }

    #[test]
    fn failed_scanner_yields_review_required() {
        let subject = file_subject("partial.rs");
        let health = vec![ScannerHealthEntry {
            scanner: "candidate_collector".to_string(),
            health: ScannerHealth::Failed,
            warnings: vec![],
            errors: vec!["crashed".to_string()],
        }];
        let result = classify_subject(&subject, health, vec![], &default_options()).unwrap();
        assert_eq!(result.classification, OrphanClassification::ReviewRequired);
        assert!(result.reasons.iter().any(|r| r.contains("Failed")));
    }

    #[test]
    fn unused_evidence_with_passed_scanners_yields_remove_candidate() {
        let subject = file_subject("dead_code.rs");
        let evidence = vec![signal(EvidencePolarity::SupportsUnused, "candidate_collector", &subject)];
        let health = vec![passed_health("candidate_collector")];
        let result = classify_subject(&subject, health, evidence, &default_options()).unwrap();
        assert_eq!(result.classification, OrphanClassification::RemoveCandidate);
    }

    #[test]
    fn no_evidence_no_unused_yields_review_required() {
        let subject = file_subject("mystery.rs");
        let health = vec![passed_health("candidate_collector")];
        let result = classify_subject(&subject, health, vec![], &default_options()).unwrap();
        assert_eq!(result.classification, OrphanClassification::ReviewRequired);
        assert!(result.reasons.iter().any(|r| r.contains("No positive unused evidence")));
    }

    #[test]
    fn stale_evidence_triggers_review() {
        let subject = file_subject("old.rs");
        let mut s = signal(EvidencePolarity::SupportsUnused, "candidate_collector", &subject);
        s.observed_at = Some(100);
        let evidence = vec![s];
        let health = vec![passed_health("candidate_collector")];
        let mut opts = default_options();
        opts.max_age_secs = Some(600);
        opts.now_secs = Some(1000);
        let result = classify_subject(&subject, health, evidence, &opts).unwrap();
        assert_eq!(result.classification, OrphanClassification::ReviewRequired);
        assert!(result.reasons.iter().any(|r| r.contains("older than")));
    }

    #[test]
    fn unknown_signal_kind_ignored() {
        let subject = file_subject("ignore.rs");
        let mut s = signal(EvidencePolarity::SupportsUnused, "future_scanner", &subject);
        s.signal_kind = "future_scanner".to_string();
        let evidence = vec![s];
        let health = vec![passed_health("candidate_collector")];
        let result = classify_subject(&subject, health, evidence, &default_options()).unwrap();
        assert_eq!(result.classification, OrphanClassification::ReviewRequired);
    }
}
