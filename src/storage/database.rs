use crate::error::Result;
use rusqlite::{Connection, Params};
use std::path::Path;

use super::migrations::{self, SchemaKind};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;",
        )?;
        Ok(Self { conn })
    }

    pub fn open_registry(path: &Path) -> Result<Self> {
        let db = Self::open(path)?;
        migrations::migrate(&db.conn, SchemaKind::Registry)?;
        Ok(db)
    }

    pub fn open_project(path: &Path) -> Result<Self> {
        let db = Self::open(path)?;
        migrations::migrate(&db.conn, SchemaKind::Project)?;
        Ok(db)
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn execute<P: Params>(&self, sql: &str, params: P) -> Result<usize> {
        Ok(self.conn.execute(sql, params)?)
    }

    pub fn query_row<T, P, F>(&self, sql: &str, params: P, f: F) -> Result<T>
    where
        P: Params,
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        Ok(self.conn.query_row(sql, params, f)?)
    }

    pub fn prepare_and_query<T, P, F>(&self, sql: &str, params: P, f: F) -> Result<Vec<T>>
    where
        P: Params,
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params, f)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn execute_batch(&self, sql: &str) -> Result<()> {
        self.conn.execute_batch(sql)?;
        Ok(())
    }
}
