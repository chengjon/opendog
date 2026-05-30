use super::*;
use crate::config::RetentionPolicy;
use crate::core::retention::{StorageEvidenceCounts, StorageMetrics};

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
    let result = project_storage_maintenance_with_policy(None, None, &RetentionPolicy::default());
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
    let result =
        project_storage_maintenance_with_policy(Some(&metrics), None, &RetentionPolicy::default());
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
    let result =
        project_storage_maintenance_with_policy(Some(&metrics), None, &RetentionPolicy::default());
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
    let result =
        project_storage_maintenance_with_policy(Some(&metrics), None, &RetentionPolicy::default());
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
    let result =
        project_storage_maintenance_with_policy(Some(&metrics), None, &RetentionPolicy::default());
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
    let result =
        project_storage_maintenance_with_policy(Some(&metrics), None, &RetentionPolicy::default());
    assert_eq!(result["page_count"], 500);
    assert_eq!(result["freelist_count"], 50);
    assert_eq!(result["approx_db_size_bytes"], 2_048_000);
    assert_eq!(result["approx_reclaimable_bytes"], 204_800);
}

#[test]
fn project_storage_maintenance_activity_pressure_recommends_activity_cleanup() {
    let metrics = StorageMetrics {
        page_size: 4096,
        page_count: 100,
        freelist_count: 5,
        approx_db_size_bytes: 409_600,
        approx_reclaimable_bytes: 20_480,
    };
    let counts = StorageEvidenceCounts {
        file_sightings: 1_200_000,
        file_events: 20,
        activity_daily_rollups: 0,
        verification_runs: 0,
        snapshot_runs: 0,
    };
    let result = project_storage_maintenance_with_policy(
        Some(&metrics),
        Some(&counts),
        &RetentionPolicy::default(),
    );
    assert_eq!(result["cleanup_review_candidate"], false);
    assert_eq!(result["evidence_pressure_candidate"], true);
    assert_eq!(result["maintenance_candidate"], true);
    assert_eq!(result["suggested_mode"], "review_cleanup");
    assert_eq!(result["pressure_level"], "high");
    assert_eq!(
        result["cleanup_recommendations"][0]["scope"],
        json!("activity")
    );
    assert_eq!(
        result["cleanup_recommendations"][0]["older_than_days"],
        json!(30)
    );
}

#[test]
fn project_storage_maintenance_snapshot_pressure_recommends_snapshot_cleanup() {
    let metrics = StorageMetrics {
        page_size: 4096,
        page_count: 100,
        freelist_count: 5,
        approx_db_size_bytes: 409_600,
        approx_reclaimable_bytes: 20_480,
    };
    let counts = StorageEvidenceCounts {
        file_sightings: 0,
        file_events: 0,
        activity_daily_rollups: 0,
        verification_runs: 0,
        snapshot_runs: 250,
    };
    let result = project_storage_maintenance_with_policy(
        Some(&metrics),
        Some(&counts),
        &RetentionPolicy::default(),
    );
    assert_eq!(result["evidence_pressure_candidate"], true);
    assert_eq!(
        result["cleanup_recommendations"][0]["scope"],
        json!("snapshots")
    );
    assert_eq!(
        result["cleanup_recommendations"][0]["keep_snapshot_runs"],
        json!(20)
    );
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
    assert!(hints[0]["description"]
        .as_str()
        .unwrap()
        .contains("project id"));
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
fn execution_templates_include_scope_specific_cleanup_recommendations() {
    let sm = json!({
        "maintenance_candidate": true,
        "vacuum_candidate": false,
        "approx_reclaimable_bytes": 0,
        "cleanup_recommendations": [
            {"scope": "activity", "older_than_days": 30},
            {"scope": "snapshots", "keep_snapshot_runs": 20}
        ],
    });
    let result = storage_maintenance_execution_templates(Some("proj"), &sm);
    let commands: Vec<&str> = result
        .iter()
        .filter_map(|template| template["command_template"].as_str())
        .collect();
    assert!(commands.iter().any(|cmd| cmd.contains(
        "opendog cleanup-data --id proj --scope activity --older-than-days 30 --dry-run --json"
    )));
    assert!(commands.iter().any(|cmd| cmd.contains(
        "opendog cleanup-data --id proj --scope snapshots --keep-snapshot-runs 20 --dry-run --json"
    )));
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

#[test]
fn execution_templates_keep_stable_catalog_order_and_priorities() {
    let sm = json!({
        "maintenance_candidate": true,
        "vacuum_candidate": true,
        "approx_reclaimable_bytes": 10_000_000,
        "cleanup_recommendations": [
            {"scope": "activity", "older_than_days": 30},
            {"scope": "snapshots", "keep_snapshot_runs": 20}
        ],
        "cleanup_plan": {
            "steps": [
                {"phase": "execute_cleanup", "scope": "activity", "older_than_days": 30},
                {"phase": "execute_cleanup", "scope": "snapshots", "keep_snapshot_runs": 20}
            ]
        }
    });

    let result = storage_maintenance_execution_templates(Some("proj"), &sm);
    let ids = result
        .iter()
        .map(|template| template["template_id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![
            "storage.cleanup.preview",
            "storage.cleanup.activity.preview",
            "storage.cleanup.snapshots.preview",
            "storage.cleanup.activity.execute",
            "storage.cleanup.snapshots.execute",
            "storage.cleanup.compact",
        ]
    );

    let priorities = result
        .iter()
        .map(|template| template["priority"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(priorities, vec![1, 2, 3, 4, 5, 6]);
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
    assert!(entrypoints
        .get("next_cli_commands")
        .is_none_or(|v| v.is_null()));
}
