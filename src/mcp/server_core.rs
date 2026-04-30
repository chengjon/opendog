use std::sync::Mutex;

use crate::config::ProjectInfo;
use crate::control::MonitorController;
use crate::error::OpenDogError;
use crate::storage::database::Database;

pub struct OpenDogServer {
    pub(super) inner: Mutex<MonitorController>,
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
        Ok(Self {
            inner: Mutex::new(MonitorController::new()?),
        })
    }

    pub(super) fn get_project(&self, id: &str) -> Result<(Database, ProjectInfo), OpenDogError> {
        let inner = self.inner.lock().unwrap();
        let info = inner
            .project_manager()
            .get(id)?
            .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
        let db = inner.project_manager().open_project_db(id)?;
        drop(inner);
        Ok((db, info))
    }
}
