use serde_json::{json, Value};

use crate::config::{
    GlobalConfigUpdateResult, ProjectConfig, ProjectConfigReload, ProjectConfigUpdateResult,
    ProjectConfigView,
};
use crate::contracts::{versioned_payload, versioned_project_payload};
use crate::core::export::PortableProjectExport;
use crate::storage::schema::SCHEMA_VERSION;

use super::super::tool_guidance;

pub(crate) struct BuildInfoPayloadInput<'a> {
    pub schema_version: &'a str,
    pub version: &'a str,
    pub git_hash: &'a str,
    pub build_time: &'a str,
    pub binary_path: &'a str,
    pub needs_rebuild: Option<bool>,
    pub daemon_running: bool,
    pub opendog_home: &'a str,
}

pub(crate) fn build_info_payload(input: BuildInfoPayloadInput<'_>) -> Value {
    let rebuild_hint = match input.needs_rebuild {
        Some(true) => Some(
            "Running binary is older than source code. Run `opendog self-update build --source /opt/claude/opendog`, then restart this MCP session."
                .to_string(),
        ),
        _ => None,
    };

    let mut fields: Vec<(&str, Value)> = vec![
        ("version", json!(input.version)),
        ("schema_version", json!(SCHEMA_VERSION)),
        ("git_hash", json!(input.git_hash)),
        ("build_time", json!(input.build_time)),
        ("binary_path", json!(input.binary_path)),
        ("needs_rebuild", json!(input.needs_rebuild)),
        ("daemon_running", json!(input.daemon_running)),
        ("opendog_home", json!(input.opendog_home)),
    ];
    if let Some(hint) = &rebuild_hint {
        fields.push(("rebuild_hint", json!(hint)));
    }
    fields.push((
        "guidance",
        tool_guidance(
            if input.needs_rebuild == Some(true) {
                "Binary is stale — rebuild and restart to pick up latest changes."
            } else {
                "Build info loaded. Binary is up to date."
            },
            &["opendog self-update status --source /opt/claude/opendog"],
            &[],
            None,
        ),
    ));
    versioned_payload(input.schema_version, fields)
}

pub(crate) fn global_config_payload(schema_version: &str, config: &ProjectConfig) -> Value {
    versioned_payload(
        schema_version,
        [
            ("global_defaults", json!(config)),
            (
                "guidance",
                tool_guidance(
                    "Global defaults loaded. Use project config to inspect overrides, then switch to CLI config commands for mutations or runtime reload.",
                    &[
                        "opendog config show --id <project>",
                        "opendog config set-global --ignore-pattern <pattern>",
                        "opendog config reload --id <project>",
                    ],
                    &["get_project_config"],
                    Some(
                        "Use shell edits only if you intentionally want to bypass OPENDOG-managed config persistence.",
                    ),
                ),
            ),
        ],
    )
}

pub(crate) fn project_config_payload(schema_version: &str, view: &ProjectConfigView) -> Value {
    versioned_project_payload(
        schema_version,
        &view.project_id,
        [
            ("global_defaults", json!(view.global_defaults)),
            ("project_overrides", json!(view.project_overrides)),
            ("effective", json!(view.effective)),
            (
                "inherits",
                json!({
                    "ignore_patterns": view.project_overrides.ignore_patterns.is_none(),
                    "process_whitelist": view.project_overrides.process_whitelist.is_none(),
                }),
            ),
            (
                "guidance",
                tool_guidance(
                    "Project config loaded. Use CLI config commands for override changes or monitor reload when runtime state must pick up persisted values.",
                    &[
                        "opendog config set-project --id <project> --ignore-pattern <pattern>",
                        "opendog config reload --id <project>",
                    ],
                    &["get_project_config"],
                    Some(
                        "Use OPENDOG config tools instead of manual registry edits so precedence and reload behavior stay explicit.",
                    ),
                ),
            ),
        ],
    )
}

pub(crate) fn update_global_config_payload(
    schema_version: &str,
    result: &GlobalConfigUpdateResult,
) -> Value {
    versioned_payload(
        schema_version,
        [
            ("status", json!("updated")),
            ("global_defaults", json!(result.global_defaults)),
            ("reloaded_projects", json!(result.reloaded_projects)),
            (
                "guidance",
                tool_guidance(
                    "Global defaults updated. Review which running monitors reloaded automatically and reload any remaining projects if needed.",
                    &[
                        "opendog config reload --id <project>",
                        "opendog config show --id <project>",
                    ],
                    &["get_project_config"],
                    Some(
                        "Use shell verification after config changes if cleanup or monitoring scope is being narrowed materially.",
                    ),
                ),
            ),
        ],
    )
}

pub(crate) fn project_config_update_payload(
    schema_version: &str,
    result: &ProjectConfigUpdateResult,
) -> Value {
    versioned_project_payload(
        schema_version,
        &result.project_id,
        [
            ("status", json!("updated")),
            ("global_defaults", json!(result.global_defaults)),
            ("project_overrides", json!(result.project_overrides)),
            ("effective", json!(result.effective)),
            ("reload", json!(result.reload)),
            (
                "guidance",
                tool_guidance(
                    "Project override updated. Inspect the reload block to confirm whether runtime monitor state changed immediately or will apply on next start.",
                    &[
                        "opendog config reload --id <project>",
                        "opendog start --id <project>",
                    ],
                    &["get_project_config", "start_monitor"],
                    Some(
                        "Do not assume a running monitor picked up changes unless the reload block reports runtime_reloaded=true.",
                    ),
                ),
            ),
        ],
    )
}

