use super::*;

// ---- REGISTRY_SCHEMA ----

#[test]
fn registry_schema_contains_projects_table() {
    assert!(REGISTRY_SCHEMA.contains("CREATE TABLE IF NOT EXISTS projects"));
}

#[test]
fn registry_schema_projects_has_id_column() {
    assert!(REGISTRY_SCHEMA.contains("id") && REGISTRY_SCHEMA.contains("TEXT PRIMARY KEY"));
}

#[test]
fn registry_schema_projects_has_root_path() {
    assert!(REGISTRY_SCHEMA.contains("root_path"));
}

#[test]
fn registry_schema_projects_has_db_path() {
    assert!(REGISTRY_SCHEMA.contains("db_path"));
}

#[test]
fn registry_schema_projects_has_status() {
    assert!(REGISTRY_SCHEMA.contains("status"));
}

// ---- PROJECT_SCHEMA tables ----

#[test]
fn project_schema_contains_snapshot_table() {
    assert!(PROJECT_SCHEMA.contains("CREATE TABLE IF NOT EXISTS snapshot"));
}

#[test]
fn project_schema_contains_snapshot_runs_table() {
    assert!(PROJECT_SCHEMA.contains("CREATE TABLE IF NOT EXISTS snapshot_runs"));
}

#[test]
fn project_schema_contains_snapshot_history_table() {
    assert!(PROJECT_SCHEMA.contains("CREATE TABLE IF NOT EXISTS snapshot_history"));
}

#[test]
fn project_schema_contains_file_stats_table() {
    assert!(PROJECT_SCHEMA.contains("CREATE TABLE IF NOT EXISTS file_stats"));
}

#[test]
fn project_schema_contains_file_sightings_table() {
    assert!(PROJECT_SCHEMA.contains("CREATE TABLE IF NOT EXISTS file_sightings"));
}

#[test]
fn project_schema_contains_file_events_table() {
    assert!(PROJECT_SCHEMA.contains("CREATE TABLE IF NOT EXISTS file_events"));
}

#[test]
fn project_schema_contains_verification_runs_table() {
    assert!(PROJECT_SCHEMA.contains("CREATE TABLE IF NOT EXISTS verification_runs"));
}

#[test]
fn project_schema_contains_activity_daily_rollups_table() {
    assert!(PROJECT_SCHEMA.contains("CREATE TABLE IF NOT EXISTS activity_daily_rollups"));
}

#[test]
fn project_schema_contains_governance_lanes_table() {
    assert!(PROJECT_SCHEMA.contains("CREATE TABLE IF NOT EXISTS governance_lanes"));
}

#[test]
fn project_schema_contains_governance_nodes_table() {
    assert!(PROJECT_SCHEMA.contains("CREATE TABLE IF NOT EXISTS governance_nodes"));
}

#[test]
fn project_schema_contains_data_risk_cache_table() {
    assert!(PROJECT_SCHEMA.contains("CREATE TABLE IF NOT EXISTS data_risk_cache"));
}

// ---- PROJECT_SCHEMA indexes ----

#[test]
fn project_schema_has_file_type_index() {
    assert!(PROJECT_SCHEMA.contains("idx_snapshot_file_type"));
}

#[test]
fn project_schema_has_snapshot_runs_time_index() {
    assert!(PROJECT_SCHEMA.contains("idx_snapshot_runs_time"));
}

#[test]
fn project_schema_has_file_stats_access_count_index() {
    assert!(PROJECT_SCHEMA.contains("idx_file_stats_access_count"));
}

#[test]
fn project_schema_has_file_sightings_file_index() {
    assert!(PROJECT_SCHEMA.contains("idx_file_sightings_file"));
}

#[test]
fn project_schema_has_activity_daily_rollups_day_index() {
    assert!(PROJECT_SCHEMA.contains("idx_activity_daily_rollups_day"));
}

#[test]
fn project_schema_has_file_events_file_index() {
    assert!(PROJECT_SCHEMA.contains("idx_file_events_file"));
}

#[test]
fn project_schema_has_verification_runs_index() {
    assert!(PROJECT_SCHEMA.contains("idx_verification_runs_kind_time"));
}

#[test]
fn project_schema_has_governance_nodes_lane_index() {
    assert!(PROJECT_SCHEMA.contains("idx_governance_nodes_lane"));
}

#[test]
fn project_schema_has_governance_nodes_state_index() {
    assert!(PROJECT_SCHEMA.contains("idx_governance_nodes_state"));
}

// ---- SCHEMA_VERSION ----

#[test]
fn schema_version_is_current() {
    // The version should match the latest migration state
    assert_eq!(SCHEMA_VERSION, 7);
}

// ---- Structural integrity checks ----

#[test]
fn registry_schema_is_valid_sql_fragment() {
    // Should contain a semicolon at the end of the CREATE TABLE statement
    assert!(REGISTRY_SCHEMA.contains(';'));
}

#[test]
fn project_schema_has_multiple_statements() {
    // Count semicolons — should have many CREATE TABLE/INDEX statements
    let count = PROJECT_SCHEMA.matches(';').count();
    assert!(
        count > 10,
        "expected many SQL statements, found {} semicolons",
        count
    );
}

#[test]
fn data_risk_cache_has_single_row_constraint() {
    assert!(PROJECT_SCHEMA.contains("CHECK (id = 1)"));
}
