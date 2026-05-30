use std::sync::{Mutex, MutexGuard};

use crate::config::ProjectInfo;
use crate::control::protocol::StartMonitorOutcome;
use crate::control::MonitorController;
use crate::core::snapshot::SnapshotResult;
use crate::error::{OpenDogError, Result};
use serde_json::Value;

use super::{Guidance, ProjectLifecycle, SnapshotMonitor};

pub struct DirectProjectLifecycle<'a> {
    controller: &'a Mutex<MonitorController>,
}

impl<'a> DirectProjectLifecycle<'a> {
    pub fn new(controller: &'a Mutex<MonitorController>) -> Self {
        Self { controller }
    }

    fn controller(&self) -> Result<MutexGuard<'_, MonitorController>> {
        self.controller
            .lock()
            .map_err(|e| OpenDogError::LockPoisoned(format!("MonitorController: {}", e)))
    }
}

impl ProjectLifecycle for DirectProjectLifecycle<'_> {
    fn create_project(&self, id: &str, path: &str) -> Result<ProjectInfo> {
        let inner = self.controller()?;
        inner
            .project_manager()
            .create(id, std::path::Path::new(path))
    }

    fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        let inner = self.controller()?;
        inner.list_projects()
    }

    fn delete_project(&self, id: &str) -> Result<bool> {
        let mut inner = self.controller()?;
        inner.delete_project(id)
    }
}

impl SnapshotMonitor for DirectProjectLifecycle<'_> {
    fn take_snapshot(&self, id: &str) -> Result<SnapshotResult> {
        let inner = self.controller()?;
        inner.take_snapshot(id)
    }

    fn start_monitor(&self, id: &str) -> Result<StartMonitorOutcome> {
        let mut inner = self.controller()?;
        inner.start_monitor(id)
    }

    fn stop_monitor(&self, id: &str) -> Result<bool> {
        let mut inner = self.controller()?;
        Ok(inner.stop_monitor(id))
    }
}

impl Guidance for DirectProjectLifecycle<'_> {
    fn get_agent_guidance(&self, project: Option<&str>, top: usize) -> Result<Value> {
        let inner = self.controller()?;
        inner.get_agent_guidance(project, top)
    }

    fn get_decision_brief(
        &self,
        schema_version: &str,
        project: Option<&str>,
        top: usize,
    ) -> Result<Value> {
        let inner = self.controller()?;
        inner.get_decision_brief(schema_version, project, top)
    }
}
