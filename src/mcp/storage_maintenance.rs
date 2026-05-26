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
mod tests {
    use super::*;
    use crate::core::retention::StorageMetrics;

    // --- storage_reclaim_ratio tests ---

    #[test]
    fn reclaim_ratio_normal_case() {
        let metrics = StorageMetrics {
            page_size: 4096,
            page_count: 1000,
            freelist_count: 200,
            approx_db_size_bytes: 4_096_000,
            approx_reclaimable_bytes: 819_200,
        };
        let ratio = storage_reclaim_ratio(&metrics);
        assert!((ratio - 0.2).abs() < 0.001);
    }

    #[test]
    fn reclaim_ratio_zero_db_size() {
        let metrics = StorageMetrics {
            page_size: 4096,
            page_count: 0,
            freelist_count: 0,
            approx_db_size_bytes: 0,
            approx_reclaimable_bytes: 0,
        };
        let ratio = storage_reclaim_ratio(&metrics);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn reclaim_ratio_negative_db_size() {
        let metrics = StorageMetrics {
            page_size: 4096,
            page_count: 0,
            freelist_count: 0,
            approx_db_size_bytes: -1,
            approx_reclaimable_bytes: 100,
        };
        let ratio = storage_reclaim_ratio(&metrics);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn reclaim_ratio_no_reclaimable() {
        let metrics = StorageMetrics {
            page_size: 4096,
            page_count: 100,
            freelist_count: 0,
            approx_db_size_bytes: 409_600,
            approx_reclaimable_bytes: 0,
        };
        let ratio = storage_reclaim_ratio(&metrics);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn reclaim_ratio_full_reclaim() {
        let metrics = StorageMetrics {
            page_size: 4096,
            page_count: 100,
            freelist_count: 100,
            approx_db_size_bytes: 409_600,
            approx_reclaimable_bytes: 409_600,
        };
        let ratio = storage_reclaim_ratio(&metrics);
        assert!((ratio - 1.0).abs() < 0.001);
    }

    // --- project_storage_maintenance tests ---

    #[test]
    fn project_storage_maintenance_none_metrics() {
        let result = project_storage_maintenance(None);
        assert_eq!(result["status"], "unavailable");
        assert_eq!(result["cleanup_review_candidate"], false);
        assert_eq!(result["maintenance_candidate"], false);
        assert_eq!(result["vacuum_candidate"], false);
        assert_eq!(result["suggested_mode"], "none");
    }

    #[test]
    fn project_storage_maintenance_small_db_under_threshold() {
        let metrics = StorageMetrics {
            page_size: 4096,
            page_count: 100,
            freelist_count: 5,
            approx_db_size_bytes: 409_600,
            approx_reclaimable_bytes: 20_480,
        };
        let result = project_storage_maintenance(Some(&metrics));
        assert_eq!(result["status"], "available");
        assert_eq!(result["cleanup_review_candidate"], false);
        assert_eq!(result["maintenance_candidate"], false);
        assert_eq!(result["vacuum_candidate"], false);
        assert_eq!(result["suggested_mode"], "none");
    }

    #[test]
    fn project_storage_maintenance_large_db_over_threshold() {
        let metrics = StorageMetrics {
            page_size: 4096,
            page_count: 10000,
            freelist_count: 500,
            approx_db_size_bytes: 40_960_000,
            approx_reclaimable_bytes: 2_048_000,
        };
        let result = project_storage_maintenance(Some(&metrics));
        assert_eq!(result["cleanup_review_candidate"], true);
        assert_eq!(result["maintenance_candidate"], true);
        assert_eq!(result["vacuum_candidate"], false);
        assert_eq!(result["suggested_mode"], "review_cleanup");
    }

    #[test]
    fn project_storage_maintenance_vacuum_candidate() {
        // Thresholds: db >= 16MB, reclaimable >= 8MB, ratio >= 0.20
        let metrics = StorageMetrics {
            page_size: 4096,
            page_count: 10000,
            freelist_count: 5000,
            approx_db_size_bytes: 40_960_000,
            approx_reclaimable_bytes: 20_480_000,
        };
        let result = project_storage_maintenance(Some(&metrics));
        assert_eq!(result["cleanup_review_candidate"], true);
        assert_eq!(result["maintenance_candidate"], true);
        assert_eq!(result["vacuum_candidate"], true);
        assert_eq!(result["suggested_mode"], "review_cleanup_then_vacuum");
    }

    #[test]
    fn project_storage_maintenance_large_db_but_small_reclaimable() {
        let metrics = StorageMetrics {
            page_size: 4096,
            page_count: 10000,
            freelist_count: 10,
            approx_db_size_bytes: 40_960_000,
            approx_reclaimable_bytes: 40_960,
        };
        let result = project_storage_maintenance(Some(&metrics));
        assert_eq!(result["cleanup_review_candidate"], true);
        assert_eq!(result["maintenance_candidate"], true);
        // Reclaimable is under 8MB threshold
        assert_eq!(result["vacuum_candidate"], false);
    }

    #[test]
    fn project_storage_maintenance_reflects_fields() {
        let metrics = StorageMetrics {
            page_size: 4096,
            page_count: 500,
            freelist_count: 50,
            approx_db_size_bytes: 2_048_000,
            approx_reclaimable_bytes: 204_800,
        };
        let result = project_storage_maintenance(Some(&metrics));
        assert_eq!(result["page_count"], 500);
        assert_eq!(result["freelist_count"], 50);
        assert_eq!(result["approx_db_size_bytes"], 2_048_000);
        assert_eq!(result["approx_reclaimable_bytes"], 204_800);
    }

    // --- storage_maintenance_layer tests ---

    #[test]
    fn storage_maintenance_layer_empty_projects() {
        let result = storage_maintenance_layer(&[]);
        assert_eq!(result["status"], "available");
        assert_eq!(result["projects_with_candidates"], 0);
        assert_eq!(result["projects_with_vacuum_candidates"], 0);
        assert_eq!(result["total_approx_db_size_bytes"], 0);
        assert_eq!(result["total_approx_reclaimable_bytes"], 0);
        assert!(result["priority_projects"].as_array().unwrap().is_empty());
    }

    #[test]
    fn storage_maintenance_layer_aggregates_sizes() {
        let projects = vec![
            json!({
                "project_id": "proj_a",
                "status": "available",
                "storage_maintenance": {
                    "maintenance_candidate": false,
                    "vacuum_candidate": false,
                    "cleanup_review_candidate": false,
                    "approx_db_size_bytes": 1_000_000,
                    "approx_reclaimable_bytes": 100_000,
                    "reclaim_ratio": 0.1,
                    "suggested_mode": "none",
                    "summary": "small",
                }
            }),
            json!({
                "project_id": "proj_b",
                "status": "available",
                "storage_maintenance": {
                    "maintenance_candidate": false,
                    "vacuum_candidate": false,
                    "cleanup_review_candidate": false,
                    "approx_db_size_bytes": 2_000_000,
                    "approx_reclaimable_bytes": 200_000,
                    "reclaim_ratio": 0.1,
                    "suggested_mode": "none",
                    "summary": "small",
                }
            }),
        ];
        let result = storage_maintenance_layer(&projects);
        assert_eq!(result["total_approx_db_size_bytes"], 3_000_000);
        assert_eq!(result["total_approx_reclaimable_bytes"], 300_000);
        assert_eq!(result["projects_with_candidates"], 0);
    }

    #[test]
    fn storage_maintenance_layer_filters_candidates() {
        let projects = vec![
            json!({
                "project_id": "clean_project",
                "status": "available",
                "storage_maintenance": {
                    "maintenance_candidate": false,
                    "vacuum_candidate": false,
                    "cleanup_review_candidate": false,
                    "approx_db_size_bytes": 500_000,
                    "approx_reclaimable_bytes": 0,
                    "reclaim_ratio": 0.0,
                    "suggested_mode": "none",
                    "summary": "no issues",
                }
            }),
            json!({
                "project_id": "dirty_project",
                "status": "available",
                "storage_maintenance": {
                    "maintenance_candidate": true,
                    "vacuum_candidate": true,
                    "cleanup_review_candidate": true,
                    "approx_db_size_bytes": 50_000_000,
                    "approx_reclaimable_bytes": 25_000_000,
                    "reclaim_ratio": 0.5,
                    "suggested_mode": "review_cleanup_then_vacuum",
                    "summary": "needs cleanup",
                }
            }),
        ];
        let result = storage_maintenance_layer(&projects);
        assert_eq!(result["projects_with_candidates"], 1);
        assert_eq!(result["projects_with_vacuum_candidates"], 1);
        let priority = result["priority_projects"].as_array().unwrap();
        assert_eq!(priority.len(), 1);
        assert_eq!(priority[0]["project_id"], "dirty_project");
    }

    #[test]
    fn storage_maintenance_layer_truncates_to_five() {
        let mut projects = Vec::new();
        for i in 0..8 {
            projects.push(json!({
                "project_id": format!("proj_{}", i),
                "status": "available",
                "storage_maintenance": {
                    "maintenance_candidate": true,
                    "vacuum_candidate": i % 2 == 0,
                    "cleanup_review_candidate": true,
                    "approx_db_size_bytes": 30_000_000 + i as i64 * 1_000_000,
                    "approx_reclaimable_bytes": 15_000_000 + i as i64 * 500_000,
                    "reclaim_ratio": 0.5,
                    "suggested_mode": "review_cleanup_then_vacuum",
                    "summary": "needs attention",
                }
            }));
        }
        let result = storage_maintenance_layer(&projects);
        let priority = result["priority_projects"].as_array().unwrap();
        assert_eq!(priority.len(), 5);
    }

    #[test]
    fn storage_maintenance_layer_sorts_vacuum_candidates_first() {
        let projects = vec![
            json!({
                "project_id": "no_vacuum",
                "status": "available",
                "storage_maintenance": {
                    "maintenance_candidate": true,
                    "vacuum_candidate": false,
                    "cleanup_review_candidate": true,
                    "approx_db_size_bytes": 30_000_000,
                    "approx_reclaimable_bytes": 2_000_000,
                    "reclaim_ratio": 0.07,
                    "suggested_mode": "review_cleanup",
                    "summary": "cleanup only",
                }
            }),
            json!({
                "project_id": "with_vacuum",
                "status": "available",
                "storage_maintenance": {
                    "maintenance_candidate": true,
                    "vacuum_candidate": true,
                    "cleanup_review_candidate": true,
                    "approx_db_size_bytes": 30_000_000,
                    "approx_reclaimable_bytes": 2_000_000,
                    "reclaim_ratio": 0.07,
                    "suggested_mode": "review_cleanup_then_vacuum",
                    "summary": "vacuum needed",
                }
            }),
        ];
        let result = storage_maintenance_layer(&projects);
        let priority = result["priority_projects"].as_array().unwrap();
        assert_eq!(priority.len(), 2);
        // Vacuum candidate should be first
        assert_eq!(priority[0]["project_id"], "with_vacuum");
        assert_eq!(priority[1]["project_id"], "no_vacuum");
    }

    // --- storage_maintenance_execution_templates tests ---

    #[test]
    fn execution_templates_returns_empty_for_non_maintenance() {
        let sm = json!({
            "maintenance_candidate": false,
            "vacuum_candidate": false,
            "approx_reclaimable_bytes": 0,
        });
        let result = storage_maintenance_execution_templates(Some("proj"), &sm);
        assert!(result.is_empty());
    }

    #[test]
    fn execution_templates_returns_empty_for_missing_field() {
        let sm = json!({});
        let result = storage_maintenance_execution_templates(Some("proj"), &sm);
        assert!(result.is_empty());
    }

    #[test]
    fn execution_templates_returns_preview_template_with_project_id() {
        let sm = json!({
            "maintenance_candidate": true,
            "vacuum_candidate": false,
            "approx_reclaimable_bytes": 0,
        });
        let result = storage_maintenance_execution_templates(Some("myproject"), &sm);
        assert_eq!(result.len(), 1);
        let tpl = &result[0];
        assert_eq!(tpl["template_id"], "storage.cleanup.preview");
        let cmd = tpl["command_template"].as_str().unwrap();
        assert!(cmd.contains("myproject"));
        let hints = tpl["placeholder_hints"].as_array().unwrap();
        assert!(hints.is_empty());
    }

    #[test]
    fn execution_templates_returns_preview_template_with_placeholder() {
        let sm = json!({
            "maintenance_candidate": true,
            "vacuum_candidate": false,
            "approx_reclaimable_bytes": 0,
        });
        let result = storage_maintenance_execution_templates(None, &sm);
        assert_eq!(result.len(), 1);
        let hints = result[0]["placeholder_hints"].as_array().unwrap();
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0]["field"], "id");
        assert_eq!(hints[0]["placeholder"], "<project>");
        assert!(hints[0]["description"].as_str().unwrap().contains("project id"));
    }

    #[test]
    fn execution_templates_returns_compact_template_for_vacuum_candidate() {
        let sm = json!({
            "maintenance_candidate": true,
            "vacuum_candidate": true,
            "approx_reclaimable_bytes": 10_000_000,
        });
        let result = storage_maintenance_execution_templates(Some("proj"), &sm);
        assert_eq!(result.len(), 2);
        assert_eq!(result[1]["template_id"], "storage.cleanup.compact");
        assert_eq!(result[1]["requires_human_confirmation"], true);
    }

    #[test]
    fn execution_templates_compact_includes_reclaimable_bytes_in_success_signal() {
        let sm = json!({
            "maintenance_candidate": true,
            "vacuum_candidate": true,
            "approx_reclaimable_bytes": 5000,
        });
        let result = storage_maintenance_execution_templates(Some("proj"), &sm);
        assert_eq!(result.len(), 2);
        let signal = result[1]["success_signal"].as_str().unwrap();
        assert!(signal.contains("5000"));
    }

    #[test]
    fn execution_templates_preview_has_priority_1() {
        let sm = json!({
            "maintenance_candidate": true,
            "vacuum_candidate": false,
            "approx_reclaimable_bytes": 0,
        });
        let result = storage_maintenance_execution_templates(Some("proj"), &sm);
        assert_eq!(result[0]["priority"], 1);
    }

    #[test]
    fn execution_templates_compact_has_priority_2() {
        let sm = json!({
            "maintenance_candidate": true,
            "vacuum_candidate": true,
            "approx_reclaimable_bytes": 10_000_000,
        });
        let result = storage_maintenance_execution_templates(Some("proj"), &sm);
        assert_eq!(result[0]["priority"], 1);
        assert_eq!(result[1]["priority"], 2);
    }

    // --- augment_entrypoints_for_storage_maintenance tests ---

    #[test]
    fn augment_no_op_for_non_maintenance() {
        let mut entrypoints = json!({
            "next_cli_commands": ["original_cmd"],
            "selection_reasons": [{"kind": "other", "why": "original"}],
            "execution_templates": [{"template_id": "existing", "priority": 1}],
        });
        let sm = json!({"maintenance_candidate": false});
        augment_entrypoints_for_storage_maintenance(&mut entrypoints, Some("proj"), &sm);
        assert_eq!(entrypoints["next_cli_commands"][0], "original_cmd");
        assert_eq!(entrypoints["selection_reasons"][0]["why"], "original");
    }

    #[test]
    fn augment_prepends_cleanup_command() {
        let mut entrypoints = json!({
            "next_cli_commands": ["existing_cmd"],
            "selection_reasons": [],
            "execution_templates": [],
        });
        let sm = json!({
            "maintenance_candidate": true,
            "vacuum_candidate": false,
            "approx_reclaimable_bytes": 0,
            "summary": "needs cleanup",
        });
        augment_entrypoints_for_storage_maintenance(&mut entrypoints, Some("myproj"), &sm);
        let cmds = entrypoints["next_cli_commands"].as_array().unwrap();
        assert!(cmds[0].as_str().unwrap().contains("myproj"));
        assert!(cmds[0].as_str().unwrap().contains("cleanup-data"));
    }

    #[test]
    fn augment_prepends_selection_reason() {
        let mut entrypoints = json!({
            "next_cli_commands": [],
            "selection_reasons": [],
            "execution_templates": [],
        });
        let sm = json!({
            "maintenance_candidate": true,
            "vacuum_candidate": false,
            "approx_reclaimable_bytes": 0,
            "summary": "database is large",
        });
        augment_entrypoints_for_storage_maintenance(&mut entrypoints, Some("proj"), &sm);
        let reasons = entrypoints["selection_reasons"].as_array().unwrap();
        assert_eq!(reasons[0]["why"], "database is large");
    }

    #[test]
    fn augment_prepends_execution_templates_and_renumbers() {
        let mut entrypoints = json!({
            "next_cli_commands": [],
            "selection_reasons": [],
            "execution_templates": [{"template_id": "existing", "priority": 99}],
        });
        let sm = json!({
            "maintenance_candidate": true,
            "vacuum_candidate": true,
            "approx_reclaimable_bytes": 10_000_000,
            "summary": "needs vacuum",
        });
        augment_entrypoints_for_storage_maintenance(&mut entrypoints, Some("proj"), &sm);
        let templates = entrypoints["execution_templates"].as_array().unwrap();
        assert_eq!(templates.len(), 3);
        // Reverse insertion: compact inserted at 0, then preview inserted at 0, pushing compact to 1
        assert_eq!(templates[0]["template_id"], "storage.cleanup.preview");
        assert_eq!(templates[1]["template_id"], "storage.cleanup.compact");
        assert_eq!(templates[2]["template_id"], "existing");
        // All priorities renumbered sequentially
        assert_eq!(templates[0]["priority"], 1);
        assert_eq!(templates[1]["priority"], 2);
        assert_eq!(templates[2]["priority"], 3);
    }

    #[test]
    fn augment_uses_project_placeholder_when_none() {
        let mut entrypoints = json!({
            "next_cli_commands": [],
            "selection_reasons": [],
            "execution_templates": [],
        });
        let sm = json!({
            "maintenance_candidate": true,
            "vacuum_candidate": false,
            "approx_reclaimable_bytes": 0,
            "summary": "needs cleanup",
        });
        augment_entrypoints_for_storage_maintenance(&mut entrypoints, None, &sm);
        let cmds = entrypoints["next_cli_commands"].as_array().unwrap();
        assert!(cmds[0].as_str().unwrap().contains("<project>"));
    }

    #[test]
    fn augment_handles_missing_arrays_gracefully() {
        let mut entrypoints = json!({
            "unrelated_key": "preserved",
        });
        let sm = json!({
            "maintenance_candidate": true,
            "vacuum_candidate": true,
            "approx_reclaimable_bytes": 10_000_000,
            "summary": "needs work",
        });
        // Should not panic and should not remove existing keys
        augment_entrypoints_for_storage_maintenance(&mut entrypoints, Some("proj"), &sm);
        assert_eq!(entrypoints["unrelated_key"], "preserved");
        // No next_cli_commands, selection_reasons, or execution_templates arrays created
        assert!(entrypoints.get("next_cli_commands").is_none_or(|v| v.is_null()));
    }
}
