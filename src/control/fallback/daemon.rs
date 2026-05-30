use crate::config::ProjectInfo;
use crate::control::protocol::StartMonitorOutcome;
use crate::control::DaemonClient;
use crate::core::snapshot::SnapshotResult;
use crate::error::Result;
use serde_json::Value;

use super::{Guidance, ProjectLifecycle, SnapshotMonitor};

pub struct DaemonProjectLifecycle<'a> {
    client: &'a DaemonClient,
}

impl<'a> DaemonProjectLifecycle<'a> {
    pub fn new(client: &'a DaemonClient) -> Self {
        Self { client }
    }
}

impl ProjectLifecycle for DaemonProjectLifecycle<'_> {
    fn create_project(&self, id: &str, path: &str) -> Result<ProjectInfo> {
        self.client.create_project(id, path)
    }

    fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        self.client.list_projects()
    }

    fn delete_project(&self, id: &str) -> Result<bool> {
        self.client.delete_project(id)
    }
}

impl SnapshotMonitor for DaemonProjectLifecycle<'_> {
    fn take_snapshot(&self, id: &str) -> Result<SnapshotResult> {
        self.client.take_snapshot(id)
    }

    fn start_monitor(&self, id: &str) -> Result<StartMonitorOutcome> {
        self.client.start_monitor(id)
    }

    fn stop_monitor(&self, id: &str) -> Result<bool> {
        self.client.stop_monitor(id)
    }
}

impl Guidance for DaemonProjectLifecycle<'_> {
    fn get_agent_guidance(&self, project: Option<&str>, top: usize) -> Result<Value> {
        self.client.get_agent_guidance(project, top)
    }

    fn get_decision_brief(
        &self,
        schema_version: &str,
        project: Option<&str>,
        top: usize,
    ) -> Result<Value> {
        self.client.get_decision_brief(project, top, schema_version)
    }
}
