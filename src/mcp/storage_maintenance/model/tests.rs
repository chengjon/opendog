use crate::config::RetentionPolicy;
use crate::core::retention::StorageMetrics;
use serde_json::json;

use super::{
    StorageCleanupScope, StorageMaintenanceAssessment, StorageMaintenanceTemplateContext,
    StorageMaintenanceWorkspaceSummary, CLEANUP_PLAN_PHASE_EXECUTE_CLEANUP,
};

fn policy() -> RetentionPolicy {
    RetentionPolicy {
        cleanup_review_db_bytes_threshold: 100,
        vacuum_reclaimable_bytes_threshold: 20,
        vacuum_reclaim_ratio_threshold_percent: 25,
        ..Default::default()
    }
}

#[test]
fn template_context_captures_project_and_pressure_fields() {
    let context = StorageMaintenanceTemplateContext::from_inputs(
        None,
        &json!({
            "maintenance_candidate": true,
            "vacuum_candidate": true,
            "approx_reclaimable_bytes": 42,
        }),
    );

    assert!(context.should_emit_templates());
    assert_eq!(context.project_id_value(), "<project>");
    assert!(context.project_placeholder_required());
    assert!(context.vacuum_candidate);
    assert_eq!(context.approx_reclaimable_bytes, 42);
}

#[test]
fn template_context_parses_recommendations_and_cleanup_plan_steps() {
    let context = StorageMaintenanceTemplateContext::from_inputs(
        Some("proj"),
        &json!({
            "maintenance_candidate": true,
            "cleanup_recommendations": [
                {"scope": "activity", "older_than_days": 14},
                {"scope": "snapshots", "keep_snapshot_runs": 12},
                {"scope": "unknown", "older_than_days": 1}
            ],
            "cleanup_plan": {
                "steps": [
                    {"phase": "prepare_cleanup", "scope": "all"},
                    {"phase": CLEANUP_PLAN_PHASE_EXECUTE_CLEANUP, "scope": "verification", "retention_parameters": {"older_than_days": 21}},
                    {"phase": CLEANUP_PLAN_PHASE_EXECUTE_CLEANUP, "scope": "snapshots", "retention_parameters": {"keep_snapshot_runs": 7}},
                    {"phase": CLEANUP_PLAN_PHASE_EXECUTE_CLEANUP, "scope": "unknown"}
                ]
            }
        }),
    );

    assert_eq!(context.project_id_value(), "proj");
    assert!(!context.project_placeholder_required());
    assert_eq!(context.cleanup_recommendations.len(), 2);
    assert_eq!(
        context.cleanup_recommendations[0].scope,
        StorageCleanupScope::Activity
    );
    assert_eq!(context.cleanup_recommendations[0].older_than_days, Some(14));
    assert_eq!(
        context.cleanup_recommendations[1].scope,
        StorageCleanupScope::Snapshots
    );
    assert_eq!(
        context.cleanup_recommendations[1].keep_snapshot_runs,
        Some(12)
    );
    assert_eq!(context.cleanup_plan_steps.len(), 2);
    assert_eq!(
        context.cleanup_plan_steps[0].scope,
        StorageCleanupScope::Verification
    );
    assert_eq!(context.cleanup_plan_steps[0].older_than_days, Some(21));
    assert_eq!(
        context.cleanup_plan_steps[1].scope,
        StorageCleanupScope::Snapshots
    );
    assert_eq!(context.cleanup_plan_steps[1].keep_snapshot_runs, Some(7));
}

#[test]
fn assessment_marks_vacuum_candidate_as_high_priority() {
    let assessment = StorageMaintenanceAssessment::from_inputs(
        &StorageMetrics {
            approx_db_size_bytes: 100,
            approx_reclaimable_bytes: 30,
            ..Default::default()
        },
        false,
        &policy(),
    );

    assert!(assessment.vacuum_candidate);
    assert!(assessment.maintenance_candidate);
    assert_eq!(assessment.suggested_mode, "review_cleanup_then_vacuum");
    assert_eq!(assessment.pressure_level, "high");
}

