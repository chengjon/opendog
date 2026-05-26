use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(super) const INTERNAL_SCANNERS: &[&str] = &[
    "candidate_collector",
    "entrypoint_scanner",
    "docs_ownership_gate",
    "frontend_literal_scanner",
];

pub(super) const DEFAULT_REQUIRED_SCANNERS: &[&str] = &[
    "candidate_collector",
    "entrypoint_scanner",
    "docs_ownership_gate",
];

pub(super) const KNOWN_SIGNAL_KINDS: &[&str] = &[
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn classification_options_default_includes_required_scanners() {
        let opts = ClassificationOptions::default();
        assert!(opts
            .required_scanners
            .contains(&"candidate_collector".to_string()));
        assert!(opts
            .required_scanners
            .contains(&"entrypoint_scanner".to_string()));
        assert!(opts
            .required_scanners
            .contains(&"docs_ownership_gate".to_string()));
        assert_eq!(opts.used_signal_threshold, 0.80);
        assert!(opts.max_age_secs.is_none());
        assert!(opts.now_secs.is_none());
    }

    #[test]
    fn scan_orphans_input_default_enables_internal_scanners() {
        let input = ScanOrphansInput::default();
        assert!(input.include_internal_scanners);
        assert!(input.include_evidence);
        assert!(input.subjects.is_none());
        assert!(input.external_reports.is_empty());
        assert!(input.required_scanners.is_none());
        assert!(input.max_age_secs.is_none());
        assert!(input.limit.is_none());
    }

    #[test]
    fn known_signal_kinds_includes_core_kinds() {
        assert!(KNOWN_SIGNAL_KINDS.contains(&"incoming_ref"));
        assert!(KNOWN_SIGNAL_KINDS.contains(&"candidate_collector"));
        assert!(KNOWN_SIGNAL_KINDS.contains(&"entrypoint"));
        assert!(KNOWN_SIGNAL_KINDS.contains(&"frontend_consumer"));
        assert!(KNOWN_SIGNAL_KINDS.contains(&"docs_owner"));
    }

    #[test]
    fn internal_scanners_includes_all_four() {
        assert_eq!(INTERNAL_SCANNERS.len(), 4);
        assert!(INTERNAL_SCANNERS.contains(&"candidate_collector"));
        assert!(INTERNAL_SCANNERS.contains(&"entrypoint_scanner"));
        assert!(INTERNAL_SCANNERS.contains(&"docs_ownership_gate"));
        assert!(INTERNAL_SCANNERS.contains(&"frontend_literal_scanner"));
    }

    #[test]
    fn default_required_scanners_subset_of_internal() {
        for scanner in DEFAULT_REQUIRED_SCANNERS {
            assert!(
                INTERNAL_SCANNERS.contains(scanner),
                "{} not in INTERNAL_SCANNERS",
                scanner
            );
        }
    }

    #[test]
    fn orphan_subject_kind_serde_roundtrip() {
        for kind in [
            OrphanSubjectKind::File,
            OrphanSubjectKind::Module,
            OrphanSubjectKind::Route,
            OrphanSubjectKind::Url,
            OrphanSubjectKind::Command,
            OrphanSubjectKind::Unknown,
        ] {
            let json_str = serde_json::to_string(&kind).unwrap();
            let back: OrphanSubjectKind = serde_json::from_str(&json_str).unwrap();
            assert_eq!(kind, back);
        }
    }

    #[test]
    fn evidence_polarity_serde_roundtrip() {
        for polarity in [
            EvidencePolarity::SupportsUsed,
            EvidencePolarity::SupportsUnused,
            EvidencePolarity::Veto,
            EvidencePolarity::Informational,
        ] {
            let json_str = serde_json::to_string(&polarity).unwrap();
            let back: EvidencePolarity = serde_json::from_str(&json_str).unwrap();
            assert_eq!(polarity, back);
        }
    }

    #[test]
    fn scanner_health_serde_roundtrip() {
        for health in [
            ScannerHealth::Passed,
            ScannerHealth::PassedWithWarnings,
            ScannerHealth::Skipped,
            ScannerHealth::Failed,
            ScannerHealth::Unavailable,
        ] {
            let json_str = serde_json::to_string(&health).unwrap();
            let back: ScannerHealth = serde_json::from_str(&json_str).unwrap();
            assert_eq!(health, back);
        }
    }

    #[test]
    fn orphan_classification_serde_roundtrip() {
        for class in [
            OrphanClassification::RemoveCandidate,
            OrphanClassification::ReviewRequired,
            OrphanClassification::Blocked,
        ] {
            let json_str = serde_json::to_string(&class).unwrap();
            let back: OrphanClassification = serde_json::from_str(&json_str).unwrap();
            assert_eq!(class, back);
        }
    }

    #[test]
    fn evidence_signal_serde_roundtrip() {
        let signal = EvidenceSignal {
            source: "test".to_string(),
            source_kind: "rust_internal".to_string(),
            signal_kind: "candidate_collector".to_string(),
            polarity: EvidencePolarity::SupportsUnused,
            confidence: 0.75,
            observed_at: Some(12345),
            subject: OrphanSubject {
                subject_kind: OrphanSubjectKind::File,
                subject: "dead.rs".to_string(),
                path: Some("src/dead.rs".to_string()),
                display_name: None,
            },
            detail: json!({"reason": "test"}),
        };
        let json_str = serde_json::to_string(&signal).unwrap();
        let back: EvidenceSignal = serde_json::from_str(&json_str).unwrap();
        assert_eq!(signal.source, back.source);
        assert_eq!(signal.polarity, back.polarity);
        assert_eq!(signal.confidence, back.confidence);
        assert_eq!(signal.subject.subject, back.subject.subject);
    }

    #[test]
    fn orphan_subject_serde_roundtrip() {
        let subject = OrphanSubject {
            subject_kind: OrphanSubjectKind::Module,
            subject: "utils".to_string(),
            path: Some("src/utils.py".to_string()),
            display_name: Some("Utility module".to_string()),
        };
        let json_str = serde_json::to_string(&subject).unwrap();
        let back: OrphanSubject = serde_json::from_str(&json_str).unwrap();
        assert_eq!(subject, back);
    }

    #[test]
    fn scan_orphans_result_serde_roundtrip() {
        let result = ScanOrphansResult {
            status: "complete".to_string(),
            scan_run_id: Some(42),
            scanner_health: vec![ScannerHealthEntry {
                scanner: "candidate_collector".to_string(),
                health: ScannerHealth::Passed,
                warnings: vec![],
                errors: vec![],
            }],
            summary: OrphanScanSummary {
                total_candidates: 3,
                remove_candidate_count: 1,
                review_required_count: 1,
                blocked_count: 1,
            },
            candidates: vec![],
            warnings: vec!["test warning".to_string()],
            recommended_next_actions: vec!["review candidates".to_string()],
        };
        let json_str = serde_json::to_string(&result).unwrap();
        let back: ScanOrphansResult = serde_json::from_str(&json_str).unwrap();
        assert_eq!(result.status, back.status);
        assert_eq!(result.scan_run_id, back.scan_run_id);
        assert_eq!(
            result.summary.total_candidates,
            back.summary.total_candidates
        );
    }

    #[test]
    fn deletion_plan_input_serde_roundtrip() {
        let input = DeletionPlanInput {
            targets: vec![OrphanSubject {
                subject_kind: OrphanSubjectKind::File,
                subject: "old.rs".to_string(),
                path: Some("src/old.rs".to_string()),
                display_name: None,
            }],
            external_reports: vec![],
            required_project_verification_commands: vec!["cargo test".to_string()],
            max_age_secs: Some(3600),
        };
        let json_str = serde_json::to_string(&input).unwrap();
        let back: DeletionPlanInput = serde_json::from_str(&json_str).unwrap();
        assert_eq!(input.targets.len(), back.targets.len());
        assert_eq!(
            input.required_project_verification_commands,
            back.required_project_verification_commands
        );
    }

    #[test]
    fn deletion_plan_verification_serde_roundtrip() {
        let verification = DeletionPlanVerification {
            status: "ready".to_string(),
            safe_to_plan_deletion: true,
            blocked_targets: vec![],
            review_required_targets: vec![],
            remove_candidates: vec![],
            required_project_verification_commands: vec!["cargo test".to_string()],
            evidence_gaps: vec![],
        };
        let json_str = serde_json::to_string(&verification).unwrap();
        let back: DeletionPlanVerification = serde_json::from_str(&json_str).unwrap();
        assert_eq!(verification.status, back.status);
        assert_eq!(
            verification.safe_to_plan_deletion,
            back.safe_to_plan_deletion
        );
    }

    #[test]
    fn external_scanner_report_serde_roundtrip() {
        let report = ExternalScannerReport {
            scanner: "ext".to_string(),
            version: "1.0".to_string(),
            health: ScannerHealth::PassedWithWarnings,
            started_at: Some(100),
            finished_at: Some(200),
            evidence: vec![],
            warnings: vec!["slow".to_string()],
            errors: vec![],
        };
        let json_str = serde_json::to_string(&report).unwrap();
        let back: ExternalScannerReport = serde_json::from_str(&json_str).unwrap();
        assert_eq!(report.scanner, back.scanner);
        assert_eq!(report.health, back.health);
        assert_eq!(report.warnings, back.warnings);
    }

    #[test]
    fn scanner_health_entry_serde_roundtrip() {
        let entry = ScannerHealthEntry {
            scanner: "test_scanner".to_string(),
            health: ScannerHealth::Failed,
            warnings: vec!["w1".to_string()],
            errors: vec!["e1".to_string()],
        };
        let json_str = serde_json::to_string(&entry).unwrap();
        let back: ScannerHealthEntry = serde_json::from_str(&json_str).unwrap();
        assert_eq!(entry.scanner, back.scanner);
        assert_eq!(entry.health, back.health);
        assert_eq!(entry.warnings, back.warnings);
        assert_eq!(entry.errors, back.errors);
    }

    #[test]
    fn classified_orphan_candidate_serde_roundtrip() {
        let candidate = ClassifiedOrphanCandidate {
            subject: OrphanSubject {
                subject_kind: OrphanSubjectKind::File,
                subject: "dead.rs".to_string(),
                path: Some("src/dead.rs".to_string()),
                display_name: None,
            },
            classification: OrphanClassification::RemoveCandidate,
            confidence: 0.95,
            reasons: vec!["No references found".to_string()],
            vetoes: vec![],
            evidence: vec![],
        };
        let json_str = serde_json::to_string(&candidate).unwrap();
        let back: ClassifiedOrphanCandidate = serde_json::from_str(&json_str).unwrap();
        assert_eq!(candidate.subject, back.subject);
        assert_eq!(candidate.classification, back.classification);
        assert_eq!(candidate.confidence, back.confidence);
    }
}
