use std::sync::{Mutex, MutexGuard};

use crate::config::{ProjectConfig, ProjectInfo};
use crate::control::MonitorController;
use crate::error::OpenDogError;
use crate::storage::database::Database;

pub struct OpenDogServer {
    pub(super) inner: Mutex<MonitorController>,
}

pub fn run_stdio() {
    if let Err(e) = try_run_stdio() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

pub fn try_run_stdio() -> Result<(), OpenDogError> {
    crate::daemon::ensure_running_for_mcp()?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        use rmcp::ServiceExt;

        let server = OpenDogServer::new()?;
        let transport = (tokio::io::stdin(), tokio::io::stdout());
        let running = server
            .serve(transport)
            .await
            .map_err(|e| OpenDogError::Mcp(format!("MCP server exited with error: {}", e)))?;
        running
            .waiting()
            .await
            .map_err(|e| OpenDogError::Mcp(format!("MCP server task join failed: {}", e)))?;
        Ok::<(), OpenDogError>(())
    })
}

impl OpenDogServer {
    pub fn new() -> Result<Self, OpenDogError> {
        Ok(Self {
            inner: Mutex::new(MonitorController::new()?),
        })
    }

    pub(super) fn get_project(&self, id: &str) -> Result<(Database, ProjectInfo), OpenDogError> {
        let inner = self.lock_inner()?;
        let info = inner
            .project_manager()
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = inner.project_manager().open_project_db(id)?;
        drop(inner);
        Ok((db, info))
    }

    pub(super) fn get_project_with_config(
        &self,
        id: &str,
    ) -> Result<(ProjectInfo, ProjectConfig), OpenDogError> {
        let inner = self.lock_inner()?;
        let info = inner
            .project_manager()
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let config = inner.project_manager().resolve_project_config(&info)?;
        drop(inner);
        Ok((info, config))
    }

    pub(super) fn lock_inner(&self) -> Result<MutexGuard<'_, MonitorController>, OpenDogError> {
        self.inner
            .lock()
            .map_err(|e| OpenDogError::LockPoisoned(format!("OpenDogServer controller: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::OpenDogServer;
    use crate::control::MonitorController;
    use crate::core::project::ProjectManager;
    use crate::error::OpenDogError;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::Mutex;

    #[test]
    fn poisoned_controller_lock_returns_structured_error() {
        let dir = tempfile::tempdir().expect("tempdir should initialize");
        let pm =
            ProjectManager::with_data_dir(dir.path()).expect("project manager should initialize");
        let server = OpenDogServer {
            inner: Mutex::new(MonitorController::with_project_manager(pm)),
        };

        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _guard = server.inner.lock().expect("initial lock should succeed");
            panic!("poison controller lock");
        }));
        assert!(panic_result.is_err());

        match server.lock_inner() {
            Err(OpenDogError::LockPoisoned(message)) => {
                assert!(message.contains("OpenDogServer controller"));
            }
            Err(err) => panic!("expected lock poison error, got {err}"),
            Ok(_) => panic!("expected poisoned lock to return an error"),
        };
    }
}
