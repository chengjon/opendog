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
            let config = parse_config_or_warn(&config_str);
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
            let config = parse_config_or_warn(&config_str);
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

fn parse_config_or_warn(config_str: &str) -> ProjectConfigOverrides {
    match serde_json::from_str(config_str) {
        Ok(overrides) => normalize_project_overrides(overrides),
        Err(e) => {
            tracing::warn!(
                "malformed project config JSON ({}), falling back to defaults: {}",
                e,
                config_str.chars().take(80).collect::<String>()
            );
            ProjectConfigOverrides::default()
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProjectConfigOverrides;
    use crate::storage::database::Database;
    use std::path::PathBuf;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("registry_test.db");
        let db = Database::open_registry(&db_path).unwrap();
        Box::leak(Box::new(dir));
        db
    }

    fn sample_info(id: &str) -> ProjectInfo {
        ProjectInfo {
            id: id.to_string(),
            root_path: PathBuf::from("/tmp/project"),
            db_path: PathBuf::from("/tmp/project.db"),
            config: ProjectConfigOverrides::default(),
            created_at: "2026-01-01".to_string(),
            status: "active".to_string(),
        }
    }

    #[test]
    fn insert_get_and_list_project() {
        let db = test_db();
        let info = sample_info("proj-1");
        insert_project(&db, &info).unwrap();

        let found = get_project(&db, "proj-1").unwrap().unwrap();
        assert_eq!(found.id, "proj-1");
        assert_eq!(found.status, "active");
        assert_eq!(found.config.ignore_patterns, None);

        let missing = get_project(&db, "nope").unwrap();
        assert!(missing.is_none());

        let listed = list_projects(&db).unwrap();
        assert_eq!(listed.len(), 1);
    }

    #[test]
    fn delete_soft_marks_deleted() {
        let db = test_db();
        insert_project(&db, &sample_info("proj-2")).unwrap();
        assert!(delete_project(&db, "proj-2").unwrap());
        // Soft-deleted → get still returns it, but list filters it out
        let found = get_project(&db, "proj-2").unwrap().unwrap();
        assert_eq!(found.status, "deleted");
        assert!(list_projects(&db).unwrap().is_empty());
        // Double-delete returns false
        assert!(!delete_project(&db, "proj-2").unwrap());
    }

    #[test]
    fn update_project_config_persists() {
        let db = test_db();
        insert_project(&db, &sample_info("proj-3")).unwrap();
        let new_config = ProjectConfigOverrides {
            ignore_patterns: Some(vec!["target/**".to_string()]),
            process_whitelist: Some(vec!["codex".to_string()]),
        };
        update_project_config(&db, "proj-3", &new_config).unwrap();
        let updated = get_project(&db, "proj-3").unwrap().unwrap();
        assert_eq!(
            updated.config.ignore_patterns,
            Some(vec!["target/**".to_string()])
        );
    }

    #[test]
    fn update_config_rejects_unknown_project() {
        let db = test_db();
        let err = update_project_config(&db, "ghost", &ProjectConfigOverrides::default()).unwrap_err();
        assert!(err.to_string().contains("ghost"));
    }

    #[test]
    fn get_project_handles_malformed_config_json() {
        let db = test_db();
        // Directly insert a row with bad JSON to test parse_config_or_warn fallback
        db.execute(
            "INSERT INTO projects (id, root_path, db_path, config, created_at, status) VALUES ('bad-config', '/tmp', '/tmp/db', 'not-valid-json', '0', 'active')",
            rusqlite::params![],
        )
        .unwrap();
        let found = get_project(&db, "bad-config").unwrap().unwrap();
        assert_eq!(found.id, "bad-config");
        // Falls back to default config
        assert_eq!(found.config.ignore_patterns, None);
    }
}
