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
mod tests;
