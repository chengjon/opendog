use crate::config::{should_ignore_path, ProjectConfig};
use crate::core::file_classification::{classify_file_path, FilePathClassification};
use crate::error::{OpenDogError, Result};
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

const INTERNAL_SCANNERS: &[&str] = &[
    "candidate_collector",
    "entrypoint_scanner",
    "docs_ownership_gate",
    "frontend_literal_scanner",
];

const DEFAULT_REQUIRED_SCANNERS: &[&str] = &[
    "candidate_collector",
    "entrypoint_scanner",
    "docs_ownership_gate",
];

const KNOWN_SIGNAL_KINDS: &[&str] = &[
    "incoming_ref",
    "outgoing_ref",
    "runtime_route",
    "openapi_path",
    "test_coverage",
    "entrypoint",
    "frontend_consumer",
    "docs_owner",
    "telemetry",
    "dynamic_import_risk",
    "scanner_warning",
    "candidate_collector",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrphanSubjectKind {
    File,
    Module,
    Route,
    Url,
    Command,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct OrphanSubject {
    pub subject_kind: OrphanSubjectKind,
    pub subject: String,
    pub path: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidencePolarity {
    SupportsUsed,
    SupportsUnused,
    Veto,
    Informational,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScannerHealth {
    Passed,
    PassedWithWarnings,
    Skipped,
    Failed,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrphanClassification {
    RemoveCandidate,
    ReviewRequired,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EvidenceSignal {
    pub source: String,
    pub source_kind: String,
    pub signal_kind: String,
    pub polarity: EvidencePolarity,
    pub confidence: f64,
    pub observed_at: Option<u64>,
    pub subject: OrphanSubject,
    pub detail: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ScannerHealthEntry {
    pub scanner: String,
    pub health: ScannerHealth,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExternalScannerReport {
    pub scanner: String,
    pub version: String,
    pub health: ScannerHealth,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    pub evidence: Vec<EvidenceSignal>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ClassificationOptions {
    pub required_scanners: Vec<String>,
    pub used_signal_threshold: f64,
    pub max_age_secs: Option<u64>,
    pub now_secs: Option<u64>,
}

impl Default for ClassificationOptions {
    fn default() -> Self {
        Self {
            required_scanners: DEFAULT_REQUIRED_SCANNERS
                .iter()
                .map(|scanner| (*scanner).to_string())
                .collect(),
            used_signal_threshold: 0.80,
            max_age_secs: None,
            now_secs: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ClassifiedOrphanCandidate {
    pub subject: OrphanSubject,
    pub classification: OrphanClassification,
    pub confidence: f64,
    pub reasons: Vec<String>,
    pub vetoes: Vec<String>,
    pub evidence: Vec<EvidenceSignal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ScanOrphansInput {
    pub subjects: Option<Vec<OrphanSubject>>,
    pub external_reports: Vec<ExternalScannerReport>,
    pub include_internal_scanners: bool,
    pub required_scanners: Option<Vec<String>>,
    pub max_age_secs: Option<u64>,
    pub limit: Option<usize>,
    pub include_evidence: bool,
}

impl Default for ScanOrphansInput {
    fn default() -> Self {
        Self {
            subjects: None,
            external_reports: Vec::new(),
            include_internal_scanners: true,
            required_scanners: None,
            max_age_secs: None,
            limit: None,
            include_evidence: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct OrphanScanSummary {
    pub total_candidates: usize,
    pub remove_candidate_count: usize,
    pub review_required_count: usize,
    pub blocked_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ScanOrphansResult {
    pub status: String,
    pub scan_run_id: Option<i64>,
    pub scanner_health: Vec<ScannerHealthEntry>,
    pub summary: OrphanScanSummary,
    pub candidates: Vec<ClassifiedOrphanCandidate>,
    pub warnings: Vec<String>,
    pub recommended_next_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeletionPlanInput {
    pub targets: Vec<OrphanSubject>,
    pub external_reports: Vec<ExternalScannerReport>,
    pub required_project_verification_commands: Vec<String>,
    pub max_age_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeletionPlanVerification {
    pub status: String,
    pub safe_to_plan_deletion: bool,
    pub blocked_targets: Vec<ClassifiedOrphanCandidate>,
    pub review_required_targets: Vec<ClassifiedOrphanCandidate>,
    pub remove_candidates: Vec<ClassifiedOrphanCandidate>,
    pub required_project_verification_commands: Vec<String>,
    pub evidence_gaps: Vec<String>,
}

pub fn validate_required_scanners(
    required_scanners: Option<&[String]>,
    external_reports: &[ExternalScannerReport],
) -> Result<Vec<String>> {
    let Some(required_scanners) = required_scanners else {
        return Ok(DEFAULT_REQUIRED_SCANNERS
            .iter()
            .map(|scanner| (*scanner).to_string())
            .collect());
    };

    if required_scanners.is_empty() {
        return Err(OpenDogError::InvalidInput(
            "required_scanners cannot be empty".to_string(),
        ));
    }

    let external_names: BTreeSet<&str> = external_reports
        .iter()
        .map(|report| report.scanner.as_str())
        .collect();
    let mut validated = BTreeSet::new();
    for scanner in required_scanners {
        let known_internal = INTERNAL_SCANNERS.contains(&scanner.as_str());
        if !known_internal && !external_names.contains(scanner.as_str()) {
            return Err(OpenDogError::InvalidInput(format!(
                "unknown required scanner '{}'",
                scanner
            )));
        }
        validated.insert(scanner.clone());
    }

    Ok(validated.into_iter().collect())
}

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

fn derive_required_scanners(root: &Path, subjects: &[OrphanSubject]) -> Vec<String> {
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

fn scanner_health_from_external_reports(
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

fn evidence_from_external_reports(reports: &[ExternalScannerReport]) -> Vec<EvidenceSignal> {
    reports
        .iter()
        .flat_map(|report| report.evidence.clone())
        .collect()
}

fn scanner_health_entry(scanner: &str, health: ScannerHealth) -> ScannerHealthEntry {
    ScannerHealthEntry {
        scanner: scanner.to_string(),
        health,
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn signal_matches_subject(signal: &EvidenceSignal, subject: &OrphanSubject) -> bool {
    signal.subject.subject == subject.subject
        || signal
            .subject
            .path
            .as_deref()
            .zip(subject.path.as_deref())
            .is_some_and(|(left, right)| normalize_path(left) == normalize_path(right))
}

fn collect_candidate_subjects(root: &Path, config: &ProjectConfig) -> Result<Vec<OrphanSubject>> {
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

fn candidate_collector_evidence(subjects: &[OrphanSubject]) -> Vec<EvidenceSignal> {
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

fn entrypoint_scanner_evidence(
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

fn docs_ownership_evidence(root: &Path, subjects: &[OrphanSubject]) -> Result<Vec<EvidenceSignal>> {
    text_scanner_evidence(
        root,
        subjects,
        "docs_ownership_gate",
        "docs_owner",
        is_docs_or_ownership_file,
    )
}

fn frontend_literal_evidence(
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

fn text_scanner_evidence(
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

fn subject_text_matches(text: &str, subject: &OrphanSubject) -> bool {
    let normalized_text = text.replace('\\', "/");
    let candidates = subject_match_tokens(subject);
    candidates
        .iter()
        .filter(|candidate| !candidate.is_empty())
        .any(|candidate| normalized_text.contains(candidate))
}

fn subject_match_tokens(subject: &OrphanSubject) -> Vec<String> {
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

fn summarize_candidates(candidates: &[ClassifiedOrphanCandidate]) -> OrphanScanSummary {
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

fn relative_path(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root)
        .ok()
        .and_then(|path| path.to_str())
        .map(normalize_path)
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn is_docs_path(rel_path: &str) -> bool {
    let lower = rel_path.to_ascii_lowercase();
    lower == "readme.md"
        || lower.ends_with(".md")
        || lower.starts_with("docs/")
        || lower.starts_with(".planning/")
}

fn is_entrypoint_file(rel_path: &str) -> bool {
    let lower = rel_path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(lower.as_str());
    name == "dockerfile"
        || name == "procfile"
        || name == "makefile"
        || lower.starts_with(".github/workflows/")
        || lower.starts_with("scripts/")
        || lower.ends_with(".service")
        || (name.starts_with("docker-compose")
            && (name.ends_with(".yml") || name.ends_with(".yaml")))
        || (name.starts_with("pm2") && name.ends_with(".json"))
}

fn is_docs_or_ownership_file(rel_path: &str) -> bool {
    let lower = rel_path.to_ascii_lowercase();
    lower == "owners"
        || lower == "codeowners"
        || lower.ends_with("/owners")
        || lower.ends_with("/codeowners")
        || lower == "architecture/standards.md"
        || lower.starts_with("openspec/")
        || lower.starts_with(".planning/")
        || lower.starts_with("docs/")
        || lower.ends_with(".md")
}

fn is_frontend_source_file(rel_path: &str) -> bool {
    let lower = rel_path.to_ascii_lowercase();
    (lower.starts_with("web/")
        || lower.starts_with("frontend/")
        || lower.starts_with("src/")
        || lower.starts_with("app/"))
        && matches!(
            lower.rsplit('.').next(),
            Some("ts" | "tsx" | "js" | "jsx" | "vue" | "svelte")
        )
}

fn frontend_marker_exists(root: &Path) -> bool {
    root.join("web").is_dir()
        || root.join("frontend").is_dir()
        || root.join("package.json").exists()
}

fn python_marker_exists(root: &Path) -> bool {
    root.join("pyproject.toml").exists()
        || root.join("requirements.txt").exists()
        || root.join("setup.py").exists()
}

fn fastapi_marker_exists(root: &Path) -> bool {
    if !python_marker_exists(root) {
        return false;
    }
    let candidates = ["pyproject.toml", "requirements.txt"];
    candidates.iter().any(|name| {
        fs::read_to_string(root.join(name))
            .map(|text| text.to_ascii_lowercase().contains("fastapi"))
            .unwrap_or(false)
    })
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

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
}
