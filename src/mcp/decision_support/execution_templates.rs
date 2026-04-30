mod catalog;
mod enrichment;

use serde_json::{json, Value};

pub(in crate::mcp) fn decision_execution_templates(
    action: &str,
    project_id: Option<&str>,
    verification_status: &str,
    repo_risk_level: &str,
    safe_for_cleanup: Option<bool>,
    safe_for_refactor: Option<bool>,
) -> Value {
    let project_id_value = project_id.unwrap_or("<project>");
    let cleanup_ready = safe_for_cleanup.unwrap_or(false);
    let refactor_ready = safe_for_refactor.unwrap_or(false);
    let project_placeholder_hint = if project_id.is_none() {
        json!([{
            "field": "id",
            "placeholder": "<project>",
            "description": "replace with a registered OPENDOG project id"
        }])
    } else {
        json!([])
    };

    let templates = catalog::base_templates(
        action,
        project_id_value,
        verification_status,
        repo_risk_level,
        cleanup_ready,
        refactor_ready,
        &project_placeholder_hint,
    );

    enrichment::enrich_templates(action, templates, cleanup_ready, refactor_ready)
}
