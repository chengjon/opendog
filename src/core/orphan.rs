mod classification;
mod deletion_plan;
mod evidence;
mod path_rules;
mod scan;
mod scanner_contract;
mod types;

pub use self::classification::classify_subject;
pub use self::deletion_plan::verify_deletion_plan;
pub use self::scan::scan_project_orphans;
pub use self::scanner_contract::validate_required_scanners;
pub use self::types::{
    ClassificationOptions, ClassifiedOrphanCandidate, DeletionPlanInput, DeletionPlanVerification,
    EvidencePolarity, EvidenceSignal, ExternalScannerReport, OrphanClassification,
    OrphanScanSummary, OrphanSubject, OrphanSubjectKind, ScanOrphansInput, ScanOrphansResult,
    ScannerHealth, ScannerHealthEntry,
};

#[cfg(test)]
mod tests;
