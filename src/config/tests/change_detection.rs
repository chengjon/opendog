use super::*;

#[test]
fn changed_config_fields_reports_only_real_differences() {
    let before = ProjectConfig::default();
    let after = ProjectConfig {
        ignore_patterns: vec!["logs".to_string()],
        process_whitelist: before.process_whitelist.clone(),
        ..before.clone()
    };
    assert_eq!(
        changed_config_fields(&before, &after),
        vec!["ignore_patterns".to_string()]
    );
}

#[test]
fn changed_config_fields_reports_retention_policy_changes() {
    let before = ProjectConfig::default();
    let after = ProjectConfig {
        retention: RetentionPolicy {
            snapshot_runs_threshold: before.retention.snapshot_runs_threshold + 1,
            ..before.retention.clone()
        },
        ..before.clone()
    };

    assert_eq!(changed_config_fields(&before, &after), vec!["retention"]);
}

#[test]
fn retention_policy_defaults_are_storage_maintenance_defaults() {
    let policy = RetentionPolicy::default();

    assert_eq!(policy.cleanup_review_db_bytes_threshold, 16 * 1024 * 1024);
    assert_eq!(policy.vacuum_reclaimable_bytes_threshold, 8 * 1024 * 1024);
    assert_eq!(policy.vacuum_reclaim_ratio_threshold_percent, 20);
    assert_eq!(policy.activity_rows_threshold, 1_000_000);
    assert_eq!(policy.verification_runs_threshold, 10_000);
    assert_eq!(policy.snapshot_runs_threshold, 100);
    assert_eq!(policy.activity_retention_days, 30);
    assert_eq!(policy.verification_retention_days, 60);
    assert_eq!(policy.keep_snapshot_runs, 20);
}
