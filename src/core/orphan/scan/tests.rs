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
    let result = scan_project_orphans(dir.path(), &default_config(), default_input()).unwrap();

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
    assert_eq!(result.summary.total_candidates, result.candidates.len());
}

// --- Test 2: Empty directory produces warnings ---

#[test]
fn empty_directory_produces_warning() {
    let dir = tempfile::tempdir().unwrap();
    let result = scan_project_orphans(dir.path(), &default_config(), default_input()).unwrap();

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

    let result = scan_project_orphans(dir.path(), &default_config(), input).unwrap();

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

    let result = scan_project_orphans(dir.path(), &default_config(), input).unwrap();

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

    let result = scan_project_orphans(dir.path(), &default_config(), default_input()).unwrap();

    let remove = result.summary.remove_candidate_count;
    let review = result.summary.review_required_count;
    let blocked = result.summary.blocked_count;

    assert_eq!(
        remove + review + blocked,
        result.summary.total_candidates,
        "summary counts do not add up to total"
    );
    assert_eq!(result.summary.total_candidates, result.candidates.len());
}
