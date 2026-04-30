use serde_json::{json, Value};

use crate::config::{
    GlobalConfigUpdateResult, ProjectConfig, ProjectConfigReload, ProjectConfigUpdateResult,
    ProjectConfigView,
};
use crate::contracts::{versioned_payload, versioned_project_payload};
use crate::core::stats::ProjectSummary;

use super::super::tool_guidance;

pub(crate) fn global_config_payload(schema_version: &str, config: &ProjectConfig) -> Value {
    versioned_payload(
        schema_version,
        [
            ("global_defaults", json!(config)),
            (
                "guidance",
                tool_guidance(
                    "Global defaults loaded. Use project config to inspect overrides or update defaults before reloading active monitors.",
                    &[
                        "opendog config show --id <project>",
                        "opendog config set-global --ignore-pattern <pattern>",
                        "opendog config reload --id <project>",
                    ],
                    &["get_project_config", "update_global_config", "reload_project_config"],
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
                    "Project config loaded. Update overrides or reload a running monitor if runtime state must pick up persisted changes.",
                    &[
                        "opendog config set-project --id <project> --ignore-pattern <pattern>",
                        "opendog config reload --id <project>",
                    ],
                    &["update_project_config", "reload_project_config"],
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
                    &["reload_project_config", "get_project_config"],
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
                    &["reload_project_config", "start_monitor"],
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
    id: &str,
    format: &str,
    view: &str,
    output_path: &str,
    bytes_written: u64,
    row_count: usize,
    summary: &ProjectSummary,
    content: &str,
) -> Value {
    versioned_project_payload(
        schema_version,
        id,
        [
            ("status", json!("exported")),
            ("format", json!(format)),
            ("view", json!(view)),
            ("output_path", json!(output_path)),
            ("bytes_written", json!(bytes_written)),
            ("row_count", json!(row_count)),
            ("summary", json!(summary)),
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
