use crate::config::{
    self, apply_global_config_patch, apply_project_config_patch, load_global_config_from_path,
    resolve_project_config, save_global_config_to_path, ConfigPatch, ProjectConfig,
    ProjectConfigOverrides, ProjectConfigPatch, ProjectConfigUpdateResult, ProjectConfigView,
    ProjectInfo,
};
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
            config: ProjectConfigOverrides::default(),
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
                match std::fs::remove_file(&info.db_path) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e.into()),
                }
            }
        }

        Ok(deleted)
    }

    pub fn open_project_db(&self, id: &str) -> Result<Database> {
        let info = queries::get_project(&self.registry, id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        Database::open_project(&info.db_path)
    }

    pub fn global_config(&self) -> Result<ProjectConfig> {
        load_global_config_from_path(&self.global_config_path())
    }

    pub fn update_global_config(&self, patch: ConfigPatch) -> Result<ProjectConfig> {
        if patch.is_empty() {
            return Err(OpenDogError::InvalidInput(
                "config patch must change at least one field".to_string(),
            ));
        }
        let current = self.global_config()?;
        let updated = apply_global_config_patch(&current, patch);
        save_global_config_to_path(&self.global_config_path(), &updated)?;
        Ok(updated)
    }

    pub fn effective_project_config(&self, id: &str) -> Result<ProjectConfig> {
        let info = self
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        self.resolve_project_config(&info)
    }

    pub fn resolve_project_config(&self, info: &ProjectInfo) -> Result<ProjectConfig> {
        let global_defaults = self.global_config()?;
        Ok(resolve_project_config(&global_defaults, &info.config))
    }

    pub fn project_config_view(&self, id: &str) -> Result<ProjectConfigView> {
        let info = self
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let global_defaults = self.global_config()?;
        Ok(ProjectConfigView {
            project_id: info.id,
            global_defaults: global_defaults.clone(),
            project_overrides: info.config.clone(),
            effective: resolve_project_config(&global_defaults, &info.config),
        })
    }

    pub fn update_project_config(
        &self,
        id: &str,
        patch: ProjectConfigPatch,
    ) -> Result<ProjectConfigUpdateResult> {
        if patch.is_empty() {
            return Err(OpenDogError::InvalidInput(
                "project config patch must change at least one field".to_string(),
            ));
        }

        let info = self
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let global_defaults = self.global_config()?;
        let effective = resolve_project_config(&global_defaults, &info.config);
        let updated_overrides = apply_project_config_patch(&info.config, &effective, patch);
        queries::update_project_config(&self.registry, id, &updated_overrides)?;
        let effective = resolve_project_config(&global_defaults, &updated_overrides);

        Ok(ProjectConfigUpdateResult {
            project_id: id.to_string(),
            global_defaults,
            project_overrides: updated_overrides,
            effective,
            reload: Default::default(),
        })
    }

    fn global_config_path(&self) -> PathBuf {
        if self.data_dir.file_name().and_then(|name| name.to_str()) == Some("data") {
            self.data_dir
                .parent()
                .unwrap_or(self.data_dir.as_path())
                .join("config.json")
        } else {
            self.data_dir.join("config.json")
        }
    }
}

