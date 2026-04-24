use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::{schemars, tool, tool_router};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use crate::config::ProjectInfo;
use crate::core::monitor::{self, MonitorHandle};
use crate::core::project::ProjectManager;
use crate::core::{snapshot, stats};
use crate::error::OpenDogError;
use crate::storage::database::Database;

pub struct OpenDogServer {
    inner: Mutex<ServerInner>,
}

struct ServerInner {
    pm: ProjectManager,
    monitors: HashMap<String, MonitorHandle>,
}

pub fn run_stdio() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    rt.block_on(async {
        use rmcp::ServiceExt;

        let server = OpenDogServer::new().expect("Failed to create OpenDogServer");
        let transport = (tokio::io::stdin(), tokio::io::stdout());
        server
            .serve(transport)
            .await
            .expect("MCP server exited with error");
    });
}

impl OpenDogServer {
    pub fn new() -> Result<Self, OpenDogError> {
        let pm = ProjectManager::new()?;
        Ok(Self {
            inner: Mutex::new(ServerInner {
                pm,
                monitors: HashMap::new(),
            }),
        })
    }

    fn get_project(&self, id: &str) -> Result<(Database, ProjectInfo), OpenDogError> {
        let inner = self.inner.lock().unwrap();
        let info = inner
            .pm
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = inner.pm.open_project_db(id)?;
        drop(inner);
        Ok((db, info))
    }
}

// --- Parameter structs ---

#[derive(Deserialize, schemars::JsonSchema)]
pub struct CreateProjectParams {
    /// Unique project identifier (alphanumeric, dash, underscore, max 64 chars)
    pub id: String,
    /// Absolute path to the project root directory
    pub path: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct ProjectIdParams {
    /// Project identifier
    pub id: String,
}

// --- Tool handlers ---

#[tool_router(server_handler)]
impl OpenDogServer {
    #[tool(name = "create_project", description = "Register a new project with a unique ID and root directory path for file monitoring")]
    fn create_project(
        &self,
        Parameters(CreateProjectParams { id, path }): Parameters<CreateProjectParams>,
    ) -> Json<Value> {
        let inner = self.inner.lock().unwrap();
        match inner.pm.create(&id, Path::new(&path)) {
            Ok(info) => Json(json!({
                "id": info.id,
                "root_path": info.root_path.display().to_string(),
                "status": "created"
            })),
            Err(e) => Json(json!({"error": e.to_string()})),
        }
    }

    #[tool(name = "take_snapshot", description = "Trigger a full recursive file scan for a project, recording file paths, sizes, and metadata")]
    fn take_snapshot(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        let result = (|| -> crate::error::Result<_> {
            let (db, info) = self.get_project(&id)?;
            snapshot::take_snapshot(&db, &info.root_path, &info.config)
        })();
        match result {
            Ok(r) => Json(json!({
                "project_id": id,
                "total_files": r.total_files,
                "new_files": r.new_files,
                "removed_files": r.removed_files,
            })),
            Err(e) => Json(json!({"error": e.to_string()})),
        }
    }

    #[tool(name = "start_monitor", description = "Start file monitoring for a project — begins /proc scanning and inotify change detection")]
    fn start_monitor(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        let result = (|| -> crate::error::Result<_> {
            let mut inner = self.inner.lock().unwrap();
            if inner.monitors.contains_key(&id) {
                return Err(OpenDogError::InvalidProjectId(format!(
                    "Monitor already running for project '{}'",
                    id
                )));
            }
            let info = inner
                .pm
                .get(&id)?
                .ok_or_else(|| OpenDogError::ProjectNotFound(id.clone()))?;
            let handle = monitor::start_monitor(
                &info.db_path,
                info.root_path.clone(),
                info.config.clone(),
            )?;
            inner.monitors.insert(id.clone(), handle);
            Ok(())
        })();
        match result {
            Ok(()) => Json(json!({
                "project_id": id,
                "status": "monitoring"
            })),
            Err(e) => Json(json!({"error": e.to_string()})),
        }
    }

