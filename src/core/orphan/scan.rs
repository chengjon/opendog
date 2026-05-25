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
mod tests {
    use super::*;
    use crate::core::orphan::types::{
        ClassifiedOrphanCandidate, OrphanClassification, OrphanSubject, OrphanSubjectKind,
        ScanOrphansInput,
    };
    use std::fs;

    fn default_config() -> ProjectConfig {
        ProjectConfig::default()
    }

    /// Create a temp project directory with several source files.
    /// Returns (temp_dir, list of created relative paths).
    fn create_test_project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();

        // Source files that the candidate_collector should discover
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("utils.rs"), "pub fn helper() {}").unwrap();
        fs::write(dir.path().join("lib.py"), "def run(): pass").unwrap();

        // A subdirectory with more source files
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/mod.rs"), "pub mod sub;").unwrap();
        fs::write(dir.path().join("src/sub.rs"), "pub fn inner() {}").unwrap();

        // Non-source files (should NOT be collected as candidates)
        fs::write(dir.path().join("README.md"), "# docs").unwrap();
        fs::write(dir.path().join("config.toml"), "[package]").unwrap();

        dir
    }

    fn default_input() -> ScanOrphansInput {
        ScanOrphansInput {
            include_internal_scanners: true,
            ..ScanOrphansInput::default()
        }
    }

    // --- Test 1: Internal scanners discover source files and classify them ---

    #[test]
    fn internal_scanners_find_source_files_and_classify() {
        let dir = create_test_project();
        let result =
            scan_project_orphans(dir.path(), &default_config(), default_input()).unwrap();

        assert_eq!(result.status, "ok");
        // Should have found .rs and .py source files
        assert!(
            !result.candidates.is_empty(),
            "expected at least one candidate from source files"
        );

        // Verify scanner health entries for all internal scanners
        let scanner_names: Vec<&str> = result
            .scanner_health
            .iter()
            .map(|e| e.scanner.as_str())
            .collect();
        assert!(
            scanner_names.contains(&"candidate_collector"),
            "missing candidate_collector scanner"
        );
        assert!(
            scanner_names.contains(&"entrypoint_scanner"),
            "missing entrypoint_scanner"
        );
        assert!(
            scanner_names.contains(&"docs_ownership_gate"),
            "missing docs_ownership_gate"
        );

        // Summary should be internally consistent
        assert_eq!(
            result.summary.total_candidates,
            result.candidates.len()
        );
    }

    // --- Test 2: Empty directory produces warnings ---

    #[test]
    fn empty_directory_produces_warning() {
        let dir = tempfile::tempdir().unwrap();
        let result =
            scan_project_orphans(dir.path(), &default_config(), default_input()).unwrap();

        assert_eq!(result.status, "ok");
        assert!(result.candidates.is_empty());
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("No orphan subjects")),
            "expected 'No orphan subjects' warning, got: {:?}",
            result.warnings
        );
    }

    // --- Test 3: include_internal_scanners = false produces no internal candidates ---

    #[test]
    fn external_only_mode_uses_supplied_subjects() {
        let dir = create_test_project();

        // Supply our own subject since internal collection is disabled
        let subject = OrphanSubject {
            subject_kind: OrphanSubjectKind::File,
            subject: "main.rs".to_string(),
            path: Some("main.rs".to_string()),
            display_name: None,
        };
        let input = ScanOrphansInput {
            subjects: Some(vec![subject.clone()]),
            include_internal_scanners: false,
            ..ScanOrphansInput::default()
        };

        let result = scan_project_orphans(dir.path(), &default_config(), input).unwrap();

        assert_eq!(result.status, "ok");
        // No internal scanner health entries should appear
        let internal_scanners: Vec<&str> = result
            .scanner_health
            .iter()
            .map(|e| e.scanner.as_str())
            .filter(|name| {
                [
                    "candidate_collector",
                    "entrypoint_scanner",
                    "docs_ownership_gate",
                    "frontend_literal_scanner",
                ]
                .contains(name)
            })
            .collect();
        assert!(
            internal_scanners.is_empty(),
            "expected no internal scanners in external-only mode, got: {:?}",
            internal_scanners
        );

        // Should have exactly one candidate (our supplied subject)
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].subject.subject, "main.rs");
    }

    // --- Test 4: File referenced from another file is classified as Blocked ---

    #[test]
    fn referenced_file_classified_as_blocked() {
        let dir = tempfile::tempdir().unwrap();

        // Create an entrypoint-like file that references the subject
        fs::create_dir_all(dir.path().join(".github/workflows")).unwrap();
        fs::write(
            dir.path().join(".github/workflows/ci.yml"),
            "# uses worker.rs in build step",
        )
        .unwrap();

        // Create the subject file
        fs::write(dir.path().join("worker.rs"), "pub fn do_work() {}").unwrap();

        // Explicitly supply the subject so the entrypoint scanner sees it
        let subject = OrphanSubject {
            subject_kind: OrphanSubjectKind::File,
            subject: "worker.rs".to_string(),
            path: Some("worker.rs".to_string()),
            display_name: None,
        };
        let input = ScanOrphansInput {
            subjects: Some(vec![subject]),
            include_internal_scanners: true,
            ..ScanOrphansInput::default()
        };

        let result = scan_project_orphans(dir.path(), &default_config(), input).unwrap();

        // The entrypoint_scanner should have found a text reference to "worker.rs"
        // in .github/workflows/ci.yml, which produces a Veto signal, making it Blocked
        let blocked: Vec<&ClassifiedOrphanCandidate> = result
            .candidates
            .iter()
            .filter(|c| c.classification == OrphanClassification::Blocked)
            .collect();
        assert!(
            !blocked.is_empty(),
            "expected at least one Blocked candidate because worker.rs is referenced from ci.yml"
        );
    }

    // --- Test 5: limit parameter truncates results ---

    #[test]
    fn limit_truncates_candidates() {
        let dir = create_test_project();

        let input = ScanOrphansInput {
            limit: Some(2),
            ..default_input()
        };

        let result =
            scan_project_orphans(dir.path(), &default_config(), input).unwrap();

        assert!(
            result.candidates.len() <= 2,
            "expected at most 2 candidates with limit=2, got {}",
            result.candidates.len()
        );
    }

    // --- Test 6: include_evidence = false clears evidence from candidates ---

    #[test]
    fn include_evidence_false_clears_evidence() {
        let dir = create_test_project();

        let input = ScanOrphansInput {
            include_evidence: false,
            ..default_input()
        };

        let result =
            scan_project_orphans(dir.path(), &default_config(), input).unwrap();

        for candidate in &result.candidates {
            assert!(
                candidate.evidence.is_empty(),
                "expected empty evidence when include_evidence=false, but candidate '{}' has {} signals",
                candidate.subject.subject,
                candidate.evidence.len()
            );
        }
    }

    // --- Test 7: Summary counts are internally consistent ---

    #[test]
    fn summary_counts_match_candidates() {
        let dir = create_test_project();

        let result =
            scan_project_orphans(dir.path(), &default_config(), default_input()).unwrap();

        let remove = result.summary.remove_candidate_count;
        let review = result.summary.review_required_count;
        let blocked = result.summary.blocked_count;

        assert_eq!(
            remove + review + blocked,
            result.summary.total_candidates,
            "summary counts do not add up to total"
        );
        assert_eq!(
            result.summary.total_candidates,
            result.candidates.len()
        );
    }
}
