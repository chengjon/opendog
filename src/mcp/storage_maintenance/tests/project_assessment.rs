use super::*;

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
