use serde_json::{json, Value};

use crate::config::RetentionPolicy;

use super::super::model::{StorageCleanupScope, StorageMaintenanceTemplateContext};

pub(super) fn all_scope_preview_template(
    project_id_value: &str,
    project_placeholder_hint: &Value,
) -> Value {
    json!({
        "template_id": "storage.cleanup.preview",
        "kind": "cli_command",
        "command_template": format!(
            "opendog cleanup-data --id {} --scope all --older-than-days 30 --dry-run --json",
            project_id_value
        ),
        "preconditions": [
            "project must exist in OPENDOG",
            "use dry-run first before deleting retained OPENDOG evidence"
        ],
        "blocking_conditions": [],
        "success_signal": "cleanup preview returns deleted counts plus storage_before metrics",
        "parameter_schema": {
            "id": { "type": "string", "required": true, "source": "project_id" },
            "scope": { "type": "enum", "required": true, "allowed_values": ["activity", "snapshots", "verification", "all"] },
            "older_than_days": { "type": "integer", "required": false },
            "dry_run": { "type": "boolean", "required": false },
            "json": { "type": "boolean", "required": false }
        },
        "default_values": {
            "scope": "all",
            "older_than_days": 30,
            "dry_run": true,
            "json": true
        },
        "placeholder_hints": project_placeholder_hint.clone(),
        "priority": 1,
        "should_run_if": [
            "run when OPENDOG project-db size or reclaimable pages suggest retention maintenance"
        ],
        "skip_if": [
            "skip if project database is still small and reclaimable space is negligible"
        ],
        "expected_output_fields": [
            "deleted",
            "storage_before.approx_db_size_bytes",
            "storage_before.approx_reclaimable_bytes",
            "maintenance"
        ],
        "follow_up_on_success": [
            "if deleted counts are meaningful, ask whether to rerun cleanup without dry_run",
            "if reclaimable bytes remain high, consider a follow-up vacuum pass"
        ],
        "follow_up_on_failure": [
            "fallback to project-scoped stats and verification review if cleanup preview is unavailable"
        ],
        "plan_stage": "maintain",
        "terminality": "non_terminal",
        "can_run_in_parallel": false,
        "requires_human_confirmation": false,
        "evidence_written_to_opendog": false,
        "retry_policy": {
            "allowed": true,
            "max_attempts": 2,
            "strategy": "rerun_after_scope_or_retention_parameter_adjustment",
            "retry_when": ["project id or cleanup scope was corrected"]
        }
    })
}

pub(super) fn append_scope_preview_templates(
    templates: &mut Vec<Value>,
    context: &StorageMaintenanceTemplateContext,
    project_placeholder_hint: &Value,
    default_policy: &RetentionPolicy,
    next_priority: &mut usize,
) {
    for recommendation in &context.cleanup_recommendations {
        let Some(template) = scope_preview_template(
            context.project_id_value(),
            recommendation.scope,
            recommendation.older_than_days_or_default(default_policy),
            recommendation.keep_snapshot_runs_or_default(default_policy),
            project_placeholder_hint,
            *next_priority,
        ) else {
            continue;
        };
        templates.push(template);
        *next_priority += 1;
    }
}

fn scope_preview_template(
    project_id_value: &str,
    scope: StorageCleanupScope,
    older_than_days: i64,
    keep_snapshot_runs: i64,
    project_placeholder_hint: &Value,
    priority: usize,
) -> Option<Value> {
    let scope_name = scope.as_str();
    let (command_template, default_values, success_signal) = match scope {
        StorageCleanupScope::Activity | StorageCleanupScope::Verification => {
            (
                format!(
                    "opendog cleanup-data --id {} --scope {} --older-than-days {} --dry-run --json",
                    project_id_value, scope_name, older_than_days
                ),
                json!({
                    "scope": scope_name,
                    "older_than_days": older_than_days,
                    "dry_run": true,
                    "json": true
                }),
                format!(
                    "{} cleanup preview returns deleted counts plus storage_before metrics",
                    scope_name
                ),
            )
        }
        StorageCleanupScope::Snapshots => {
            (
                format!(
                    "opendog cleanup-data --id {} --scope snapshots --keep-snapshot-runs {} --dry-run --json",
                    project_id_value, keep_snapshot_runs
                ),
                json!({
                    "scope": "snapshots",
                    "keep_snapshot_runs": keep_snapshot_runs,
                    "dry_run": true,
                    "json": true
                }),
                "snapshots cleanup preview returns deleted counts plus storage_before metrics"
                    .to_string(),
            )
        }
        StorageCleanupScope::All => return None,
    };

    Some(json!({
        "template_id": format!("storage.cleanup.{}.preview", scope_name),
        "kind": "cli_command",
        "command_template": command_template,
        "preconditions": [
            "project must exist in OPENDOG",
            "use dry-run first before deleting retained OPENDOG evidence"
        ],
        "blocking_conditions": [],
        "success_signal": success_signal,
        "parameter_schema": {
            "id": { "type": "string", "required": true, "source": "project_id" },
            "scope": { "type": "enum", "required": true, "allowed_values": ["activity", "snapshots", "verification"] },
            "older_than_days": { "type": "integer", "required": false },
            "keep_snapshot_runs": { "type": "integer", "required": false },
            "dry_run": { "type": "boolean", "required": false },
            "json": { "type": "boolean", "required": false }
        },
        "default_values": default_values,
        "placeholder_hints": project_placeholder_hint.clone(),
        "priority": priority,
        "should_run_if": [
            "run when OPENDOG storage pressure recommendation names this cleanup scope"
        ],
        "skip_if": [
            "skip if the operator wants a single all-scope cleanup preview instead"
        ],
        "expected_output_fields": [
            "deleted",
            "storage_before.approx_db_size_bytes",
            "storage_before.approx_reclaimable_bytes",
            "maintenance"
        ],
        "follow_up_on_success": [
            "if deleted counts are meaningful, ask whether to rerun cleanup without dry_run"
        ],
        "follow_up_on_failure": [
            "fallback to the all-scope cleanup preview to compare retained evidence counts"
        ],
        "plan_stage": "maintain",
        "terminality": "non_terminal",
        "can_run_in_parallel": false,
        "requires_human_confirmation": false,
        "evidence_written_to_opendog": false,
        "retry_policy": {
            "allowed": true,
            "max_attempts": 2,
            "strategy": "rerun_after_scope_or_retention_parameter_adjustment",
            "retry_when": ["project id or cleanup scope was corrected"]
        }
    }))
}
