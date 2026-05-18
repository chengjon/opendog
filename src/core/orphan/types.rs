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
