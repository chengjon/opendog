use crate::config::{self, ProjectConfig, ProjectInfo};
use crate::error::{OpenDogError, Result};
use crate::storage::database::Database;
use crate::storage::queries;
use std::path::{Path, PathBuf};

pub struct ProjectManager {
    registry: Database,
    data_dir: PathBuf,
}

impl ProjectManager {
    pub fn new() -> Result<Self> {
        let data_dir = config::data_dir();
        Self::with_data_dir(&data_dir)
    }

    pub fn with_data_dir(data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let registry_path = data_dir.join("registry.db");
        let registry = Database::open_registry(&registry_path)?;
        Ok(Self {
            registry,
            data_dir: data_dir.to_path_buf(),
        })
    }

    pub fn create(&self, id: &str, root_path: &Path) -> Result<ProjectInfo> {
        if !config::validate_project_id(id) {
            return Err(OpenDogError::InvalidProjectId(id.to_string()));
        }
        if !config::validate_root_path(root_path) {
            return Err(OpenDogError::InvalidPath(root_path.display().to_string()));
        }
        if queries::get_project(&self.registry, id)?.is_some() {
            return Err(OpenDogError::ProjectExists(id.to_string()));
        }

        let projects_dir = self.data_dir.join("projects");
        std::fs::create_dir_all(&projects_dir)?;
        let db_path = projects_dir.join(format!("{}.db", id));

        let info = ProjectInfo {
            id: id.to_string(),
            root_path: root_path.to_path_buf(),
            db_path: db_path.clone(),
            config: ProjectConfig::default(),
            created_at: now_iso(),
            status: "active".to_string(),
        };

        // Create the per-project database to verify it works
        {
            let _project_db = Database::open_project(&db_path)?;
        }

        queries::insert_project(&self.registry, &info)?;
        Ok(info)
    }

    pub fn get(&self, id: &str) -> Result<Option<ProjectInfo>> {
        queries::get_project(&self.registry, id)
    }

    pub fn list(&self) -> Result<Vec<ProjectInfo>> {
        queries::list_projects(&self.registry)
    }

    pub fn delete(&self, id: &str) -> Result<bool> {
        let info = queries::get_project(&self.registry, id).ok().flatten();

        let deleted = queries::delete_project(&self.registry, id)?;

        if deleted {
            if let Some(info) = info {
                let _ = std::fs::remove_file(&info.db_path);
            }
        }

        Ok(deleted)
    }

    pub fn open_project_db(&self, id: &str) -> Result<Database> {
        let info = queries::get_project(&self.registry, id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        Database::open_project(&info.db_path)
    }
}

fn now_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}
