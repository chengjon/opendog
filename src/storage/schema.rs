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

CREATE TABLE IF NOT EXISTS file_stats (
    file_path           TEXT PRIMARY KEY,
    access_count        INTEGER NOT NULL DEFAULT 0,
    estimated_duration_ms INTEGER NOT NULL DEFAULT 0,
    modification_count  INTEGER NOT NULL DEFAULT 0,
    last_access_time    TEXT,
    first_seen_time     TEXT NOT NULL,
    last_updated        TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_snapshot_file_type ON snapshot(file_type);
CREATE INDEX IF NOT EXISTS idx_file_stats_access_count ON file_stats(access_count);
"#;

pub const SCHEMA_VERSION: u32 = 1;
