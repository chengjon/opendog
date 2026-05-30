use super::transport::map_connect_error_with_liveness;
use super::*;
use crate::config::{ConfigPatch, ProjectConfigPatch};
use crate::control::client::decode_control_response;
use crate::core::governance::{
    CloseLaneInput, CreateLaneInput, GetGovernanceStateInput, UpsertNodeInput,
};
use crate::core::orphan::{DeletionPlanInput, OrphanSubject, OrphanSubjectKind, ScanOrphansInput};
use crate::error::OpenDogError;
use serde_json::json;
use tempfile::TempDir;

fn test_controller() -> (TempDir, MonitorController) {
    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path().join("data");
    let project_root = dir.path().join("project");
    std::fs::create_dir_all(&project_root).unwrap();
    let pm = ProjectManager::with_data_dir(&data_dir).unwrap();
    pm.create("demo", &project_root).unwrap();
    (dir, MonitorController::with_project_manager(pm))
}

#[path = "tests/config_updates.rs"]
mod config_updates;
#[path = "tests/decision_and_data_risk.rs"]
mod decision_and_data_risk;
#[path = "tests/governance_orphans.rs"]
mod governance_orphans;
#[path = "tests/monitor_lifecycle.rs"]
mod monitor_lifecycle;
#[path = "tests/transport_and_basics.rs"]
mod transport_and_basics;