    #[tool(name = "stop_monitor", description = "Stop file monitoring for a project")]
    fn stop_monitor(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        let mut inner = self.inner.lock().unwrap();
        match inner.monitors.remove(&id) {
            Some(handle) => {
                handle.stop();
                Json(json!({
                    "project_id": id,
                    "status": "stopped"
                }))
            }
            None => Json(json!({
                "error": format!("No monitor running for project '{}'", id)
            })),
        }
    }

    #[tool(name = "get_stats", description = "Query usage statistics for a project — per-file access count, estimated duration, modifications, last access")]
    fn get_stats(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        let result = (|| -> crate::error::Result<_> {
            let (db, _info) = self.get_project(&id)?;
            let summary = stats::get_summary(&db)?;
            let entries = stats::get_stats(&db)?;
            Ok((summary, entries))
        })();
        match result {
            Ok((summary, entries)) => {
                let files: Vec<Value> = entries
                    .iter()
                    .map(|e| {
                        json!({
                            "path": e.file_path,
                            "size": e.size,
                            "file_type": e.file_type,
                            "access_count": e.access_count,
                            "estimated_duration_ms": e.estimated_duration_ms,
                            "modification_count": e.modification_count,
                            "last_access_time": e.last_access_time,
                        })
                    })
                    .collect();
                Json(json!({
                    "project_id": id,
                    "summary": {
                        "total_files": summary.total_files,
                        "accessed": summary.accessed_files,
                        "unused": summary.unused_files,
                    },
                    "files": files,
                }))
            }
            Err(e) => Json(json!({"error": e.to_string()})),
        }
    }

    #[tool(name = "get_unused_files", description = "List never-accessed files for a project — candidates for cleanup or removal review")]
    fn get_unused_files(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        let result = (|| -> crate::error::Result<_> {
            let (db, _info) = self.get_project(&id)?;
            let unused = stats::get_unused_files(&db)?;
            Ok(unused)
        })();
        match result {
            Ok(unused) => {
                let files: Vec<Value> = unused
                    .iter()
                    .map(|e| {
                        json!({
                            "path": e.file_path,
                            "size": e.size,
                            "file_type": e.file_type,
                        })
                    })
                    .collect();
                Json(json!({
                    "project_id": id,
                    "unused_count": files.len(),
                    "files": files,
                }))
            }
            Err(e) => Json(json!({"error": e.to_string()})),
        }
    }

    #[tool(name = "list_projects", description = "List all registered projects with their status, root path, and database location")]
    fn list_projects(&self) -> Json<Value> {
        let inner = self.inner.lock().unwrap();
        match inner.pm.list() {
            Ok(projects) => {
                let list: Vec<Value> = projects
                    .iter()
                    .map(|p| {
                        json!({
                            "id": p.id,
                            "root_path": p.root_path.display().to_string(),
                            "status": p.status,
                            "created_at": p.created_at,
                        })
                    })
                    .collect();
                Json(json!({
                    "projects": list,
                    "count": list.len(),
                }))
            }
            Err(e) => Json(json!({"error": e.to_string()})),
        }
    }

    #[tool(name = "delete_project", description = "Delete a project and all its associated data — database, configuration, stops monitor if running")]
    fn delete_project(
        &self,
        Parameters(ProjectIdParams { id }): Parameters<ProjectIdParams>,
    ) -> Json<Value> {
        let mut inner = self.inner.lock().unwrap();
        // Stop monitor if running
        if let Some(handle) = inner.monitors.remove(&id) {
            handle.stop();
        }
        match inner.pm.delete(&id) {
            Ok(true) => Json(json!({
                "id": id,
                "status": "deleted"
            })),
            Ok(false) => Json(json!({
                "error": format!("Project '{}' not found", id)
            })),
            Err(e) => Json(json!({"error": e.to_string()})),
        }
    }
}
