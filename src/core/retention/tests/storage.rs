use super::*;

#[test]
fn collect_storage_metrics_returns_valid_page_info() {
    let db = test_db();
    let metrics = collect_storage_metrics(&db).unwrap();
    assert!(metrics.page_size > 0);
    assert!(metrics.page_count >= 1);
    assert!(metrics.approx_db_size_bytes >= metrics.page_size);
    // Fresh DB should have minimal reclaimable space
    assert!(metrics.approx_reclaimable_bytes >= 0);
}

#[test]
fn collect_storage_metrics_size_equals_page_size_times_page_count() {
    let db = test_db();
    let metrics = collect_storage_metrics(&db).unwrap();
    assert_eq!(
        metrics.approx_db_size_bytes,
        metrics.page_size.saturating_mul(metrics.page_count)
    );
}

#[test]
fn collect_storage_metrics_reclaimable_equals_page_size_times_freelist() {
    let db = test_db();
    let metrics = collect_storage_metrics(&db).unwrap();
    assert_eq!(
        metrics.approx_reclaimable_bytes,
        metrics.page_size.saturating_mul(metrics.freelist_count)
    );
}
