use super::*;
use crate::core::orphan::{OrphanSubject, OrphanSubjectKind};
use serde_json::json;

// ---- RecordVerificationParams::into_parts ----

#[test]
fn record_verification_into_parts_maps_required_fields() {
    let params = RecordVerificationParams {
        id: "proj-1".to_string(),
        kind: "test".to_string(),
        status: "passed".to_string(),
        command: "cargo test".to_string(),
        exit_code: None,
        summary: None,
        source: None,
        started_at: None,
    };
    let (id, input) = params.into_parts();
    assert_eq!(id, "proj-1");
    assert_eq!(input.kind, "test");
    assert_eq!(input.status, "passed");
    assert_eq!(input.command, "cargo test");
}

#[test]
fn record_verification_into_parts_source_defaults_to_mcp() {
    let params = RecordVerificationParams {
        id: "proj-2".to_string(),
        kind: "lint".to_string(),
        status: "failed".to_string(),
        command: "clippy".to_string(),
        exit_code: None,
        summary: None,
        source: None,
        started_at: None,
    };
    let (_, input) = params.into_parts();
    assert_eq!(input.source, "mcp");
}

#[test]
fn record_verification_into_parts_preserves_all_optionals() {
    let params = RecordVerificationParams {
        id: "proj-3".to_string(),
        kind: "build".to_string(),
        status: "passed".to_string(),
        command: "cargo build".to_string(),
        exit_code: Some(0),
        summary: Some("all good".to_string()),
        source: Some("cli".to_string()),
        started_at: Some("2026-01-01T00:00:00Z".to_string()),
    };
    let (_, input) = params.into_parts();
    assert_eq!(input.exit_code, Some(0));
    assert_eq!(input.summary, Some("all good".to_string()));
    assert_eq!(input.source, "cli");
    assert_eq!(input.started_at, Some("2026-01-01T00:00:00Z".to_string()));
}

// ---- ExecuteVerificationParams::into_parts ----

#[test]
fn execute_verification_into_parts_maps_required_fields() {
    let params = ExecuteVerificationParams {
        id: "proj-4".to_string(),
        kind: "test".to_string(),
        command: "cargo test".to_string(),
        source: None,
    };
    let (id, input) = params.into_parts();
    assert_eq!(id, "proj-4");
    assert_eq!(input.kind, "test");
    assert_eq!(input.command, "cargo test");
}

#[test]
fn execute_verification_into_parts_source_defaults_to_mcp() {
    let params = ExecuteVerificationParams {
        id: "proj-5".to_string(),
        kind: "lint".to_string(),
        command: "clippy".to_string(),
        source: None,
    };
    let (_, input) = params.into_parts();
    assert_eq!(input.source, "mcp");
}

#[test]
fn execute_verification_into_parts_preserves_explicit_source() {
    let params = ExecuteVerificationParams {
        id: "proj-6".to_string(),
        kind: "build".to_string(),
        command: "cargo build".to_string(),
        source: Some("ci".to_string()),
    };
    let (_, input) = params.into_parts();
    assert_eq!(input.source, "ci");
}

// ---- ScanOrphansParams::into_parts ----

#[test]
fn scan_orphans_into_parts_defaults() {
    let params = ScanOrphansParams {
        id: "proj-7".to_string(),
        subjects: None,
        external_reports: None,
        include_internal_scanners: None,
        required_scanners: None,
        max_age_secs: None,
        limit: None,
        include_evidence: None,
    };
    let (id, input) = params.into_parts();
    assert_eq!(id, "proj-7");
    assert!(input.subjects.is_none());
    assert!(input.external_reports.is_empty());
    assert!(input.include_internal_scanners);
    assert!(input.required_scanners.is_none());
    assert!(input.max_age_secs.is_none());
    assert!(input.limit.is_none());
    assert!(input.include_evidence);
}

#[test]
fn scan_orphans_into_parts_preserves_explicit_values() {
    let params = ScanOrphansParams {
        id: "proj-8".to_string(),
        subjects: None,
        external_reports: None,
        include_internal_scanners: Some(false),
        required_scanners: Some(vec!["scanner-a".to_string()]),
        max_age_secs: Some(3600),
        limit: Some(10),
        include_evidence: Some(false),
    };
    let (_, input) = params.into_parts();
    assert!(!input.include_internal_scanners);
    assert_eq!(input.required_scanners, Some(vec!["scanner-a".to_string()]));
    assert_eq!(input.max_age_secs, Some(3600));
    assert_eq!(input.limit, Some(10));
    assert!(!input.include_evidence);
}

// ---- VerifyDeletionPlanParams::into_parts ----

#[test]
fn verify_deletion_plan_into_parts_defaults() {
    let params = VerifyDeletionPlanParams {
        id: "proj-9".to_string(),
        targets: vec![],
        external_reports: None,
        required_project_verification_commands: None,
        max_age_secs: None,
    };
    let (id, input) = params.into_parts();
    assert_eq!(id, "proj-9");
    assert!(input.targets.is_empty());
    assert!(input.external_reports.is_empty());
    assert!(input.required_project_verification_commands.is_empty());
    assert!(input.max_age_secs.is_none());
}

