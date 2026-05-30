use super::*;
use crate::storage::queries::VerificationRun;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

const NOW: i64 = 1_700_000_000;

fn current_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn make_run(kind: &str, status: &str, finished_at: &str) -> VerificationRun {
    VerificationRun {
        id: 1,
        kind: kind.to_string(),
        status: status.to_string(),
        command: format!("run-{}", kind),
        exit_code: Some(0),
        summary: Some(format!("{} summary", kind)),
        source: "test".to_string(),
        started_at: Some(finished_at.to_string()),
        finished_at: finished_at.to_string(),
    }
}

mod gate_assessment;
mod gate_basics;
mod gate_reasons;
mod status_layer;
mod workspace_layer;
