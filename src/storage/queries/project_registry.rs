use crate::config::{normalize_project_overrides, ProjectConfigOverrides, ProjectInfo};
use crate::error::{OpenDogError, Result};
use crate::storage::database::Database;
use rusqlite::params;
use std::path::Path;

pub fn insert_project(db: &Database, info: &ProjectInfo) -> Result<()> {
    let config_json = serde_json::to_string(&info.config)?;
    db.execute(
        "INSERT INTO projects (id, root_path, db_path, config, created_at, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![info.id, info.root_path.to_str(), info.db_path.to_str(), config_json, info.created_at, info.status],
    )?;
    Ok(())
}

pub fn get_project(db: &Database, id: &str) -> Result<Option<ProjectInfo>> {
    let result = db.query_row(
        "SELECT id, root_path, db_path, config, created_at, status FROM projects WHERE id = ?1",
        params![id],
        |row| {
            let config_str: String = row.get(3)?;
            let config =
                normalize_project_overrides(serde_json::from_str(&config_str).unwrap_or_default());
            Ok(ProjectInfo {
                id: row.get(0)?,
                root_path: Path::new(&row.get::<_, String>(1)?).to_path_buf(),
                db_path: Path::new(&row.get::<_, String>(2)?).to_path_buf(),
                config,
                created_at: row.get(4)?,
                status: row.get(5)?,
            })
        },
    );
    match result {
        Ok(info) => Ok(Some(info)),
        Err(OpenDogError::Database(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn list_projects(db: &Database) -> Result<Vec<ProjectInfo>> {
    db.prepare_and_query(
        "SELECT id, root_path, db_path, config, created_at, status FROM projects WHERE status = 'active' ORDER BY created_at",
        params![],
        |row| {
            let config_str: String = row.get(3)?;
            let config = normalize_project_overrides(serde_json::from_str(&config_str).unwrap_or_default());
            Ok(ProjectInfo {
                id: row.get(0)?,
                root_path: Path::new(&row.get::<_, String>(1)?).to_path_buf(),
                db_path: Path::new(&row.get::<_, String>(2)?).to_path_buf(),
                config,
                created_at: row.get(4)?,
                status: row.get(5)?,
            })
        },
    )
}

pub fn delete_project(db: &Database, id: &str) -> Result<bool> {
    let rows = db.execute(
        "UPDATE projects SET status = 'deleted' WHERE id = ?1 AND status = 'active'",
        params![id],
    )?;
    Ok(rows > 0)
}

pub fn update_project_config(
    db: &Database,
    id: &str,
    config: &ProjectConfigOverrides,
) -> Result<()> {
    let config_json = serde_json::to_string(config)?;
    let rows = db.execute(
        "UPDATE projects SET config = ?1 WHERE id = ?2 AND status = 'active'",
        params![config_json, id],
    )?;
    if rows == 0 {
        return Err(OpenDogError::ProjectNotFound(id.to_string()));
    }
    Ok(())
}
