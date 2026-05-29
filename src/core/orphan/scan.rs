use crate::config::ProjectConfig;
use crate::error::Result;
use std::path::Path;

use super::classification::classify_subject;
use super::evidence::{
    candidate_collector_evidence, collect_candidate_subjects, derive_required_scanners,
    docs_ownership_evidence, entrypoint_scanner_evidence, evidence_from_external_reports,
    frontend_literal_evidence, scanner_health_entry, scanner_health_from_external_reports,
    summarize_candidates,
};
use super::path_rules::{frontend_marker_exists, now_unix_secs};
use super::scanner_contract::validate_required_scanners;
use super::types::{
    ClassificationOptions, OrphanSubjectKind, ScanOrphansInput, ScanOrphansResult, ScannerHealth,
};

pub fn scan_project_orphans(
    root: &Path,
    config: &ProjectConfig,
    input: ScanOrphansInput,
) -> Result<ScanOrphansResult> {
    let mut warnings = Vec::new();
    let mut scanner_health = scanner_health_from_external_reports(&input.external_reports);
    let mut evidence = evidence_from_external_reports(&input.external_reports);
    let mut subjects = input.subjects.clone().unwrap_or_default();

    if input.include_internal_scanners {
        let collected_subjects = collect_candidate_subjects(root, config)?;
        scanner_health.push(scanner_health_entry(
            "candidate_collector",
            ScannerHealth::Passed,
        ));
        if subjects.is_empty() {
            subjects = collected_subjects;
        }
        evidence.extend(candidate_collector_evidence(&subjects));

        let entrypoint_signals = entrypoint_scanner_evidence(root, &subjects)?;
        scanner_health.push(scanner_health_entry(
            "entrypoint_scanner",
            ScannerHealth::Passed,
        ));
        evidence.extend(entrypoint_signals);

        let docs_signals = docs_ownership_evidence(root, &subjects)?;
        scanner_health.push(scanner_health_entry(
            "docs_ownership_gate",
            ScannerHealth::Passed,
        ));
        evidence.extend(docs_signals);

        if subjects
            .iter()
            .any(|subject| subject.subject_kind == OrphanSubjectKind::Url)
            || frontend_marker_exists(root)
        {
            let frontend_signals = frontend_literal_evidence(root, &subjects)?;
            scanner_health.push(scanner_health_entry(
                "frontend_literal_scanner",
                ScannerHealth::Passed,
            ));
            evidence.extend(frontend_signals);
        }
    }

    let mut required_scanners =
        validate_required_scanners(input.required_scanners.as_deref(), &input.external_reports)?;
    for derived in derive_required_scanners(root, &subjects) {
        if !required_scanners.contains(&derived) {
            required_scanners.push(derived);
        }
    }

    let options = ClassificationOptions {
        required_scanners,
        max_age_secs: input.max_age_secs,
        now_secs: Some(now_unix_secs()),
        ..Default::default()
    };

    let limit = input.limit.unwrap_or(50);
    let mut candidates = Vec::new();
    for subject in subjects.into_iter().take(limit.max(1)) {
        let mut candidate =
            classify_subject(&subject, scanner_health.clone(), evidence.clone(), &options)?;
        if !input.include_evidence {
            candidate.evidence.clear();
        }
        candidates.push(candidate);
    }

    if candidates.is_empty() {
        warnings.push("No orphan subjects were found or supplied.".to_string());
    }

    let summary = summarize_candidates(&candidates);
    Ok(ScanOrphansResult {
        status: "ok".to_string(),
        scan_run_id: None,
        scanner_health,
        summary,
        candidates,
        warnings,
        recommended_next_actions: vec![
            "Review blocked and review_required candidates before drafting deletion changes."
                .to_string(),
            "Run project test/lint/build verification before applying any deletion patch."
                .to_string(),
        ],
    })
}

#[cfg(test)]
mod tests;
