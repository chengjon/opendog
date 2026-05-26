use super::*;

#[test]
fn cleanup_validation_requires_explicit_action_parameters() {
    let db = test_db();

    let error = cleanup_project_data_at(
        &db,
        &ProjectDataCleanupRequest {
            scope: CleanupScope::All,
            older_than_days: None,
            keep_snapshot_runs: None,
            vacuum: false,
            dry_run: true,
        },
        1,
    )
    .unwrap_err();

    assert!(error.to_string().contains("cleanup"));
}

#[test]
fn cleanup_validation_rejects_vacuum_in_dry_run_mode() {
    let db = test_db();

    let error = cleanup_project_data_at(
        &db,
        &ProjectDataCleanupRequest {
            scope: CleanupScope::Activity,
            older_than_days: Some(1),
            keep_snapshot_runs: None,
            vacuum: true,
            dry_run: true,
        },
        1,
    )
    .unwrap_err();

    assert!(error.to_string().contains("vacuum"));
}

#[test]
fn cleanup_scope_parse_rejects_invalid_value() {
    let err = CleanupScope::parse("everything").unwrap_err();
    assert!(err
        .to_string()
        .contains("cleanup scope must be one of: activity, snapshots, verification, all"));
    assert!(err.to_string().contains("everything"));
}

#[test]
fn cleanup_scope_parse_activity() {
    assert_eq!(
        CleanupScope::parse("activity").unwrap(),
        CleanupScope::Activity
    );
}

#[test]
fn cleanup_scope_parse_snapshots() {
    assert_eq!(
        CleanupScope::parse("snapshots").unwrap(),
        CleanupScope::Snapshots
    );
}

#[test]
fn cleanup_scope_parse_verification() {
    assert_eq!(
        CleanupScope::parse("verification").unwrap(),
        CleanupScope::Verification
    );
}

#[test]
fn cleanup_scope_parse_all() {
    assert_eq!(CleanupScope::parse("all").unwrap(), CleanupScope::All);
}

#[test]
fn cleanup_scope_parse_empty_string_is_error() {
    assert!(CleanupScope::parse("").is_err());
}

#[test]
fn cleanup_scope_as_str_activity() {
    assert_eq!(CleanupScope::Activity.as_str(), "activity");
}

#[test]
fn cleanup_scope_as_str_snapshots() {
    assert_eq!(CleanupScope::Snapshots.as_str(), "snapshots");
}

#[test]
fn cleanup_scope_as_str_verification() {
    assert_eq!(CleanupScope::Verification.as_str(), "verification");
}

#[test]
fn cleanup_scope_as_str_all() {
    assert_eq!(CleanupScope::All.as_str(), "all");
}

#[test]
fn cleanup_scope_roundtrip_parse_as_str() {
    for (label, scope) in [
        ("activity", CleanupScope::Activity),
        ("snapshots", CleanupScope::Snapshots),
        ("verification", CleanupScope::Verification),
        ("all", CleanupScope::All),
    ] {
        assert_eq!(CleanupScope::parse(label).unwrap(), scope);
        assert_eq!(scope.as_str(), label);
    }
}

// -----------------------------------------------------------------------
// Snapshot-only cleanup integration tests
// -----------------------------------------------------------------------
