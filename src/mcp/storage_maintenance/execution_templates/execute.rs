use serde_json::{json, Value};

use crate::config::RetentionPolicy;

use super::super::model::{StorageCleanupScope, StorageMaintenanceTemplateContext};

pub(super) fn append_execution_step_templates(
    templates: &mut Vec<Value>,
    context: &StorageMaintenanceTemplateContext,
    project_placeholder_hint: &Value,
    default_policy: &RetentionPolicy,
    next_priority: &mut usize,
) {
    for plan_step in &context.cleanup_plan_steps {
        templates.push(execution_step_template(
            context.project_id_value(),
            plan_step.scope,
            plan_step.older_than_days_or_default(default_policy),
            plan_step.keep_snapshot_runs_or_default(default_policy),
            project_placeholder_hint,
            *next_priority,
        ));
        *next_priority += 1;
    }
}

fn execution_step_template(
    project_id_value: &str,
    scope: StorageCleanupScope,
    older_than_days: i64,
    keep_snapshot_runs: i64,
    project_placeholder_hint: &Value,
    priority: usize,
) -> Value {
    let scope_name = scope.as_str();
    let (command_template, default_values) = match scope {
        StorageCleanupScope::Activity
        | StorageCleanupScope::Verification
        | StorageCleanupScope::All => (
            format!(
                "opendog cleanup-data --id {} --scope {} --older-than-days {} --json",
                project_id_value, scope_name, older_than_days
            ),
            json!({
                "scope": scope_name,
                "older_than_days": older_than_days,
                "dry_run": false,
                "json": true
            }),
        ),
        StorageCleanupScope::Snapshots => (
            format!(
                "opendog cleanup-data --id {} --scope snapshots --keep-snapshot-runs {} --json",
                project_id_value, keep_snapshot_runs
            ),
            json!({
                "scope": "snapshots",
                "keep_snapshot_runs": keep_snapshot_runs,
                "dry_run": false,
                "json": true
            }),
        ),
    };

    json!({
        "template_id": format!("storage.cleanup.{}.execute", scope_name),
        "kind": "cli_command",
        "command_template": command_template,
        "preconditions": [
            "run only after the matching cleanup-data dry-run preview was reviewed",
            "operator must confirm retained OPENDOG evidence can be deleted"
        ],
        "blocking_conditions": [
            "requires explicit confirmation because this command deletes retained OPENDOG evidence"
        ],
        "success_signal": format!("{} cleanup execution returns deleted counts plus storage_after metrics", scope_name),
        "parameter_schema": {
            "id": { "type": "string", "required": true, "source": "project_id" },
            "scope": { "type": "enum", "required": true, "allowed_values": ["activity", "snapshots", "verification", "all"] },
            "older_than_days": { "type": "integer", "required": false },
            "keep_snapshot_runs": { "type": "integer", "required": false },
            "dry_run": { "type": "boolean", "required": false },
            "json": { "type": "boolean", "required": false }
        },
        "default_values": default_values,
        "placeholder_hints": project_placeholder_hint.clone(),
        "priority": priority,
        "should_run_if": [
            "run only when the policy cleanup plan execute_cleanup step is approved"
        ],
        "skip_if": [
            "skip if the dry-run preview was not reviewed or deleted counts look unsafe"
        ],
        "expected_output_fields": [
            "deleted",
            "storage_before.approx_db_size_bytes",
            "storage_after.approx_db_size_bytes",
            "maintenance"
        ],
        "follow_up_on_success": [
            "refresh agent guidance to confirm storage pressure decreased"
        ],
        "follow_up_on_failure": [
            "rerun the matching dry-run preview and adjust retention policy before retrying"
        ],
        "plan_stage": "maintain",
        "terminality": "non_terminal",
        "can_run_in_parallel": false,
        "requires_human_confirmation": true,
        "evidence_written_to_opendog": false,
        "retry_policy": {
            "allowed": true,
            "max_attempts": 1,
            "strategy": "retry_after_preview_review_or_retention_parameter_change",
            "retry_when": ["the operator explicitly approved a corrected cleanup execution"]
        }
    })
}
