use serde_json::{json, Value};

use crate::config::RetentionPolicy;
use crate::core::retention::{StorageEvidenceCounts, StorageMetrics};

mod model;
#[cfg(test)]
use model::storage_reclaim_ratio;
use model::{
    StorageCleanupScope, StorageMaintenanceAssessment, StorageMaintenanceTemplateContext,
    StorageMaintenanceWorkspaceSummary,
};

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

fn storage_maintenance_execution_templates(
    project_id: Option<&str>,
    storage_maintenance: &Value,
) -> Vec<Value> {
    let context = StorageMaintenanceTemplateContext::from_inputs(project_id, storage_maintenance);
    if !context.should_emit_templates() {
        return Vec::new();
    }

    let project_id_value = context.project_id_value();
    let project_placeholder_hint = context.project_placeholder_hint_json();
    let reclaimable_bytes = context.approx_reclaimable_bytes;
    let default_policy = RetentionPolicy::default();
    let mut templates = vec![json!({
        "template_id": "storage.cleanup.preview",
        "kind": "cli_command",
        "command_template": format!(
            "opendog cleanup-data --id {} --scope all --older-than-days 30 --dry-run --json",
            project_id_value
        ),
        "preconditions": [
            "project must exist in OPENDOG",
            "use dry-run first before deleting retained OPENDOG evidence"
        ],
        "blocking_conditions": [],
        "success_signal": "cleanup preview returns deleted counts plus storage_before metrics",
        "parameter_schema": {
            "id": { "type": "string", "required": true, "source": "project_id" },
            "scope": { "type": "enum", "required": true, "allowed_values": ["activity", "snapshots", "verification", "all"] },
            "older_than_days": { "type": "integer", "required": false },
            "dry_run": { "type": "boolean", "required": false },
            "json": { "type": "boolean", "required": false }
        },
        "default_values": {
            "scope": "all",
            "older_than_days": 30,
            "dry_run": true,
            "json": true
        },
        "placeholder_hints": project_placeholder_hint.clone(),
        "priority": 1,
        "should_run_if": [
            "run when OPENDOG project-db size or reclaimable pages suggest retention maintenance"
        ],
        "skip_if": [
            "skip if project database is still small and reclaimable space is negligible"
        ],
        "expected_output_fields": [
            "deleted",
            "storage_before.approx_db_size_bytes",
            "storage_before.approx_reclaimable_bytes",
            "maintenance"
        ],
        "follow_up_on_success": [
            "if deleted counts are meaningful, ask whether to rerun cleanup without dry_run",
            "if reclaimable bytes remain high, consider a follow-up vacuum pass"
        ],
        "follow_up_on_failure": [
            "fallback to project-scoped stats and verification review if cleanup preview is unavailable"
        ],
        "plan_stage": "maintain",
        "terminality": "non_terminal",
        "can_run_in_parallel": false,
        "requires_human_confirmation": false,
        "evidence_written_to_opendog": false,
        "retry_policy": {
            "allowed": true,
            "max_attempts": 2,
            "strategy": "rerun_after_scope_or_retention_parameter_adjustment",
            "retry_when": ["project id or cleanup scope was corrected"]
        }
    })];
    let mut next_priority = 2;

    for recommendation in &context.cleanup_recommendations {
        let scope = recommendation.scope.as_str();
        let (command_template, default_values, success_signal) = match recommendation.scope {
            StorageCleanupScope::Activity | StorageCleanupScope::Verification => {
                let days = recommendation.older_than_days_or_default(&default_policy);
                (
                    format!(
                        "opendog cleanup-data --id {} --scope {} --older-than-days {} --dry-run --json",
                        project_id_value, scope, days
                    ),
                    json!({
                        "scope": scope,
                        "older_than_days": days,
                        "dry_run": true,
                        "json": true
                    }),
                    format!(
                        "{} cleanup preview returns deleted counts plus storage_before metrics",
                        scope
                    ),
                )
            }
            StorageCleanupScope::Snapshots => {
                let keep_snapshot_runs =
                    recommendation.keep_snapshot_runs_or_default(&default_policy);
                (
                    format!(
                        "opendog cleanup-data --id {} --scope snapshots --keep-snapshot-runs {} --dry-run --json",
                        project_id_value, keep_snapshot_runs
                    ),
                    json!({
                        "scope": "snapshots",
                        "keep_snapshot_runs": keep_snapshot_runs,
                        "dry_run": true,
                        "json": true
                    }),
                    "snapshots cleanup preview returns deleted counts plus storage_before metrics"
                        .to_string(),
                )
            }
            StorageCleanupScope::All => continue,
        };

        templates.push(json!({
            "template_id": format!("storage.cleanup.{}.preview", scope),
            "kind": "cli_command",
            "command_template": command_template,
            "preconditions": [
                "project must exist in OPENDOG",
                "use dry-run first before deleting retained OPENDOG evidence"
            ],
            "blocking_conditions": [],
            "success_signal": success_signal,
            "parameter_schema": {
                "id": { "type": "string", "required": true, "source": "project_id" },
                "scope": { "type": "enum", "required": true, "allowed_values": ["activity", "snapshots", "verification"] },
                "older_than_days": { "type": "integer", "required": false },
                "keep_snapshot_runs": { "type": "integer", "required": false },
                "dry_run": { "type": "boolean", "required": false },
                "json": { "type": "boolean", "required": false }
            },
            "default_values": default_values,
            "placeholder_hints": project_placeholder_hint.clone(),
            "priority": next_priority,
            "should_run_if": [
                "run when OPENDOG storage pressure recommendation names this cleanup scope"
            ],
            "skip_if": [
                "skip if the operator wants a single all-scope cleanup preview instead"
            ],
            "expected_output_fields": [
                "deleted",
                "storage_before.approx_db_size_bytes",
                "storage_before.approx_reclaimable_bytes",
                "maintenance"
            ],
            "follow_up_on_success": [
                "if deleted counts are meaningful, ask whether to rerun cleanup without dry_run"
            ],
            "follow_up_on_failure": [
                "fallback to the all-scope cleanup preview to compare retained evidence counts"
            ],
            "plan_stage": "maintain",
            "terminality": "non_terminal",
            "can_run_in_parallel": false,
            "requires_human_confirmation": false,
            "evidence_written_to_opendog": false,
            "retry_policy": {
                "allowed": true,
                "max_attempts": 2,
                "strategy": "rerun_after_scope_or_retention_parameter_adjustment",
                "retry_when": ["project id or cleanup scope was corrected"]
            }
        }));
        next_priority += 1;
    }

    for plan_step in &context.cleanup_plan_steps {
        let scope = plan_step.scope.as_str();
        let (command_template, default_values) = match plan_step.scope {
            StorageCleanupScope::Activity
            | StorageCleanupScope::Verification
            | StorageCleanupScope::All => {
                let days = plan_step.older_than_days_or_default(&default_policy);
                (
                    format!(
                        "opendog cleanup-data --id {} --scope {} --older-than-days {} --json",
                        project_id_value, scope, days
                    ),
                    json!({
                        "scope": scope,
                        "older_than_days": days,
                        "dry_run": false,
                        "json": true
                    }),
                )
            }
            StorageCleanupScope::Snapshots => {
                let keep_snapshot_runs = plan_step.keep_snapshot_runs_or_default(&default_policy);
                (
                    format!(
                        "opendog cleanup-data --id {} --scope snapshots --keep-snapshot-runs {} --json",
                        project_id_value, keep_snapshot_runs
                    ),
                    json!({
                        "scope": "snapshots",
                        "keep_snapshot_runs": keep_snapshot_runs,
                        "dry_run": false,
                        "json": true
                    }),
                )
            }
        };

        templates.push(json!({
            "template_id": format!("storage.cleanup.{}.execute", scope),
            "kind": "cli_command",
            "command_template": command_template,
            "preconditions": [
                "run only after the matching cleanup-data dry-run preview was reviewed",
                "operator must confirm retained OPENDOG evidence can be deleted"
            ],
            "blocking_conditions": [
                "requires explicit confirmation because this command deletes retained OPENDOG evidence"
            ],
            "success_signal": format!("{} cleanup execution returns deleted counts plus storage_after metrics", scope),
            "parameter_schema": {
                "id": { "type": "string", "required": true, "source": "project_id" },
                "scope": { "type": "enum", "required": true, "allowed_values": ["activity", "snapshots", "verification", "all"] },
                "older_than_days": { "type": "integer", "required": false },
                "keep_snapshot_runs": { "type": "integer", "required": false },
                "dry_run": { "type": "boolean", "required": false },
                "json": { "type": "boolean", "required": false }
            },
            "default_values": default_values,
            "placeholder_hints": project_placeholder_hint.clone(),
            "priority": next_priority,
            "should_run_if": [
                "run only when the policy cleanup plan execute_cleanup step is approved"
            ],
            "skip_if": [
                "skip if the dry-run preview was not reviewed or deleted counts look unsafe"
            ],
            "expected_output_fields": [
                "deleted",
                "storage_before.approx_db_size_bytes",
                "storage_after.approx_db_size_bytes",
                "maintenance"
            ],
            "follow_up_on_success": [
                "refresh agent guidance to confirm storage pressure decreased"
            ],
            "follow_up_on_failure": [
                "rerun the matching dry-run preview and adjust retention policy before retrying"
            ],
            "plan_stage": "maintain",
            "terminality": "non_terminal",
            "can_run_in_parallel": false,
            "requires_human_confirmation": true,
            "evidence_written_to_opendog": false,
            "retry_policy": {
                "allowed": true,
                "max_attempts": 1,
                "strategy": "retry_after_preview_review_or_retention_parameter_change",
                "retry_when": ["the operator explicitly approved a corrected cleanup execution"]
            }
        }));
        next_priority += 1;
    }

    if context.vacuum_candidate {
        templates.push(json!({
            "template_id": "storage.cleanup.compact",
            "kind": "cli_command",
            "command_template": format!("opendog cleanup-data --id {} --scope all --older-than-days 30 --vacuum --json", project_id_value),
            "preconditions": [
                "run only after preview confirms retained evidence can be pruned safely",
                "reserve vacuum for explicit space-reclaim passes"
            ],
            "blocking_conditions": [
                "requires explicit confirmation because cleanup plus vacuum rewrites the project database"
            ],
            "success_signal": format!("storage_after shows reduced reclaimable space after reclaiming about {} bytes", reclaimable_bytes),
            "parameter_schema": {
                "id": { "type": "string", "required": true, "source": "project_id" },
                "scope": { "type": "enum", "required": true, "allowed_values": ["all"] },
                "older_than_days": { "type": "integer", "required": false },
                "vacuum": { "type": "boolean", "required": false }
            },
            "default_values": {
                "scope": "all",
                "older_than_days": 30,
                "vacuum": true,
                "json": true
            },
            "placeholder_hints": project_placeholder_hint,
            "priority": next_priority,
            "should_run_if": [
                "run only when reclaimable bytes remain materially high and the user agrees to maintenance"
            ],
            "skip_if": [
                "skip if preview shows little reclaimable space or the user does not want database compaction"
            ],
            "expected_output_fields": [
                "storage_after.approx_db_size_bytes",
                "storage_after.approx_reclaimable_bytes",
                "maintenance.vacuumed"
            ],
            "follow_up_on_success": [
                "refresh agent guidance if storage pressure was the main maintenance concern"
            ],
            "follow_up_on_failure": [
                "rerun the preview without vacuum to inspect counts and maintenance notes again"
            ],
            "plan_stage": "maintain",
            "terminality": "non_terminal",
            "can_run_in_parallel": false,
            "requires_human_confirmation": true,
            "evidence_written_to_opendog": false,
            "retry_policy": {
                "allowed": true,
                "max_attempts": 1,
                "strategy": "retry_after_confirmation_or_parameter_change",
                "retry_when": ["the user explicitly approved the vacuum pass"]
            }
        }));
    }

    templates
}

