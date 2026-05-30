use super::*;
use crate::storage::database::Database;

fn test_db() -> Database {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("governance_core_test.db");
    let db = Database::open_project(&db_path).unwrap();
    Box::leak(Box::new(dir));
    db
}

mod close_lane;
mod lane_flow;
mod observation_hints;
mod serialization_helpers;
mod state_queries;
