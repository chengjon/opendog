use super::*;

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
