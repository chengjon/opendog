pub const REGISTRY_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS projects (
    id          TEXT PRIMARY KEY,
    root_path   TEXT NOT NULL,
    db_path     TEXT NOT NULL,
    config      TEXT NOT NULL DEFAULT '{}',
    created_at  TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'active'
);
"#;

pub const PROJECT_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS snapshot (
    path            TEXT PRIMARY KEY,
    size            INTEGER NOT NULL,
    mtime           INTEGER NOT NULL,
    file_type       TEXT NOT NULL,
    scan_timestamp  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS snapshot_runs (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    captured_at   TEXT NOT NULL,
    file_count    INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS snapshot_history (
    run_id        INTEGER NOT NULL,
    path          TEXT NOT NULL,
    size          INTEGER NOT NULL,
    mtime         INTEGER NOT NULL,
    file_type     TEXT NOT NULL,
    PRIMARY KEY (run_id, path)
);

CREATE TABLE IF NOT EXISTS file_stats (
    file_path           TEXT PRIMARY KEY,
    access_count        INTEGER NOT NULL DEFAULT 0,
    estimated_duration_ms INTEGER NOT NULL DEFAULT 0,
    modification_count  INTEGER NOT NULL DEFAULT 0,
    last_access_time    TEXT,
    first_seen_time     TEXT,
    last_updated        TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS file_sightings (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path     TEXT NOT NULL,
    process_name  TEXT NOT NULL,
    pid           INTEGER NOT NULL,
    seen_at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS file_events (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path     TEXT NOT NULL,
    event_type    TEXT NOT NULL,
    event_time    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS verification_runs (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    kind          TEXT NOT NULL,
    status        TEXT NOT NULL,
    command       TEXT NOT NULL,
    exit_code     INTEGER,
    summary       TEXT,
    source        TEXT NOT NULL,
    started_at    TEXT,
    finished_at   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_snapshot_file_type ON snapshot(file_type);
CREATE INDEX IF NOT EXISTS idx_snapshot_runs_time ON snapshot_runs(captured_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_snapshot_runs_time_int ON snapshot_runs(CAST(captured_at AS INTEGER) DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_snapshot_history_run ON snapshot_history(run_id);
CREATE INDEX IF NOT EXISTS idx_snapshot_history_path ON snapshot_history(path);
CREATE INDEX IF NOT EXISTS idx_file_stats_access_count ON file_stats(access_count);
CREATE INDEX IF NOT EXISTS idx_file_sightings_file ON file_sightings(file_path);
CREATE INDEX IF NOT EXISTS idx_file_sightings_time ON file_sightings(seen_at);
CREATE INDEX IF NOT EXISTS idx_file_sightings_time_int ON file_sightings(CAST(seen_at AS INTEGER));
CREATE INDEX IF NOT EXISTS idx_file_events_file ON file_events(file_path);
CREATE INDEX IF NOT EXISTS idx_file_events_time ON file_events(event_time);
CREATE INDEX IF NOT EXISTS idx_file_events_modify_time_int ON file_events(CAST(event_time AS INTEGER)) WHERE event_type = 'modify';
CREATE INDEX IF NOT EXISTS idx_verification_runs_kind_time ON verification_runs(kind, finished_at DESC);
CREATE INDEX IF NOT EXISTS idx_verification_runs_finished_time_int ON verification_runs(CAST(finished_at AS INTEGER));

CREATE TABLE IF NOT EXISTS governance_lanes (
    lane_id     TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    description TEXT,
    status      TEXT NOT NULL DEFAULT 'active',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS governance_nodes (
    node_id           TEXT PRIMARY KEY,
    lane_id           TEXT NOT NULL,
    state             TEXT NOT NULL,
    summary           TEXT,
    evidence_refs     TEXT,
    artifact_refs     TEXT,
    reported_git_head TEXT,
    suggested_next    TEXT,
    forbidden_scope   TEXT,
    external_anchors  TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_governance_nodes_lane ON governance_nodes(lane_id);
CREATE INDEX IF NOT EXISTS idx_governance_nodes_lane_updated ON governance_nodes(lane_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_governance_nodes_state ON governance_nodes(state);

CREATE TABLE IF NOT EXISTS data_risk_cache (
    id                      INTEGER PRIMARY KEY CHECK (id = 1),
    mock_candidate_count    INTEGER NOT NULL DEFAULT 0,
    hardcoded_candidate_count INTEGER NOT NULL DEFAULT 0,
    mixed_review_file_count INTEGER NOT NULL DEFAULT 0,
    updated_at              TEXT NOT NULL
);
"#;

pub const SCHEMA_VERSION: u32 = 6;

#[cfg(test)]
mod tests {
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
        assert_eq!(SCHEMA_VERSION, 6);
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
}
