use serde_json::{json, Value};

use crate::core::retention::StorageMetrics;

const STORAGE_CLEANUP_REVIEW_DB_BYTES_THRESHOLD: i64 = 16 * 1024 * 1024;
const STORAGE_VACUUM_RECLAIMABLE_BYTES_THRESHOLD: i64 = 8 * 1024 * 1024;
const STORAGE_VACUUM_RECLAIM_RATIO_THRESHOLD: f64 = 0.20;

fn storage_reclaim_ratio(metrics: &StorageMetrics) -> f64 {
    if metrics.approx_db_size_bytes <= 0 {
        0.0
    } else {
        metrics.approx_reclaimable_bytes as f64 / metrics.approx_db_size_bytes as f64
    }
}

pub(super) fn project_storage_maintenance(metrics: Option<&StorageMetrics>) -> Value {
    let Some(metrics) = metrics else {
        return json!({
            "status": "unavailable",
            "cleanup_review_candidate": false,
            "maintenance_candidate": false,
            "vacuum_candidate": false,
            "suggested_mode": "none",
            "summary": "Project database metrics are unavailable.",
        });
    };

    let reclaim_ratio = storage_reclaim_ratio(metrics);
    let cleanup_review_candidate =
        metrics.approx_db_size_bytes >= STORAGE_CLEANUP_REVIEW_DB_BYTES_THRESHOLD;
    let vacuum_candidate = metrics.approx_reclaimable_bytes
        >= STORAGE_VACUUM_RECLAIMABLE_BYTES_THRESHOLD
        && reclaim_ratio >= STORAGE_VACUUM_RECLAIM_RATIO_THRESHOLD;
    let maintenance_candidate = cleanup_review_candidate || vacuum_candidate;
    let suggested_mode = if vacuum_candidate {
        "review_cleanup_then_vacuum"
    } else if cleanup_review_candidate {
        "review_cleanup"
    } else {
        "none"
    };
    let summary = if vacuum_candidate {
        "Project database has reclaimable space; review retained OPENDOG evidence and consider vacuum after cleanup."
    } else if cleanup_review_candidate {
        "Project database is large enough that retained OPENDOG evidence should be reviewed with cleanup-data dry-run."
    } else {
        "Project database size does not currently suggest dedicated OPENDOG retention maintenance."
    };

    json!({
        "status": "available",
        "page_count": metrics.page_count,
        "freelist_count": metrics.freelist_count,
        "approx_db_size_bytes": metrics.approx_db_size_bytes,
        "approx_reclaimable_bytes": metrics.approx_reclaimable_bytes,
        "reclaim_ratio": reclaim_ratio,
        "cleanup_review_candidate": cleanup_review_candidate,
        "maintenance_candidate": maintenance_candidate,
        "vacuum_candidate": vacuum_candidate,
        "suggested_mode": suggested_mode,
        "summary": summary,
    })
}

pub(super) fn storage_maintenance_layer(project_overviews: &[Value]) -> Value {
    let projects_with_candidates = project_overviews
        .iter()
        .filter(|p| {
            p["storage_maintenance"]["maintenance_candidate"]
                .as_bool()
                .unwrap_or(false)
        })
        .count();
    let projects_with_vacuum_candidates = project_overviews
        .iter()
        .filter(|p| {
            p["storage_maintenance"]["vacuum_candidate"]
                .as_bool()
                .unwrap_or(false)
        })
        .count();
    let total_approx_db_size_bytes = project_overviews
        .iter()
        .map(|p| {
            p["storage_maintenance"]["approx_db_size_bytes"]
                .as_i64()
                .unwrap_or(0)
        })
        .sum::<i64>();
    let total_approx_reclaimable_bytes = project_overviews
        .iter()
        .map(|p| {
            p["storage_maintenance"]["approx_reclaimable_bytes"]
                .as_i64()
                .unwrap_or(0)
        })
        .sum::<i64>();

    let mut priority_projects: Vec<Value> = project_overviews
        .iter()
        .filter(|p| {
            p["storage_maintenance"]["maintenance_candidate"]
                .as_bool()
                .unwrap_or(false)
        })
        .map(|p| {
            json!({
                "project_id": p["project_id"].clone(),
                "status": p["status"].clone(),
                "vacuum_candidate": p["storage_maintenance"]["vacuum_candidate"].clone(),
                "cleanup_review_candidate": p["storage_maintenance"]["cleanup_review_candidate"].clone(),
                "approx_db_size_bytes": p["storage_maintenance"]["approx_db_size_bytes"].clone(),
                "approx_reclaimable_bytes": p["storage_maintenance"]["approx_reclaimable_bytes"].clone(),
                "reclaim_ratio": p["storage_maintenance"]["reclaim_ratio"].clone(),
                "suggested_mode": p["storage_maintenance"]["suggested_mode"].clone(),
                "summary": p["storage_maintenance"]["summary"].clone(),
            })
        })
        .collect();
    priority_projects.sort_by(|a, b| {
        b["vacuum_candidate"]
            .as_bool()
            .unwrap_or(false)
            .cmp(&a["vacuum_candidate"].as_bool().unwrap_or(false))
            .then_with(|| {
                b["approx_reclaimable_bytes"]
                    .as_i64()
                    .unwrap_or(0)
                    .cmp(&a["approx_reclaimable_bytes"].as_i64().unwrap_or(0))
            })
            .then_with(|| {
                b["approx_db_size_bytes"]
                    .as_i64()
                    .unwrap_or(0)
                    .cmp(&a["approx_db_size_bytes"].as_i64().unwrap_or(0))
            })
    });
    priority_projects.truncate(5);

    json!({
        "status": "available",
        "projects_with_candidates": projects_with_candidates,
        "projects_with_vacuum_candidates": projects_with_vacuum_candidates,
        "total_approx_db_size_bytes": total_approx_db_size_bytes,
        "total_approx_reclaimable_bytes": total_approx_reclaimable_bytes,
        "priority_projects": priority_projects,
    })
}

fn storage_maintenance_execution_templates(
    project_id: Option<&str>,
    storage_maintenance: &Value,
) -> Vec<Value> {
    if !storage_maintenance["maintenance_candidate"]
        .as_bool()
        .unwrap_or(false)
    {
        return Vec::new();
    }

    let project_id_value = project_id.unwrap_or("<project>");
    let project_placeholder_hint = if project_id.is_none() {
        json!([{
            "field": "id",
            "placeholder": "<project>",
            "description": "replace with a registered OPENDOG project id"
        }])
    } else {
        json!([])
    };
    let reclaimable_bytes = storage_maintenance["approx_reclaimable_bytes"]
        .as_i64()
        .unwrap_or(0);
    let mut templates = vec![json!({
        "template_id": "storage.cleanup.preview",
        "kind": "mcp_tool",
        "tool": "cleanup_project_data",
        "args_template": {
            "id": project_id_value,
            "scope": "all",
            "older_than_days": 30,
            "dry_run": true
        },
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
            "dry_run": { "type": "boolean", "required": false }
        },
        "default_values": {
            "scope": "all",
            "older_than_days": 30,
            "dry_run": true
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

    if storage_maintenance["vacuum_candidate"]
        .as_bool()
        .unwrap_or(false)
    {
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
            "priority": 2,
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
    if let Some(items) = entrypoints["next_mcp_tools"].as_array_mut() {
        items.insert(0, json!("cleanup_project_data"));
    }
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
                "kind": "mcp_tool",
                "target": "cleanup_project_data",
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