pub(super) fn augment_entrypoints_for_storage_maintenance(
    entrypoints: &mut Value,
    project_id: Option<&str>,
    storage_maintenance: &Value,
) {
    if !storage_maintenance["maintenance_candidate"]
        .as_bool()
        .unwrap_or(false)
    {
        return;
    }

    let project_id_value = project_id.unwrap_or("<project>");
    if let Some(items) = entrypoints["next_cli_commands"].as_array_mut() {
        items.insert(
            0,
            json!(format!(
                "opendog cleanup-data --id {} --scope all --older-than-days 30 --dry-run --json",
                project_id_value
            )),
        );
    }
    if let Some(items) = entrypoints["selection_reasons"].as_array_mut() {
        items.insert(
            0,
            json!({
                "kind": "cli_command",
                "target": format!(
                    "opendog cleanup-data --id {} --scope all --older-than-days 30 --dry-run --json",
                    project_id_value
                ),
                "why": storage_maintenance["summary"].clone(),
            }),
        );
    }
    if let Some(items) = entrypoints["execution_templates"].as_array_mut() {
        let templates = storage_maintenance_execution_templates(project_id, storage_maintenance);
        for template in templates.into_iter().rev() {
            items.insert(0, template);
        }
        for (index, template) in items.iter_mut().enumerate() {
            template["priority"] = json!(index + 1);
        }
    }
}

#[cfg(test)]
mod planner_tests;

#[cfg(test)]
mod tests;
