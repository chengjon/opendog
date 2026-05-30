use serde_json::{json, Value};

use super::super::model::StorageMaintenanceTemplateContext;

pub(super) fn vacuum_compaction_template(
    context: &StorageMaintenanceTemplateContext,
    project_id_value: &str,
    project_placeholder_hint: Value,
    priority: usize,
) -> Option<Value> {
    if !context.vacuum_candidate {
        return None;
    }

    Some(json!({
        "template_id": "storage.cleanup.compact",
        "kind": "cli_command",
        "command_template": format!("opendog cleanup-data --id {} --scope all --older-than-days 30 --vacuum --json", project_id_value),
        "preconditions": [
            "run only after preview confirms retained evidence can be pruned safely",
            "reserve vacuum for explicit space-reclaim passes"
        ],
        "blocking_conditions": [
            "requires explicit confirmation because cleanup plus vacuum rewrites the project database"
        ],
        "success_signal": format!(
            "storage_after shows reduced reclaimable space after reclaiming about {} bytes",
            context.approx_reclaimable_bytes
        ),
        "parameter_schema": {
            "id": { "type": "string", "required": true, "source": "project_id" },
            "scope": { "type": "enum", "required": true, "allowed_values": ["all"] },
            "older_than_days": { "type": "integer", "required": false },
            "vacuum": { "type": "boolean", "required": false }
        },
        "default_values": {
            "scope": "all",
            "older_than_days": 30,
            "vacuum": true,
            "json": true
        },
        "placeholder_hints": project_placeholder_hint,
        "priority": priority,
        "should_run_if": [
            "run only when reclaimable bytes remain materially high and the user agrees to maintenance"
        ],
        "skip_if": [
            "skip if preview shows little reclaimable space or the user does not want database compaction"
        ],
        "expected_output_fields": [
            "storage_after.approx_db_size_bytes",
            "storage_after.approx_reclaimable_bytes",
            "maintenance.vacuumed"
        ],
        "follow_up_on_success": [
            "refresh agent guidance if storage pressure was the main maintenance concern"
        ],
        "follow_up_on_failure": [
            "rerun the preview without vacuum to inspect counts and maintenance notes again"
        ],
        "plan_stage": "maintain",
        "terminality": "non_terminal",
        "can_run_in_parallel": false,
        "requires_human_confirmation": true,
        "evidence_written_to_opendog": false,
        "retry_policy": {
            "allowed": true,
            "max_attempts": 1,
            "strategy": "retry_after_confirmation_or_parameter_change",
            "retry_when": ["the user explicitly approved the vacuum pass"]
        }
    }))
}
