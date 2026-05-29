use super::*;

// ---- register_project_guidance ----

#[test]
fn register_project_guidance_has_schema_version() {
    let guidance = register_project_guidance();
    assert!(guidance["schema_version"].is_string());
    assert!(!guidance["schema_version"].as_str().unwrap().is_empty());
}

#[test]
fn register_project_guidance_has_summary() {
    let guidance = register_project_guidance();
    assert!(guidance["summary"].is_string());
    assert!(guidance["summary"].as_str().unwrap().contains("registered"));
}

#[test]
fn register_project_guidance_has_suggested_commands() {
    let guidance = register_project_guidance();
    let commands = guidance["suggested_commands"].as_array().unwrap();
    assert!(!commands.is_empty());
}

#[test]
fn register_project_guidance_has_next_tools() {
    let guidance = register_project_guidance();
    let tools = guidance["next_tools"].as_array().unwrap();
    assert!(tools.iter().any(|t| t == "start_monitor"));
    assert!(tools.iter().any(|t| t == "take_snapshot"));
}

#[test]
fn register_project_guidance_has_recommended_flow() {
    let guidance = register_project_guidance();
    let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
        .as_array()
        .unwrap();
    assert!(!flow.is_empty());
    assert!(flow
        .iter()
        .any(|s| s.as_str().unwrap().contains("registered")));
}

#[test]
fn register_project_guidance_has_when_to_use_shell() {
    let guidance = register_project_guidance();
    assert!(guidance["when_to_use_shell"].is_string());
}

// ---- snapshot_guidance ----

#[test]
fn snapshot_guidance_zero_files() {
    let guidance = snapshot_guidance(0);
    assert!(guidance["summary"].as_str().unwrap().contains("no files"));
    let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
        .as_array()
        .unwrap();
    assert!(flow
        .iter()
        .any(|s| s.as_str().unwrap().contains("zero files")));
}

#[test]
fn snapshot_guidance_zero_files_next_tools() {
    let guidance = snapshot_guidance(0);
    let tools = guidance["next_tools"].as_array().unwrap();
    assert!(tools.iter().any(|t| t == "list_projects"));
    assert!(tools.iter().any(|t| t == "take_snapshot"));
}

#[test]
fn snapshot_guidance_with_files() {
    let guidance = snapshot_guidance(42);
    assert!(guidance["summary"]
        .as_str()
        .unwrap()
        .contains("Snapshot complete"));
}

#[test]
fn snapshot_guidance_with_files_recommends_stats() {
    let guidance = snapshot_guidance(100);
    let tools = guidance["next_tools"].as_array().unwrap();
    assert!(tools.iter().any(|t| t == "get_stats"));
    assert!(tools.iter().any(|t| t == "get_unused_files"));
}

#[test]
fn snapshot_guidance_with_files_has_recommended_flow() {
    let guidance = snapshot_guidance(10);
    let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
        .as_array()
        .unwrap();
    assert!(flow
        .iter()
        .any(|s| s.as_str().unwrap().contains("baseline")));
}

#[test]
fn snapshot_guidance_with_files_has_shell_guidance() {
    let guidance = snapshot_guidance(10);
    assert!(guidance["when_to_use_shell"].is_string());
}

// ---- start_monitor_guidance ----

#[test]
fn start_monitor_guidance_already_running() {
    let guidance = start_monitor_guidance(true, false);
    assert!(guidance["summary"]
        .as_str()
        .unwrap()
        .contains("already active"));
    let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
        .as_array()
        .unwrap();
    assert!(flow
        .iter()
        .any(|s| s.as_str().unwrap().contains("already active")));
}

#[test]
fn start_monitor_guidance_already_running_recommends_stats() {
    let guidance = start_monitor_guidance(true, false);
    let tools = guidance["next_tools"].as_array().unwrap();
    assert!(tools.iter().any(|t| t == "get_stats"));
}

#[test]
fn start_monitor_guidance_not_running_with_snapshot() {
    let guidance = start_monitor_guidance(false, true);
    assert!(guidance["summary"]
        .as_str()
        .unwrap()
        .contains("initial snapshot"));
    let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
        .as_array()
        .unwrap();
    assert!(flow
        .iter()
        .any(|s| s.as_str().unwrap().contains("baseline snapshot")));
}

#[test]
fn start_monitor_guidance_not_running_with_snapshot_recommends_activity() {
    let guidance = start_monitor_guidance(false, true);
    let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
        .as_array()
        .unwrap();
    assert!(flow
        .iter()
        .any(|s| s.as_str().unwrap().contains("real project activity")));
}

#[test]
fn start_monitor_guidance_not_running_no_snapshot() {
    let guidance = start_monitor_guidance(false, false);
    assert!(guidance["summary"]
        .as_str()
        .unwrap()
        .contains("Monitoring is active"));
    let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
        .as_array()
        .unwrap();
    assert!(flow
        .iter()
        .any(|s| s.as_str().unwrap().contains("activity-derived")));
}

#[test]
fn start_monitor_guidance_not_running_no_snapshot_recommends_stats() {
    let guidance = start_monitor_guidance(false, false);
    let tools = guidance["next_tools"].as_array().unwrap();
    assert!(tools.iter().any(|t| t == "get_stats"));
    assert!(tools.iter().any(|t| t == "get_unused_files"));
}

#[test]
fn start_monitor_guidance_all_paths_have_schema_version() {
    for (running, snapshot) in [(true, true), (true, false), (false, true), (false, false)] {
        let guidance = start_monitor_guidance(running, snapshot);
        assert!(guidance["schema_version"].is_string());
    }
}
