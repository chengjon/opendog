use serde_json::{json, Value};

use crate::contracts::{
    versioned_project_payload, MCP_ORPHAN_DELETION_PLAN_V1, MCP_ORPHAN_SCAN_V1,
};
use crate::core::orphan::{DeletionPlanVerification, ScanOrphansResult};

pub(crate) fn orphan_scan_payload(project_id: &str, result: &ScanOrphansResult) -> Value {
    versioned_project_payload(
        MCP_ORPHAN_SCAN_V1,
        project_id,
        [
            ("status", json!(result.status)),
            ("scan_run_id", json!(result.scan_run_id)),
            ("scanner_health", json!(result.scanner_health)),
            ("summary", json!(result.summary)),
            ("candidates", json!(result.candidates)),
            ("warnings", json!(result.warnings)),
            (
                "recommended_next_actions",
                json!(result.recommended_next_actions),
            ),
        ],
    )
}

pub(crate) fn orphan_deletion_plan_payload(
    project_id: &str,
    result: &DeletionPlanVerification,
) -> Value {
    versioned_project_payload(
        MCP_ORPHAN_DELETION_PLAN_V1,
        project_id,
        [
            ("status", json!(result.status)),
            ("safe_to_plan_deletion", json!(result.safe_to_plan_deletion)),
            ("blocked_targets", json!(result.blocked_targets)),
            (
                "review_required_targets",
                json!(result.review_required_targets),
            ),
            ("remove_candidates", json!(result.remove_candidates)),
            (
                "required_project_verification_commands",
                json!(result.required_project_verification_commands),
            ),
            ("evidence_gaps", json!(result.evidence_gaps)),
        ],
    )
}