#[test]
fn verify_deletion_plan_into_parts_with_targets() {
    let target = OrphanSubject {
        subject_kind: OrphanSubjectKind::File,
        subject: "dead.rs".to_string(),
        path: Some("src/dead.rs".to_string()),
        display_name: None,
    };
    let params = VerifyDeletionPlanParams {
        id: "proj-10".to_string(),
        targets: vec![target.clone()],
        external_reports: None,
        required_project_verification_commands: Some(vec!["cargo test".to_string()]),
        max_age_secs: Some(7200),
    };
    let (id, input) = params.into_parts();
    assert_eq!(id, "proj-10");
    assert_eq!(input.targets.len(), 1);
    assert_eq!(input.targets[0].subject, "dead.rs");
    assert_eq!(
        input.required_project_verification_commands,
        vec!["cargo test".to_string()]
    );
    assert_eq!(input.max_age_secs, Some(7200));
}

// ---- CreateGovernanceLaneParams::into_parts ----

#[test]
fn create_governance_lane_into_parts_without_description() {
    let params = CreateGovernanceLaneParams {
        id: "proj-11".to_string(),
        lane_id: "lane-1".to_string(),
        title: "My Lane".to_string(),
        description: None,
    };
    let (id, input) = params.into_parts();
    assert_eq!(id, "proj-11");
    assert_eq!(input.lane_id, "lane-1");
    assert_eq!(input.title, "My Lane");
    assert!(input.description.is_none());
}

#[test]
fn create_governance_lane_into_parts_with_description() {
    let params = CreateGovernanceLaneParams {
        id: "proj-12".to_string(),
        lane_id: "lane-2".to_string(),
        title: "Lane Two".to_string(),
        description: Some("A description".to_string()),
    };
    let (_, input) = params.into_parts();
    assert_eq!(input.description, Some("A description".to_string()));
}

// ---- UpsertGovernanceNodeParams::into_parts ----

#[test]
fn upsert_governance_node_into_parts_minimal() {
    let params = UpsertGovernanceNodeParams {
        id: "proj-13".to_string(),
        lane_id: "lane-3".to_string(),
        node_id: "G2.46".to_string(),
        state: None,
        summary: None,
        evidence_refs: None,
        artifact_refs: None,
        reported_git_head: None,
        suggested_next: None,
        forbidden_scope: None,
        external_anchors: None,
    };
    let (id, input) = params.into_parts();
    assert_eq!(id, "proj-13");
    assert_eq!(input.node_id, "G2.46");
    assert_eq!(input.lane_id, "lane-3");
    assert!(input.state.is_none());
    assert!(input.summary.is_none());
    assert!(input.external_anchors.is_none());
}

#[test]
fn upsert_governance_node_into_parts_all_fields() {
    let params = UpsertGovernanceNodeParams {
        id: "proj-14".to_string(),
        lane_id: "lane-4".to_string(),
        node_id: "N1".to_string(),
        state: Some("in_progress".to_string()),
        summary: Some("doing work".to_string()),
        evidence_refs: Some(vec!["doc.md".to_string()]),
        artifact_refs: Some(vec!["output.txt".to_string()]),
        reported_git_head: Some("abc123".to_string()),
        suggested_next: Some("continue".to_string()),
        forbidden_scope: Some(vec!["scope-a".to_string()]),
        external_anchors: Some(json!({"key": "value"})),
    };
    let (_, input) = params.into_parts();
    assert_eq!(input.state, Some("in_progress".to_string()));
    assert_eq!(input.summary, Some("doing work".to_string()));
    assert_eq!(input.evidence_refs, Some(vec!["doc.md".to_string()]));
    assert_eq!(input.artifact_refs, Some(vec!["output.txt".to_string()]));
    assert_eq!(input.reported_git_head, Some("abc123".to_string()));
    assert_eq!(input.suggested_next, Some("continue".to_string()));
    assert_eq!(input.forbidden_scope, Some(vec!["scope-a".to_string()]));
    assert_eq!(input.external_anchors, Some(json!({"key": "value"})));
}

// ---- GetGovernanceStateParams::into_parts ----

#[test]
fn get_governance_state_into_parts_minimal() {
    let params = GetGovernanceStateParams {
        id: "proj-15".to_string(),
        lane_id: None,
        node_id: None,
        active_only: None,
    };
    let (id, input) = params.into_parts();
    assert_eq!(id, "proj-15");
    assert!(input.lane_id.is_none());
    assert!(input.node_id.is_none());
    assert!(input.active_only.is_none());
}

#[test]
fn get_governance_state_into_parts_with_filters() {
    let params = GetGovernanceStateParams {
        id: "proj-16".to_string(),
        lane_id: Some("lane-5".to_string()),
        node_id: Some("N2".to_string()),
        active_only: Some(true),
    };
    let (_, input) = params.into_parts();
    assert_eq!(input.lane_id, Some("lane-5".to_string()));
    assert_eq!(input.node_id, Some("N2".to_string()));
    assert_eq!(input.active_only, Some(true));
}

// ---- CloseGovernanceLaneParams::into_parts ----

#[test]
fn close_governance_lane_into_parts() {
    let params = CloseGovernanceLaneParams {
        id: "proj-17".to_string(),
        lane_id: "lane-6".to_string(),
        action: "complete".to_string(),
    };
    let (id, input) = params.into_parts();
    assert_eq!(id, "proj-17");
    assert_eq!(input.lane_id, "lane-6");
    assert_eq!(input.action, "complete");
}
