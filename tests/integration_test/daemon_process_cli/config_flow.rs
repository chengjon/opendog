use opendog::contracts::{
    CLI_GLOBAL_CONFIG_V1, CLI_PROJECT_CONFIG_V1, CLI_RELOAD_PROJECT_CONFIG_V1,
    CLI_UPDATE_GLOBAL_CONFIG_V1, CLI_UPDATE_PROJECT_CONFIG_V1,
};
use serde_json::Value;
use std::path::Path;

use crate::common::run_cli;

pub fn assert_config_reload_flow(home: &Path) {
    let start_again = run_cli(home, &["start", "--id", "demo"]);
    assert!(start_again.status.success(), "{:?}", start_again);
    assert!(String::from_utf8_lossy(&start_again.stdout).contains("already running"));

    let global_config = run_cli(home, &["config", "show", "--json"]);
    assert!(global_config.status.success(), "{:?}", global_config);
    let global_config_json: Value = serde_json::from_slice(&global_config.stdout).unwrap();
    assert_eq!(
        global_config_json["schema_version"].as_str(),
        Some(CLI_GLOBAL_CONFIG_V1)
    );
    assert!(global_config_json["global_defaults"]["ignore_patterns"].is_array());

    let project_config = run_cli(home, &["config", "show", "--id", "demo", "--json"]);
    assert!(project_config.status.success(), "{:?}", project_config);
    let project_config_json: Value = serde_json::from_slice(&project_config.stdout).unwrap();
    assert_eq!(
        project_config_json["schema_version"].as_str(),
        Some(CLI_PROJECT_CONFIG_V1)
    );
    assert_eq!(project_config_json["project_id"], "demo");
    assert!(project_config_json["effective"]["process_whitelist"].is_array());

    let overwrite_global_config = run_cli(
        home,
        &[
            "config",
            "set-global",
            "--process",
            "claude",
            "--process",
            "codex",
            "--ignore-pattern",
            "dist",
            "--json",
        ],
    );
    assert!(
        overwrite_global_config.status.success(),
        "{:?}",
        overwrite_global_config
    );
    let overwrite_global_config_json: Value =
        serde_json::from_slice(&overwrite_global_config.stdout).unwrap();
    assert_eq!(
        overwrite_global_config_json["schema_version"].as_str(),
        Some(CLI_UPDATE_GLOBAL_CONFIG_V1)
    );
    assert_eq!(
        overwrite_global_config_json["global_defaults"]["process_whitelist"],
        serde_json::json!(["claude", "codex"])
    );
    assert_eq!(
        overwrite_global_config_json["global_defaults"]["ignore_patterns"],
        serde_json::json!(["dist"])
    );

    let update_project_config = run_cli(
        home,
        &[
            "config",
            "set-project",
            "--id",
            "demo",
            "--remove-process",
            "claude",
            "--add-process",
            "roo",
            "--add-ignore-pattern",
            "logs",
            "--json",
        ],
    );
    assert!(
        update_project_config.status.success(),
        "daemon incremental project update failed: {}",
        String::from_utf8_lossy(&update_project_config.stderr)
    );
    let update_project_config_json: Value =
        serde_json::from_slice(&update_project_config.stdout).unwrap();
    assert_eq!(
        update_project_config_json["schema_version"].as_str(),
        Some(CLI_UPDATE_PROJECT_CONFIG_V1)
    );
    assert_eq!(
        update_project_config_json["project_overrides"]["process_whitelist"],
        serde_json::json!(["codex", "roo"])
    );
    assert_eq!(
        update_project_config_json["effective"]["ignore_patterns"],
        serde_json::json!(["dist", "logs"])
    );
    assert_eq!(
        update_project_config_json["reload"]["runtime_reloaded"].as_bool(),
        Some(true)
    );

    let update_global_config = run_cli(
        home,
        &[
            "config",
            "set-global",
            "--ignore-pattern",
            "generated",
            "--process",
            "claude",
            "--json",
        ],
    );
    assert!(
        update_global_config.status.success(),
        "{:?}",
        update_global_config
    );
    let update_global_config_json: Value =
        serde_json::from_slice(&update_global_config.stdout).unwrap();
    assert_eq!(
        update_global_config_json["schema_version"].as_str(),
        Some(CLI_UPDATE_GLOBAL_CONFIG_V1)
    );
    assert!(update_global_config_json["reloaded_projects"].is_array());

    let reload_project_config = run_cli(home, &["config", "reload", "--id", "demo", "--json"]);
    assert!(
        reload_project_config.status.success(),
        "{:?}",
        reload_project_config
    );
    let reload_project_config_json: Value =
        serde_json::from_slice(&reload_project_config.stdout).unwrap();
    assert_eq!(
        reload_project_config_json["schema_version"].as_str(),
        Some(CLI_RELOAD_PROJECT_CONFIG_V1)
    );
    assert_eq!(reload_project_config_json["project_id"], "demo");
    assert!(reload_project_config_json["reload"].is_object());
}
