use crate::config::{should_ignore_path, ProjectConfig};
use crate::core::file_classification::{classify_file_path, FilePathClassification};
use crate::error::Result;
use serde_json::json;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use super::path_rules::{
    fastapi_marker_exists, frontend_marker_exists, is_docs_or_ownership_file, is_docs_path,
    is_entrypoint_file, is_frontend_source_file, normalize_path, now_unix_secs,
    python_marker_exists, relative_path,
};
use super::types::{
    ClassifiedOrphanCandidate, EvidencePolarity, EvidenceSignal, ExternalScannerReport,
    OrphanClassification, OrphanScanSummary, OrphanSubject, OrphanSubjectKind, ScannerHealth,
    ScannerHealthEntry, DEFAULT_REQUIRED_SCANNERS,
};

pub(super) fn derive_required_scanners(root: &Path, subjects: &[OrphanSubject]) -> Vec<String> {
    let mut required = BTreeSet::new();
    for scanner in DEFAULT_REQUIRED_SCANNERS {
        required.insert((*scanner).to_string());
    }
    if frontend_marker_exists(root)
        || subjects
            .iter()
            .any(|subject| subject.subject_kind == OrphanSubjectKind::Url)
    {
        required.insert("frontend_literal_scanner".to_string());
    }
    if python_marker_exists(root)
        && subjects.iter().any(|subject| {
            subject.subject_kind == OrphanSubjectKind::Module
                || subject
                    .path
                    .as_deref()
                    .is_some_and(|path| path.ends_with(".py"))
        })
    {
        required.insert("python_import_graph".to_string());
    }
    if subjects.iter().any(|subject| {
        matches!(
            subject.subject_kind,
            OrphanSubjectKind::Route | OrphanSubjectKind::Url
        )
    }) && fastapi_marker_exists(root)
    {
        required.insert("fastapi_route_auditor".to_string());
        required.insert("openapi_contract".to_string());
    }
    required.into_iter().collect()
}

pub(super) fn scanner_health_from_external_reports(
    reports: &[ExternalScannerReport],
) -> Vec<ScannerHealthEntry> {
    reports
        .iter()
        .map(|report| ScannerHealthEntry {
            scanner: report.scanner.clone(),
            health: report.health.clone(),
            warnings: report.warnings.clone(),
            errors: report.errors.clone(),
        })
        .collect()
}

pub(super) fn evidence_from_external_reports(
    reports: &[ExternalScannerReport],
) -> Vec<EvidenceSignal> {
    reports
        .iter()
        .flat_map(|report| report.evidence.clone())
        .collect()
}

