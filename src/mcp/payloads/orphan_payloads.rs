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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::orphan::{
        ClassifiedOrphanCandidate, DeletionPlanVerification, OrphanScanSummary, ScanOrphansResult,
    };

    fn sample_scan_result() -> ScanOrphansResult {
        ScanOrphansResult {
            status: "completed".to_string(),
            scan_run_id: Some(42),
            scanner_health: vec![],
            summary: OrphanScanSummary {
                total_candidates: 5,
                remove_candidate_count: 2,
                review_required_count: 2,
                blocked_count: 1,
            },
            candidates: vec![],
            warnings: vec!["warn1".to_string()],
            recommended_next_actions: vec!["action1".to_string()],
        }
    }

    fn sample_deletion_plan() -> DeletionPlanVerification {
        DeletionPlanVerification {
            status: "review_required".to_string(),
            safe_to_plan_deletion: false,
            blocked_targets: vec![],
            review_required_targets: vec![],
            remove_candidates: vec![],
            required_project_verification_commands: vec!["cargo test".to_string()],
            evidence_gaps: vec!["gap1".to_string()],
        }
    }

    #[test]
    fn orphan_scan_payload_fields() {
        let result = sample_scan_result();
        let payload = orphan_scan_payload("proj1", &result);
        assert_eq!(payload["project_id"], "proj1");
        assert_eq!(payload["status"], "completed");
        assert_eq!(payload["scan_run_id"], 42);
        assert_eq!(payload["summary"]["total_candidates"], 5);
        assert_eq!(payload["summary"]["remove_candidate_count"], 2);
        assert_eq!(payload["summary"]["review_required_count"], 2);
        assert_eq!(payload["summary"]["blocked_count"], 1);
        let warnings = payload["warnings"].as_array().unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0], "warn1");
        let actions = payload["recommended_next_actions"].as_array().unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], "action1");
    }

    #[test]
    fn orphan_scan_payload_empty_candidates() {
        let result = ScanOrphansResult {
            status: "completed".to_string(),
            scan_run_id: None,
            scanner_health: vec![],
            summary: OrphanScanSummary {
                total_candidates: 0,
                remove_candidate_count: 0,
                review_required_count: 0,
                blocked_count: 0,
            },
            candidates: vec![],
            warnings: vec![],
            recommended_next_actions: vec![],
        };
        let payload = orphan_scan_payload("empty", &result);
        assert_eq!(payload["project_id"], "empty");
        assert!(payload["scan_run_id"].is_null());
        assert!(payload["candidates"].as_array().unwrap().is_empty());
        assert!(payload["warnings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn orphan_deletion_plan_payload_fields() {
        let plan = sample_deletion_plan();
        let payload = orphan_deletion_plan_payload("proj2", &plan);
        assert_eq!(payload["project_id"], "proj2");
        assert_eq!(payload["status"], "review_required");
        assert_eq!(payload["safe_to_plan_deletion"], false);
        assert!(payload["blocked_targets"].as_array().unwrap().is_empty());
        assert!(payload["review_required_targets"].as_array().unwrap().is_empty());
        assert!(payload["remove_candidates"].as_array().unwrap().is_empty());
        let cmds = payload["required_project_verification_commands"].as_array().unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], "cargo test");
        let gaps = payload["evidence_gaps"].as_array().unwrap();
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0], "gap1");
    }

    #[test]
    fn orphan_deletion_plan_payload_safe_to_delete() {
        let plan = DeletionPlanVerification {
            status: "safe".to_string(),
            safe_to_plan_deletion: true,
            blocked_targets: vec![],
            review_required_targets: vec![],
            remove_candidates: vec![],
            required_project_verification_commands: vec![],
            evidence_gaps: vec![],
        };
        let payload = orphan_deletion_plan_payload("safe-proj", &plan);
        assert_eq!(payload["status"], "safe");
        assert_eq!(payload["safe_to_plan_deletion"], true);
    }
}
