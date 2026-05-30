use serde_json::Value;

use crate::config::RetentionPolicy;

use super::super::model::StorageMaintenanceTemplateContext;
use super::{execute, preview, vacuum};

pub(in crate::mcp::storage_maintenance) fn storage_maintenance_execution_templates(
    project_id: Option<&str>,
    storage_maintenance: &Value,
) -> Vec<Value> {
    let context = StorageMaintenanceTemplateContext::from_inputs(project_id, storage_maintenance);
    if !context.should_emit_templates() {
        return Vec::new();
    }

    let project_id_value = context.project_id_value();
    let project_placeholder_hint = context.project_placeholder_hint_json();
    let default_policy = RetentionPolicy::default();
    let mut templates = vec![preview::all_scope_preview_template(
        project_id_value,
        &project_placeholder_hint,
    )];
    let mut next_priority = 2;

    preview::append_scope_preview_templates(
        &mut templates,
        &context,
        &project_placeholder_hint,
        &default_policy,
        &mut next_priority,
    );
    execute::append_execution_step_templates(
        &mut templates,
        &context,
        &project_placeholder_hint,
        &default_policy,
        &mut next_priority,
    );
    if let Some(template) = vacuum::vacuum_compaction_template(
        &context,
        project_id_value,
        project_placeholder_hint,
        next_priority,
    ) {
        templates.push(template);
    }

    templates
}