pub(crate) fn project_config_reload_payload(
    schema_version: &str,
    id: &str,
    reload: &ProjectConfigReload,
    effective: &ProjectConfig,
) -> Value {
    versioned_project_payload(
        schema_version,
        id,
        [
            ("status", json!("reloaded")),
            ("reload", json!(reload)),
            ("effective", json!(effective)),
            (
                "guidance",
                tool_guidance(
                    "Reload completed. Confirm changed fields and snapshot refresh behavior before assuming monitor scope changed.",
                    &[
                        "opendog config show --id <project>",
                        "opendog stats --id <project>",
                    ],
                    &["get_project_config", "get_stats"],
                    Some(
                        "Use shell verification if config changes were intended to exclude or include risky directories.",
                    ),
                ),
            ),
        ],
    )
}

pub(crate) fn export_project_evidence_payload(
    schema_version: &str,
    artifact: &PortableProjectExport,
    output_path: &str,
    bytes_written: u64,
    content: &str,
) -> Value {
    versioned_project_payload(
        schema_version,
        &artifact.project_id,
        [
            ("status", json!("exported")),
            ("format", json!(artifact.format.as_str())),
            ("view", json!(artifact.view.as_str())),
            ("output_path", json!(output_path)),
            ("bytes_written", json!(bytes_written)),
            ("row_count", json!(artifact.row_count)),
            ("summary", json!(&artifact.summary)),
            ("content", json!(content)),
            (
                "guidance",
                tool_guidance(
                    "Portable export written. Prefer consuming the file artifact rather than scraping formatted terminal output.",
                    &[
                        "Inspect the written JSON/CSV artifact directly",
                        "Use `opendog stats --id <project>` when you need a live terminal summary instead of an artifact",
                    ],
                    &["get_stats", "get_unused_files"],
                    Some(
                        "Use export artifacts for downstream automation or review handoff, not as a substitute for shell verification.",
                    ),
                ),
            ),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ProjectConfig, ProjectConfigOverrides};
    use crate::core::export::{ExportFormat, ExportView, PortableProjectExport};
    use crate::core::stats::ProjectSummary;

    fn sample_config() -> ProjectConfig {
        ProjectConfig {
            ignore_patterns: vec!["*.log".to_string()],
            process_whitelist: vec!["claude".to_string()],
        }
    }

    fn sample_build_info_input() -> BuildInfoPayloadInput<'static> {
        BuildInfoPayloadInput {
            schema_version: "v1",
            version: "0.1.0",
            git_hash: "abc123",
            build_time: "2025-01-01",
            binary_path: "/usr/bin/opendog",
            needs_rebuild: None,
            daemon_running: false,
            opendog_home: "/home/user/.opendog",
        }
    }

    // --- build_info_payload ---

    #[test]
    fn build_info_payload_basic() {
        let result = build_info_payload(sample_build_info_input());
        assert_eq!(result["version"], "0.1.0");
        assert_eq!(result["git_hash"], "abc123");
        assert_eq!(result["build_time"], "2025-01-01");
        assert_eq!(result["binary_path"], "/usr/bin/opendog");
        assert!(result["needs_rebuild"].is_null());
        assert!(result.get("rebuild_hint").is_none() || result["rebuild_hint"].is_null());
    }

    #[test]
    fn build_info_payload_needs_rebuild_true() {
        let mut input = sample_build_info_input();
        input.git_hash = "abc";
        input.binary_path = "/bin/od";
        input.needs_rebuild = Some(true);
        let result = build_info_payload(input);
        assert_eq!(result["needs_rebuild"], true);
        assert!(result["rebuild_hint"]
            .as_str()
            .unwrap()
            .contains("self-update"));
    }

    #[test]
    fn build_info_payload_needs_rebuild_false() {
        let mut input = sample_build_info_input();
        input.git_hash = "abc";
        input.binary_path = "/bin/od";
        input.needs_rebuild = Some(false);
        let result = build_info_payload(input);
        assert_eq!(result["needs_rebuild"], false);
        assert!(result.get("rebuild_hint").is_none() || result["rebuild_hint"].is_null());
    }

    // --- global_config_payload ---

    #[test]
    fn global_config_payload_contains_config() {
        let config = sample_config();
        let result = global_config_payload("v1", &config);
        assert!(result["global_defaults"].is_object());
        assert!(result["guidance"].is_object());
    }

    // --- project_config_payload ---

    #[test]
    fn project_config_payload_fields() {
        let view = ProjectConfigView {
            project_id: "p1".to_string(),
            global_defaults: sample_config(),
            project_overrides: ProjectConfigOverrides {
                ignore_patterns: Some(vec!["target/".to_string()]),
                process_whitelist: None,
            },
            effective: sample_config(),
        };
        let result = project_config_payload("v1", &view);
        assert_eq!(result["project_id"], "p1");
        assert!(result["global_defaults"].is_object());
        assert!(result["project_overrides"].is_object());
        assert!(result["effective"].is_object());
        assert_eq!(result["inherits"]["ignore_patterns"], false);
        assert_eq!(result["inherits"]["process_whitelist"], true);
    }

    // --- update_global_config_payload ---

    #[test]
    fn update_global_config_payload_fields() {
        let config = sample_config();
        let update = GlobalConfigUpdateResult {
            global_defaults: config,
            reloaded_projects: vec![],
        };
        let result = update_global_config_payload("v1", &update);
        assert_eq!(result["status"], "updated");
        assert!(result["global_defaults"].is_object());
        assert!(result["reloaded_projects"].as_array().unwrap().is_empty());
    }

    // --- project_config_update_payload ---

    #[test]
    fn project_config_update_payload_fields() {
        let config = sample_config();
        let update = ProjectConfigUpdateResult {
            project_id: "p2".to_string(),
            global_defaults: config.clone(),
            project_overrides: ProjectConfigOverrides::default(),
            effective: config,
            reload: ProjectConfigReload {
                monitor_running: true,
                runtime_reloaded: false,
                snapshot_refreshed: false,
                changed_fields: vec!["ignore_patterns".to_string()],
                skipped_fields: vec![],
            },
        };
        let result = project_config_update_payload("v1", &update);
        assert_eq!(result["project_id"], "p2");
        assert_eq!(result["status"], "updated");
        assert_eq!(result["reload"]["monitor_running"], true);
        assert_eq!(
            result["reload"]["changed_fields"].as_array().unwrap().len(),
            1
        );
    }

    // --- project_config_reload_payload ---

    #[test]
    fn project_config_reload_payload_fields() {
        let config = sample_config();
        let reload = ProjectConfigReload {
            monitor_running: true,
            runtime_reloaded: true,
            snapshot_refreshed: false,
            changed_fields: vec![],
            skipped_fields: vec![],
        };
        let result = project_config_reload_payload("v1", "p3", &reload, &config);
        assert_eq!(result["project_id"], "p3");
        assert_eq!(result["status"], "reloaded");
        assert_eq!(result["reload"]["runtime_reloaded"], true);
    }

    // --- export_project_evidence_payload ---

    #[test]
    fn export_payload_fields() {
        let artifact = PortableProjectExport {
            schema_version: "v1".to_string(),
            project_id: "exp1".to_string(),
            format: ExportFormat::Json.as_str().to_string(),
            view: ExportView::Stats.as_str().to_string(),
            generated_at: "2025-06-01".to_string(),
            summary: ProjectSummary {
                total_files: 50,
                accessed_files: 30,
                unused_files: 20,
            },
            row_count: 50,
            rows: vec![],
        };
        let result =
            export_project_evidence_payload("v1", &artifact, "/tmp/out.json", 1024, "content-here");
        assert_eq!(result["project_id"], "exp1");
        assert_eq!(result["status"], "exported");
        assert_eq!(result["format"], "json");
        assert_eq!(result["view"], "stats");
        assert_eq!(result["output_path"], "/tmp/out.json");
        assert_eq!(result["bytes_written"], 1024);
        assert_eq!(result["row_count"], 50);
        assert_eq!(result["content"], "content-here");
        assert!(result["guidance"].is_object());
    }

    #[test]
    fn build_info_payload_includes_schema_version() {
        let mut input = sample_build_info_input();
        input.schema_version = "1.0";
        input.build_time = "2026-01-01";
        let payload = build_info_payload(input);
        assert_eq!(payload["schema_version"], 6);
        assert_eq!(payload["version"], "0.1.0");
    }

    #[test]
    fn build_info_payload_preserves_existing_fields() {
        let mut input = sample_build_info_input();
        input.schema_version = "1.0";
        input.build_time = "2026-01-01";
        input.needs_rebuild = Some(true);
        let payload = build_info_payload(input);
        assert_eq!(payload["version"], "0.1.0");
        assert_eq!(payload["git_hash"], "abc123");
        assert_eq!(payload["needs_rebuild"], true);
        assert!(payload["rebuild_hint"].is_string());
    }

    #[test]
    fn build_info_payload_includes_daemon_running() {
        let mut input = sample_build_info_input();
        input.schema_version = "1.0";
        input.git_hash = "abc";
        input.build_time = "2026-01-01";
        input.daemon_running = true;
        let payload = build_info_payload(input);
        assert_eq!(payload["daemon_running"], true);
        assert_eq!(payload["opendog_home"], "/home/user/.opendog");
    }

    #[test]
    fn build_info_payload_daemon_not_running() {
        let mut input = sample_build_info_input();
        input.schema_version = "1.0";
        input.git_hash = "abc";
        input.build_time = "2026-01-01";
        input.opendog_home = "/var/lib/opendog";
        let payload = build_info_payload(input);
        assert_eq!(payload["daemon_running"], false);
        assert_eq!(payload["opendog_home"], "/var/lib/opendog");
    }
}
