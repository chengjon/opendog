use serde_json::{json, Value};

use crate::config::RetentionPolicy;
use crate::core::retention::{StorageEvidenceCounts, StorageMetrics};

mod execution_templates;
mod model;
pub(super) use execution_templates::augment_entrypoints_for_storage_maintenance;
#[cfg(test)]
use execution_templates::storage_maintenance_execution_templates;
#[cfg(test)]
use model::storage_reclaim_ratio;
#[cfg(test)]
use model::CLEANUP_PLAN_PHASE_EXECUTE_CLEANUP;
use model::{StorageMaintenanceAssessment, StorageMaintenanceWorkspaceSummary};

fn evidence_counts_json(counts: Option<&StorageEvidenceCounts>) -> Value {
    counts.map_or(Value::Null, |counts| {
        json!({
            "file_sightings": counts.file_sightings,
            "file_events": counts.file_events,
            "activity_daily_rollups": counts.activity_daily_rollups,
            "verification_runs": counts.verification_runs,
            "snapshot_runs": counts.snapshot_runs,
        })
    })
}

fn storage_cleanup_recommendations(
    counts: Option<&StorageEvidenceCounts>,
    policy: &RetentionPolicy,
) -> Vec<Value> {
    let Some(counts) = counts else {
        return Vec::new();
    };

    let mut recommendations = Vec::new();
    if counts.file_sightings >= policy.activity_rows_threshold
        || counts.file_events >= policy.activity_rows_threshold
        || counts.file_sightings.saturating_add(counts.file_events)
            >= policy.activity_rows_threshold
    {
        recommendations.push(json!({
            "scope": "activity",
            "older_than_days": policy.activity_retention_days,
            "dry_run": true,
            "rollup_before_delete": true,
            "rollup_granularity": "daily",
            "preserved_rollup_table": "activity_daily_rollups",
            "reason": "activity evidence row counts exceed the storage pressure threshold",
            "threshold_rows": policy.activity_rows_threshold,
            "row_counts": {
                "file_sightings": counts.file_sightings,
                "file_events": counts.file_events,
            }
        }));
    }

    if counts.snapshot_runs >= policy.snapshot_runs_threshold {
        recommendations.push(json!({
            "scope": "snapshots",
            "keep_snapshot_runs": policy.keep_snapshot_runs,
            "dry_run": true,
            "reason": "snapshot run count exceeds the storage pressure threshold",
            "threshold_runs": policy.snapshot_runs_threshold,
            "run_count": counts.snapshot_runs,
        }));
    }

    if counts.verification_runs >= policy.verification_runs_threshold {
        recommendations.push(json!({
            "scope": "verification",
            "older_than_days": policy.verification_retention_days,
            "dry_run": true,
            "reason": "verification run count exceeds the storage pressure threshold",
            "threshold_runs": policy.verification_runs_threshold,
            "run_count": counts.verification_runs,
        }));
    }

    recommendations
}

fn cleanup_retention_parameters(scope: &str, source: &Value, policy: &RetentionPolicy) -> Value {
    match scope {
        "snapshots" => json!({
            "keep_snapshot_runs": source["keep_snapshot_runs"]
                .as_i64()
                .unwrap_or(policy.keep_snapshot_runs)
        }),
        "verification" => json!({
            "older_than_days": source["older_than_days"]
                .as_i64()
                .unwrap_or(policy.verification_retention_days)
        }),
        _ => json!({
            "older_than_days": source["older_than_days"]
                .as_i64()
                .unwrap_or(policy.activity_retention_days)
        }),
    }
}

fn cleanup_plan_step(
    step: i64,
    phase: &str,
    scope: &str,
    source: &Value,
    policy: &RetentionPolicy,
    dry_run: bool,
    requires_human_confirmation: bool,
) -> Value {
    let mut value = json!({
        "step": step,
        "phase": phase,
        "scope": scope,
        "dry_run": dry_run,
        "requires_human_confirmation": requires_human_confirmation,
        "retention_parameters": cleanup_retention_parameters(scope, source, policy),
    });

    if let Some(days) = value["retention_parameters"]["older_than_days"].as_i64() {
        value["older_than_days"] = json!(days);
    }
    if let Some(keep_snapshot_runs) = value["retention_parameters"]["keep_snapshot_runs"].as_i64() {
        value["keep_snapshot_runs"] = json!(keep_snapshot_runs);
    }
    if scope == "activity" {
        value["rollup_before_delete"] = json!(true);
        value["rollup_granularity"] = json!("daily");
        value["preserved_rollup_table"] = json!("activity_daily_rollups");
    }

    value
}

