use super::*;
use crate::config::RetentionPolicy;
use crate::core::retention::{StorageEvidenceCounts, StorageMetrics};
use serde_json::json;

#[test]
fn project_storage_maintenance_uses_configured_retention_policy() {
    let metrics = StorageMetrics {
        page_size: 4096,
        page_count: 100,
        freelist_count: 5,
        approx_db_size_bytes: 409_600,
        approx_reclaimable_bytes: 20_480,
    };
    let counts = StorageEvidenceCounts {
        file_sightings: 50,
        file_events: 0,
        activity_daily_rollups: 0,
        verification_runs: 0,
        snapshot_runs: 12,
    };
    let policy = RetentionPolicy {
        activity_rows_threshold: 10,
        snapshot_runs_threshold: 10,
        activity_retention_days: 7,
        keep_snapshot_runs: 3,
        ..Default::default()
    };

    let result = project_storage_maintenance_with_policy(Some(&metrics), Some(&counts), &policy);

    assert_eq!(result["evidence_pressure_candidate"], true);
    assert_eq!(
        result["cleanup_recommendations"][0]["older_than_days"],
        json!(7)
    );
    assert_eq!(
        result["cleanup_recommendations"][1]["keep_snapshot_runs"],
        json!(3)
    );
    assert_eq!(
        result["retention_policy"]["activity_rows_threshold"],
        json!(10)
    );
}

#[test]
fn project_storage_maintenance_builds_policy_cleanup_plan() {
    let metrics = StorageMetrics {
        page_size: 4096,
        page_count: 100,
        freelist_count: 5,
        approx_db_size_bytes: 409_600,
        approx_reclaimable_bytes: 20_480,
    };
    let counts = StorageEvidenceCounts {
        file_sightings: 50,
        file_events: 0,
        activity_daily_rollups: 0,
        verification_runs: 0,
        snapshot_runs: 12,
    };
    let policy = RetentionPolicy {
        activity_rows_threshold: 10,
        snapshot_runs_threshold: 10,
        activity_retention_days: 7,
        keep_snapshot_runs: 3,
        ..Default::default()
    };

    let result = project_storage_maintenance_with_policy(Some(&metrics), Some(&counts), &policy);
    let plan = &result["cleanup_plan"];
    assert_eq!(plan["status"], "actionable");
    assert_eq!(plan["automatic_deletion"], false);
    assert_eq!(plan["requires_human_confirmation"], true);
    assert_eq!(plan["target_scopes"], json!(["activity", "snapshots"]));
    assert_eq!(plan["steps"][0]["phase"], "preview");
    assert_eq!(plan["steps"][0]["scope"], "activity");
    assert_eq!(plan["steps"][0]["dry_run"], true);
    assert_eq!(plan["steps"][0]["rollup_before_delete"], true);
    assert_eq!(
        plan["steps"][0]["preserved_rollup_table"],
        "activity_daily_rollups"
    );
    assert_eq!(plan["steps"][2]["phase"], "review");
    assert_eq!(plan["steps"][3]["phase"], "execute_cleanup");
    assert_eq!(plan["steps"][3]["requires_human_confirmation"], true);
    assert_eq!(plan["steps"][3]["rollup_granularity"], "daily");
}

#[test]
fn execution_templates_include_confirmed_execute_steps_from_cleanup_plan() {
    let sm = json!({
        "maintenance_candidate": true,
        "vacuum_candidate": false,
        "approx_reclaimable_bytes": 0,
        "cleanup_plan": {
            "status": "actionable",
            "steps": [
                {"phase": "execute_cleanup", "scope": "activity", "older_than_days": 30},
                {"phase": "execute_cleanup", "scope": "snapshots", "keep_snapshot_runs": 20}
            ]
        }
    });
    let result = storage_maintenance_execution_templates(Some("proj"), &sm);
    let activity_execute = result
        .iter()
        .find(|template| template["template_id"] == "storage.cleanup.activity.execute")
        .expect("activity execute template should exist");
    let snapshots_execute = result
        .iter()
        .find(|template| template["template_id"] == "storage.cleanup.snapshots.execute")
        .expect("snapshots execute template should exist");

    assert_eq!(activity_execute["requires_human_confirmation"], true);
    assert!(activity_execute["command_template"]
        .as_str()
        .unwrap()
        .contains("opendog cleanup-data --id proj --scope activity --older-than-days 30 --json"));
    assert!(!activity_execute["command_template"]
        .as_str()
        .unwrap()
        .contains("--dry-run"));
    assert!(snapshots_execute["command_template"]
        .as_str()
        .unwrap()
        .contains(
            "opendog cleanup-data --id proj --scope snapshots --keep-snapshot-runs 20 --json"
        ));
}