fn now_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pm() -> ProjectManager {
        let dir = tempfile::tempdir().unwrap();
        let pm = ProjectManager::with_data_dir(dir.path().join("data").as_path()).unwrap();
        Box::leak(Box::new(dir));
        pm
    }

    fn valid_root() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        // with_data_dir needs a "data" subdirectory pattern
        dir
    }

    #[test]
    fn create_rejects_empty_project_id() {
        let pm = test_pm();
        let root = tempfile::tempdir().unwrap();
        let err = pm.create("", root.path()).unwrap_err();
        assert!(matches!(err, OpenDogError::InvalidProjectId(_)));
    }

    #[test]
    fn create_rejects_project_id_with_spaces() {
        let pm = test_pm();
        let root = tempfile::tempdir().unwrap();
        let err = pm.create("has spaces", root.path()).unwrap_err();
        assert!(matches!(err, OpenDogError::InvalidProjectId(_)));
    }

    #[test]
    fn create_rejects_project_id_with_dots() {
        let pm = test_pm();
        let root = tempfile::tempdir().unwrap();
        let err = pm.create("../../etc", root.path()).unwrap_err();
        assert!(matches!(err, OpenDogError::InvalidProjectId(_)));
    }

    #[test]
    fn create_rejects_relative_root_path() {
        let pm = test_pm();
        let err = pm.create("valid-id", Path::new("relative/path")).unwrap_err();
        assert!(matches!(err, OpenDogError::InvalidPath(_)));
    }

    #[test]
    fn create_rejects_duplicate_project_id() {
        let pm = test_pm();
        let root = tempfile::tempdir().unwrap();
        pm.create("dup", root.path()).unwrap();
        let err = pm.create("dup", root.path()).unwrap_err();
        assert!(matches!(err, OpenDogError::ProjectExists(_)));
    }

    #[test]
    fn create_succeeds_with_valid_inputs() {
        let pm = test_pm();
        let root = tempfile::tempdir().unwrap();
        let info = pm.create("my-project", root.path()).unwrap();
        assert_eq!(info.id, "my-project");
        assert_eq!(info.root_path, root.path());
        assert!(info.db_path.to_string_lossy().contains("my-project.db"));
    }

    #[test]
    fn get_returns_created_project() {
        let pm = test_pm();
        let root = tempfile::tempdir().unwrap();
        pm.create("find-me", root.path()).unwrap();
        let info = pm.get("find-me").unwrap().unwrap();
        assert_eq!(info.id, "find-me");
    }

    #[test]
    fn get_returns_none_for_unknown() {
        let pm = test_pm();
        assert!(pm.get("ghost").unwrap().is_none());
    }

    #[test]
    fn list_returns_all_projects() {
        let pm = test_pm();
        let root = tempfile::tempdir().unwrap();
        pm.create("alpha", root.path()).unwrap();
        pm.create("beta", root.path()).unwrap();
        let list = pm.list().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn delete_soft_deletes_project() {
        let pm = test_pm();
        let root = tempfile::tempdir().unwrap();
        pm.create("bye", root.path()).unwrap();
        assert!(pm.delete("bye").unwrap());
        let info = pm.get("bye").unwrap().unwrap();
        assert_eq!(info.status, "deleted");
    }

    #[test]
    fn delete_returns_false_for_unknown() {
        let pm = test_pm();
        assert!(!pm.delete("ghost").unwrap());
    }

    #[test]
    fn update_global_config_rejects_empty_patch() {
        let pm = test_pm();
        let err = pm
            .update_global_config(ConfigPatch {
                ignore_patterns: None,
                process_whitelist: None,
                add_ignore_patterns: vec![],
                remove_ignore_patterns: vec![],
                add_process_whitelist: vec![],
                remove_process_whitelist: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, OpenDogError::InvalidInput(_)));
    }

    #[test]
    fn update_project_config_rejects_empty_patch() {
        let pm = test_pm();
        let root = tempfile::tempdir().unwrap();
        pm.create("cfg-test", root.path()).unwrap();
        let err = pm
            .update_project_config(
                "cfg-test",
                ProjectConfigPatch {
                    ignore_patterns: None,
                    process_whitelist: None,
                    add_ignore_patterns: vec![],
                    remove_ignore_patterns: vec![],
                    add_process_whitelist: vec![],
                    remove_process_whitelist: vec![],
                    inherit_ignore_patterns: false,
                    inherit_process_whitelist: false,
                },
            )
            .unwrap_err();
        assert!(matches!(err, OpenDogError::InvalidInput(_)));
    }

    #[test]
    fn update_project_config_rejects_unknown_project() {
        let pm = test_pm();
        let err = pm
            .update_project_config(
                "ghost",
                ProjectConfigPatch {
                    ignore_patterns: Some(vec!["*.log".to_string()]),
                    process_whitelist: None,
                    add_ignore_patterns: vec![],
                    remove_ignore_patterns: vec![],
                    add_process_whitelist: vec![],
                    remove_process_whitelist: vec![],
                    inherit_ignore_patterns: false,
                    inherit_process_whitelist: false,
                },
            )
            .unwrap_err();
        assert!(matches!(err, OpenDogError::ProjectNotFound(_)));
    }
}