#[test]
fn assessment_uses_evidence_pressure_without_large_database() {
    let assessment = StorageMaintenanceAssessment::from_inputs(
        &StorageMetrics {
            approx_db_size_bytes: 10,
            approx_reclaimable_bytes: 0,
            ..Default::default()
        },
        true,
        &policy(),
    );

    assert!(!assessment.cleanup_review_candidate);
    assert!(assessment.evidence_pressure_candidate);
    assert!(assessment.maintenance_candidate);
    assert_eq!(assessment.suggested_mode, "review_cleanup");
    assert_eq!(assessment.pressure_level, "high");
}

#[test]
fn assessment_keeps_small_clean_database_low_priority() {
    let assessment = StorageMaintenanceAssessment::from_inputs(
        &StorageMetrics {
            approx_db_size_bytes: 10,
            approx_reclaimable_bytes: 0,
            ..Default::default()
        },
        false,
        &policy(),
    );

    assert!(!assessment.maintenance_candidate);
    assert_eq!(assessment.suggested_mode, "none");
    assert_eq!(assessment.pressure_level, "low");
}

#[test]
fn workspace_summary_aggregates_and_sorts_priority_projects() {
    let project_overviews = vec![
        json!({
            "project_id": "large-cleanup",
            "status": "registered",
            "storage_maintenance": {
                "maintenance_candidate": true,
                "vacuum_candidate": false,
                "cleanup_review_candidate": true,
                "approx_db_size_bytes": 900,
                "approx_reclaimable_bytes": 100,
                "reclaim_ratio": 0.11,
                "suggested_mode": "review_cleanup",
                "summary": "large database"
            }
        }),
        json!({
            "project_id": "vacuum-first",
            "status": "registered",
            "storage_maintenance": {
                "maintenance_candidate": true,
                "vacuum_candidate": true,
                "cleanup_review_candidate": true,
                "approx_db_size_bytes": 500,
                "approx_reclaimable_bytes": 200,
                "reclaim_ratio": 0.4,
                "suggested_mode": "review_cleanup_then_vacuum",
                "summary": "vacuum candidate"
            }
        }),
        json!({
            "project_id": "healthy",
            "status": "registered",
            "storage_maintenance": {
                "maintenance_candidate": false,
                "vacuum_candidate": false,
                "cleanup_review_candidate": false,
                "approx_db_size_bytes": 50,
                "approx_reclaimable_bytes": 0,
                "reclaim_ratio": 0.0,
                "suggested_mode": "none",
                "summary": "healthy"
            }
        }),
    ];

    let summary = StorageMaintenanceWorkspaceSummary::from_project_overviews(&project_overviews);

    assert_eq!(summary.projects_with_candidates, 2);
    assert_eq!(summary.projects_with_vacuum_candidates, 1);
    assert_eq!(summary.total_approx_db_size_bytes, 1450);
    assert_eq!(summary.total_approx_reclaimable_bytes, 300);
    assert_eq!(
        summary.priority_projects[0].project_id.as_deref(),
        Some("vacuum-first")
    );
    assert_eq!(
        summary.priority_projects[1].project_id.as_deref(),
        Some("large-cleanup")
    );
}

#[test]
fn priority_project_json_preserves_contract_shape() {
    let project_overviews = vec![json!({
        "project_id": "demo",
        "status": "registered",
        "storage_maintenance": {
            "maintenance_candidate": true,
            "vacuum_candidate": true,
            "cleanup_review_candidate": true,
            "approx_db_size_bytes": 500,
            "approx_reclaimable_bytes": 200,
            "reclaim_ratio": 0.4,
            "suggested_mode": "review_cleanup_then_vacuum",
            "summary": "vacuum candidate"
        }
    })];
    let summary = StorageMaintenanceWorkspaceSummary::from_project_overviews(&project_overviews);

    assert_eq!(
        summary.priority_projects_json(),
        vec![json!({
            "project_id": "demo",
            "status": "registered",
            "vacuum_candidate": true,
            "cleanup_review_candidate": true,
            "approx_db_size_bytes": 500,
            "approx_reclaimable_bytes": 200,
            "reclaim_ratio": 0.4,
            "suggested_mode": "review_cleanup_then_vacuum",
            "summary": "vacuum candidate"
        })]
    );
}
