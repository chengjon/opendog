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
            "{} database schema version {} is newer than supported version {}. \
             Restart the daemon and MCP session with the current binary, then retry.",
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
mod tests;
