use rusqlite::Connection;

use crate::error::{OpenDogError, Result};

use super::schema::{PROJECT_SCHEMA, REGISTRY_SCHEMA, SCHEMA_VERSION};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaKind {
    Registry,
    Project,
}

impl SchemaKind {
    fn schema_sql(self) -> &'static str {
        match self {
            Self::Registry => REGISTRY_SCHEMA,
            Self::Project => PROJECT_SCHEMA,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Registry => "registry",
            Self::Project => "project",
        }
    }
}

pub fn migrate(conn: &Connection, kind: SchemaKind) -> Result<()> {
    let current_version = user_version(conn)?;
    if current_version > SCHEMA_VERSION {
        return Err(OpenDogError::SchemaMigration(format!(
            "{} database schema version {} is newer than supported version {}",
            kind.label(),
            current_version,
            SCHEMA_VERSION
        )));
    }

    conn.execute_batch(kind.schema_sql())?;
    set_user_version(conn, SCHEMA_VERSION)?;
    Ok(())
}

pub fn user_version(conn: &Connection) -> Result<u32> {
    let version: u32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    Ok(version)
}

fn set_user_version(conn: &Connection, version: u32) -> Result<()> {
    conn.pragma_update(None, "user_version", version)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rusqlite::Connection;

    use super::*;
    use crate::storage::database::Database;

    fn create_v3_project_fixture(path: &Path) {
        let conn = Connection::open(path).expect("fixture db opens");
        conn.execute_batch(PROJECT_SCHEMA)
            .expect("fixture schema creates");
        set_user_version(&conn, 3).expect("fixture user_version set");

        conn.execute(
            "INSERT INTO snapshot (path, size, mtime, file_type, scan_timestamp)
             VALUES ('src/main.rs', 42, 100, 'source', '100')",
            [],
        )
        .expect("snapshot fixture row inserts");
        conn.execute(
            "INSERT INTO file_stats (
                file_path,
                access_count,
                estimated_duration_ms,
                modification_count,
                first_seen_time,
                last_updated
             )
             VALUES ('src/main.rs', 3, 500, 1, '100', '200')",
            [],
        )
        .expect("file_stats fixture row inserts");
        conn.execute(
            "INSERT INTO verification_runs (
                kind,
                status,
                command,
                exit_code,
                summary,
                source,
                started_at,
                finished_at
             )
             VALUES ('test', 'passed', 'cargo test', 0, 'ok', 'fixture', '100', '200')",
            [],
        )
        .expect("verification fixture row inserts");
    }

    #[test]
    fn fresh_registry_database_sets_current_user_version() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("registry.db");

        let db = Database::open_registry(&db_path).expect("registry opens");

        assert_eq!(
            user_version(db.conn()).expect("user_version reads"),
            SCHEMA_VERSION
        );
    }

    #[test]
    fn fresh_project_database_sets_current_user_version() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("project.db");

        let db = Database::open_project(&db_path).expect("project opens");

        assert_eq!(
            user_version(db.conn()).expect("user_version reads"),
            SCHEMA_VERSION
        );
    }

    #[test]
    fn v3_project_fixture_migrates_forward_and_preserves_data() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("project.db");
        create_v3_project_fixture(&db_path);

        let db = Database::open_project(&db_path).expect("project migrates");

        assert_eq!(
            user_version(db.conn()).expect("user_version reads"),
            SCHEMA_VERSION
        );
        let snapshot_count: i64 = db
            .query_row("SELECT COUNT(*) FROM snapshot", [], |row| row.get(0))
            .expect("snapshot count reads");
        let access_count: i64 = db
            .query_row(
                "SELECT access_count FROM file_stats WHERE file_path = 'src/main.rs'",
                [],
                |row| row.get(0),
            )
            .expect("stats row reads");
        let verification_count: i64 = db
            .query_row("SELECT COUNT(*) FROM verification_runs", [], |row| {
                row.get(0)
            })
            .expect("verification count reads");

        assert_eq!(snapshot_count, 1);
        assert_eq!(access_count, 3);
        assert_eq!(verification_count, 1);
    }

    #[test]
    fn newer_schema_version_is_rejected() {
        let conn = Connection::open_in_memory().expect("memory db opens");
        set_user_version(&conn, SCHEMA_VERSION + 1).expect("future user_version set");

        let err = migrate(&conn, SchemaKind::Project).expect_err("future schema rejected");

        assert!(err.to_string().contains("newer than supported"));
    }

    #[test]
    fn migrates_v4_to_v5_preserving_verification_runs() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("v4_migration.db");

        // Create a v4 database with existing data
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;",
            )
            .unwrap();

            // Run v4 schema (all tables except governance)
            let v4_schema = include_str!("../storage/schema_v4_seed.sql");
            conn.execute_batch(v4_schema).unwrap();

            // Insert a verification run to preserve
            conn.execute(
                "INSERT INTO verification_runs (kind, status, command, exit_code, summary, source, started_at, finished_at)
                 VALUES ('test', 'passed', 'cargo test', 0, 'all passed', 'migration-test', '1000', '1001')",
                [],
            ).unwrap();

            conn.pragma_update(None, "user_version", 4).unwrap();
        }

        // Open with migration
        let db = Database::open_project(&db_path).unwrap();

        // Data preserved
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM verification_runs WHERE kind='test'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // New tables exist
        db.conn()
            .execute_batch("SELECT * FROM governance_lanes LIMIT 0")
            .unwrap();
        db.conn()
            .execute_batch("SELECT * FROM governance_nodes LIMIT 0")
            .unwrap();

        // Version bumped
        let version: i64 = db
            .conn()
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION as i64);
    }
}
