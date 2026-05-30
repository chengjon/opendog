use super::*;

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