pub(super) fn scanner_health_entry(scanner: &str, health: ScannerHealth) -> ScannerHealthEntry {
    ScannerHealthEntry {
        scanner: scanner.to_string(),
        health,
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

pub(super) fn signal_matches_subject(signal: &EvidenceSignal, subject: &OrphanSubject) -> bool {
    signal.subject.subject == subject.subject
        || signal
            .subject
            .path
            .as_deref()
            .zip(subject.path.as_deref())
            .is_some_and(|(left, right)| normalize_path(left) == normalize_path(right))
}

pub(super) fn collect_candidate_subjects(
    root: &Path,
    config: &ProjectConfig,
) -> Result<Vec<OrphanSubject>> {
    let mut subjects = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(rel_path) = relative_path(root, entry.path()) else {
            continue;
        };
        if should_ignore_path(&rel_path, config) || is_docs_path(&rel_path) {
            continue;
        }
        match classify_file_path(&rel_path) {
            FilePathClassification::Source => subjects.push(OrphanSubject {
                subject_kind: OrphanSubjectKind::File,
                subject: rel_path.clone(),
                path: Some(rel_path),
                display_name: None,
            }),
            FilePathClassification::Infrastructure
            | FilePathClassification::Backup
            | FilePathClassification::Project => {}
        }
    }
    Ok(subjects)
}

pub(super) fn candidate_collector_evidence(subjects: &[OrphanSubject]) -> Vec<EvidenceSignal> {
    subjects
        .iter()
        .map(|subject| EvidenceSignal {
            source: "candidate_collector".to_string(),
            source_kind: "rust_internal".to_string(),
            signal_kind: "candidate_collector".to_string(),
            polarity: EvidencePolarity::SupportsUnused,
            confidence: 0.70,
            observed_at: Some(now_unix_secs()),
            subject: subject.clone(),
            detail: json!({"reason": "subject is a source-like file candidate"}),
        })
        .collect()
}

pub(super) fn entrypoint_scanner_evidence(
    root: &Path,
    subjects: &[OrphanSubject],
) -> Result<Vec<EvidenceSignal>> {
    text_scanner_evidence(
        root,
        subjects,
        "entrypoint_scanner",
        "entrypoint",
        is_entrypoint_file,
    )
}

pub(super) fn docs_ownership_evidence(
    root: &Path,
    subjects: &[OrphanSubject],
) -> Result<Vec<EvidenceSignal>> {
    text_scanner_evidence(
        root,
        subjects,
        "docs_ownership_gate",
        "docs_owner",
        is_docs_or_ownership_file,
    )
}

pub(super) fn frontend_literal_evidence(
    root: &Path,
    subjects: &[OrphanSubject],
) -> Result<Vec<EvidenceSignal>> {
    text_scanner_evidence(
        root,
        subjects,
        "frontend_literal_scanner",
        "frontend_consumer",
        is_frontend_source_file,
    )
}

pub(super) fn text_scanner_evidence(
    root: &Path,
    subjects: &[OrphanSubject],
    scanner: &str,
    signal_kind: &str,
    include_file: fn(&str) -> bool,
) -> Result<Vec<EvidenceSignal>> {
    let mut searchable_text = String::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(rel_path) = relative_path(root, entry.path()) else {
            continue;
        };
        if !include_file(&rel_path) {
            continue;
        }
        if let Ok(metadata) = entry.metadata() {
            if metadata.len() > 256 * 1024 {
                continue;
            }
        }
        if let Ok(content) = fs::read_to_string(entry.path()) {
            searchable_text.push_str(&rel_path);
            searchable_text.push('\n');
            searchable_text.push_str(&content);
            searchable_text.push('\n');
        }
    }

    let mut evidence = Vec::new();
    for subject in subjects {
        if subject_text_matches(&searchable_text, subject) {
            evidence.push(EvidenceSignal {
                source: scanner.to_string(),
                source_kind: "rust_internal".to_string(),
                signal_kind: signal_kind.to_string(),
                polarity: EvidencePolarity::Veto,
                confidence: 0.90,
                observed_at: Some(now_unix_secs()),
                subject: subject.clone(),
                detail: json!({"match": "literal subject or path reference"}),
            });
        } else {
            evidence.push(EvidenceSignal {
                source: scanner.to_string(),
                source_kind: "rust_internal".to_string(),
                signal_kind: signal_kind.to_string(),
                polarity: EvidencePolarity::SupportsUnused,
                confidence: 0.65,
                observed_at: Some(now_unix_secs()),
                subject: subject.clone(),
                detail: json!({"match": "no literal reference found"}),
            });
        }
    }
    Ok(evidence)
}

pub(super) fn subject_text_matches(text: &str, subject: &OrphanSubject) -> bool {
    let normalized_text = text.replace('\\', "/");
    let candidates = subject_match_tokens(subject);
    candidates
        .iter()
        .filter(|candidate| !candidate.is_empty())
        .any(|candidate| normalized_text.contains(candidate))
}

pub(super) fn subject_match_tokens(subject: &OrphanSubject) -> Vec<String> {
    let mut tokens = Vec::new();
    tokens.push(subject.subject.replace('\\', "/"));
    if let Some(path) = &subject.path {
        let normalized = normalize_path(path);
        tokens.push(normalized.clone());
        tokens.push(
            normalized
                .replace('/', ".")
                .trim_end_matches(".py")
                .to_string(),
        );
        tokens.push(normalized.trim_end_matches(".py").to_string());
    }
    tokens.sort();
    tokens.dedup();
    tokens
}

pub(super) fn summarize_candidates(candidates: &[ClassifiedOrphanCandidate]) -> OrphanScanSummary {
    let remove_candidate_count = candidates
        .iter()
        .filter(|candidate| candidate.classification == OrphanClassification::RemoveCandidate)
        .count();
    let review_required_count = candidates
        .iter()
        .filter(|candidate| candidate.classification == OrphanClassification::ReviewRequired)
        .count();
    let blocked_count = candidates
        .iter()
        .filter(|candidate| candidate.classification == OrphanClassification::Blocked)
        .count();

    OrphanScanSummary {
        total_candidates: candidates.len(),
        remove_candidate_count,
        review_required_count,
        blocked_count,
    }
}

#[cfg(test)]
mod tests;