fn storage_cleanup_plan(
    cleanup_recommendations: &[Value],
    cleanup_review_candidate: bool,
    vacuum_candidate: bool,
    policy: &RetentionPolicy,
) -> Value {
    let mut targets: Vec<(&str, Value)> = cleanup_recommendations
        .iter()
        .filter_map(|recommendation| {
            recommendation["scope"]
                .as_str()
                .map(|scope| (scope, recommendation.clone()))
        })
        .collect();

    if targets.is_empty() && (cleanup_review_candidate || vacuum_candidate) {
        targets.push((
            "all",
            json!({
                "scope": "all",
                "older_than_days": policy.activity_retention_days,
            }),
        ));
    }

    if targets.is_empty() {
        return json!({
            "status": "not_needed",
            "policy_driven": true,
            "automatic_deletion": false,
            "requires_human_confirmation": false,
            "target_scopes": [],
            "steps": [],
            "summary": "No OPENDOG retention cleanup plan is needed for current storage signals.",
        });
    }

    let mut steps = Vec::new();
    let mut step = 1;
    for (scope, recommendation) in &targets {
        steps.push(cleanup_plan_step(
            step,
            "preview",
            scope,
            recommendation,
            policy,
            true,
            false,
        ));
        step += 1;
    }

    steps.push(json!({
        "step": step,
        "phase": "review",
        "requires_human_confirmation": true,
        "summary": "Review cleanup-data dry-run deleted counts, retained evidence scope, and storage_before metrics before executing any deletion.",
    }));
    step += 1;

    for (scope, recommendation) in &targets {
        steps.push(cleanup_plan_step(
            step,
            "execute_cleanup",
            scope,
            recommendation,
            policy,
            false,
            true,
        ));
        step += 1;
    }

    if vacuum_candidate {
        steps.push(json!({
            "step": step,
            "phase": "compact",
            "scope": "all",
            "vacuum": true,
            "requires_human_confirmation": true,
            "summary": "Run vacuum only after cleanup execution when reclaimable pages remain material.",
        }));
    }

    json!({
        "status": "actionable",
        "policy_driven": true,
        "automatic_deletion": false,
        "requires_human_confirmation": true,
        "target_scopes": targets
            .iter()
            .map(|(scope, _)| json!(scope))
            .collect::<Vec<_>>(),
        "steps": steps,
        "summary": "Review dry-run output first, then run confirmed cleanup steps only after operator approval.",
    })
}

pub(super) fn project_storage_maintenance_with_policy(
    metrics: Option<&StorageMetrics>,
    evidence_counts: Option<&StorageEvidenceCounts>,
    policy: &RetentionPolicy,
) -> Value {
    let Some(metrics) = metrics else {
        return json!({
            "status": "unavailable",
            "cleanup_review_candidate": false,
            "evidence_pressure_candidate": false,
            "maintenance_candidate": false,
            "vacuum_candidate": false,
            "suggested_mode": "none",
            "pressure_level": "unknown",
            "evidence_counts": Value::Null,
            "retention_policy": policy,
            "cleanup_recommendations": [],
            "cleanup_plan": {
                "status": "unavailable",
                "policy_driven": true,
                "automatic_deletion": false,
                "requires_human_confirmation": false,
                "target_scopes": [],
                "steps": [],
                "summary": "Project database metrics are unavailable.",
            },
            "summary": "Project database metrics are unavailable.",
        });
    };

    let cleanup_recommendations = storage_cleanup_recommendations(evidence_counts, policy);
    let assessment = StorageMaintenanceAssessment::from_inputs(
        metrics,
        !cleanup_recommendations.is_empty(),
        policy,
    );
    let cleanup_plan = storage_cleanup_plan(
        &cleanup_recommendations,
        assessment.cleanup_review_candidate,
        assessment.vacuum_candidate,
        policy,
    );

    json!({
        "status": "available",
        "page_count": metrics.page_count,
        "freelist_count": metrics.freelist_count,
        "approx_db_size_bytes": metrics.approx_db_size_bytes,
        "approx_reclaimable_bytes": metrics.approx_reclaimable_bytes,
        "reclaim_ratio": assessment.reclaim_ratio,
        "cleanup_review_candidate": assessment.cleanup_review_candidate,
        "evidence_pressure_candidate": assessment.evidence_pressure_candidate,
        "maintenance_candidate": assessment.maintenance_candidate,
        "vacuum_candidate": assessment.vacuum_candidate,
        "suggested_mode": assessment.suggested_mode,
        "pressure_level": assessment.pressure_level,
        "evidence_counts": evidence_counts_json(evidence_counts),
        "retention_policy": policy,
        "cleanup_recommendations": cleanup_recommendations,
        "cleanup_plan": cleanup_plan,
        "summary": assessment.summary,
    })
}

pub(super) fn storage_maintenance_layer(project_overviews: &[Value]) -> Value {
    let summary = StorageMaintenanceWorkspaceSummary::from_project_overviews(project_overviews);

    json!({
        "status": "available",
        "projects_with_candidates": summary.projects_with_candidates,
        "projects_with_vacuum_candidates": summary.projects_with_vacuum_candidates,
        "total_approx_db_size_bytes": summary.total_approx_db_size_bytes,
        "total_approx_reclaimable_bytes": summary.total_approx_reclaimable_bytes,
        "priority_projects": summary.priority_projects_json(),
    })
}

#[cfg(test)]
mod planner_tests;

#[cfg(test)]
mod tests;
