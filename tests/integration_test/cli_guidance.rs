#[path = "cli_guidance/agent_guidance_text.rs"]
mod agent_guidance_text;
#[path = "cli_guidance/decision_brief_text.rs"]
mod decision_brief_text;
#[path = "cli_guidance/json_outputs.rs"]
mod json_outputs;

use serde_json::Value;

pub(super) use super::common::run_cli;

pub(super) fn run_cli_json(home: &std::path::Path, args: &[&str]) -> Value {
    let output = run_cli(home, args);
    assert!(output.status.success(), "{:?}", output);
    serde_json::from_slice(&output.stdout).unwrap()
}
