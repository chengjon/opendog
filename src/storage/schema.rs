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
"#;

pub const SCHEMA_VERSION: u32 = 4;
