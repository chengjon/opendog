use super::*;
use crate::config::{ProjectConfig, ProjectConfigOverrides};
use crate::core::export::{ExportFormat, ExportView, PortableProjectExport};
use crate::core::stats::ProjectSummary;

fn sample_config() -> ProjectConfig {
    ProjectConfig {
        ignore_patterns: vec!["*.log".to_string()],
        process_whitelist: vec!["claude".to_string()],
        ..Default::default()
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
            ..Default::default()
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
fn build_info_payload_keeps_contract_and_storage_schema_versions_separate() {
    let mut input = sample_build_info_input();
    input.schema_version = "1.0";
    input.build_time = "2026-01-01";
    let payload = build_info_payload(input);
    assert_eq!(payload["schema_version"], "1.0");
    assert_eq!(payload["storage_schema_version"], json!(SCHEMA_VERSION));
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
