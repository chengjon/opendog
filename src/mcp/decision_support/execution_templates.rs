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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- decision_execution_templates ---

    #[test]
    fn decision_execution_templates_review_failing_verification() {
        let result = decision_execution_templates(
            "review_failing_verification",
            Some("proj-1"),
            "not_recorded",
            "low",
            None,
            None,
        );
        assert!(result.is_array());
        let templates = result.as_array().unwrap();
        assert_eq!(templates.len(), 2);
        let ids: Vec<&str> = templates
            .iter()
            .map(|t| t["template_id"].as_str().unwrap())
            .collect();
        assert!(ids.contains(&"verification.review_status"));
        assert!(ids.contains(&"verification.rerun"));
        // With project_id, placeholder hints should be empty
        assert_eq!(templates[0]["placeholder_hints"], json!([]));
    }

    #[test]
    fn decision_execution_templates_stabilize_repository() {
        let result = decision_execution_templates(
            "stabilize_repository_state",
            Some("proj-1"),
            "available",
            "high",
            None,
            None,
        );
        let templates = result.as_array().unwrap();
        assert_eq!(templates.len(), 2);
        assert_eq!(templates[0]["template_id"], "repo.status");
        assert_eq!(templates[1]["template_id"], "repo.diff");
    }

    #[test]
    fn decision_execution_templates_start_monitor() {
        let result = decision_execution_templates(
            "start_monitor",
            Some("proj-2"),
            "not_recorded",
            "low",
            None,
            None,
        );
        let templates = result.as_array().unwrap();
        assert_eq!(templates.len(), 2);
        let ids: Vec<&str> = templates
            .iter()
            .map(|t| t["template_id"].as_str().unwrap())
            .collect();
        assert!(ids.contains(&"monitor.start"));
        assert!(ids.contains(&"snapshot.baseline"));
    }

    #[test]
    fn decision_execution_templates_take_snapshot() {
        let result = decision_execution_templates(
            "take_snapshot",
            Some("proj-3"),
            "available",
            "low",
            None,
            None,
        );
        let templates = result.as_array().unwrap();
        assert_eq!(templates.len(), 2);
        assert_eq!(templates[0]["template_id"], "snapshot.take");
        assert_eq!(templates[1]["template_id"], "stats.inspect");
    }

    #[test]
    fn decision_execution_templates_unknown_action_falls_back() {
        let result = decision_execution_templates(
            "some_unknown_action",
            Some("proj-x"),
            "available",
            "low",
            None,
            None,
        );
        let templates = result.as_array().unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0]["template_id"], "guidance.refresh");
    }

    #[test]
    fn decision_execution_templates_no_project_id_adds_placeholder_hint() {
        let result =
            decision_execution_templates("start_monitor", None, "available", "low", None, None);
        let templates = result.as_array().unwrap();
        // First template should have placeholder hints because project_id is None
        let hints = templates[0]["placeholder_hints"].as_array().unwrap();
        assert!(!hints.is_empty());
        assert_eq!(hints[0]["field"], "id");
        assert_eq!(hints[0]["placeholder"], "<project>");
    }

    #[test]
    fn decision_execution_templates_with_project_id_no_placeholder_hint() {
        let result = decision_execution_templates(
            "take_snapshot",
            Some("my-project"),
            "available",
            "low",
            None,
            None,
        );
        let templates = result.as_array().unwrap();
        let hints = templates[0]["placeholder_hints"].as_array().unwrap();
        assert!(hints.is_empty());
    }

    #[test]
    fn decision_execution_templates_enrichment_adds_priority() {
        let result = decision_execution_templates(
            "review_failing_verification",
            Some("p"),
            "available",
            "low",
            None,
            None,
        );
        let templates = result.as_array().unwrap();
        assert_eq!(templates[0]["priority"], 1);
        assert_eq!(templates[1]["priority"], 2);
    }

    #[test]
    fn decision_execution_templates_enrichment_adds_plan_stage() {
        let result = decision_execution_templates(
            "review_failing_verification",
            Some("p"),
            "available",
            "low",
            None,
            None,
        );
        let templates = result.as_array().unwrap();
        assert_eq!(templates[0]["plan_stage"], "verify");
    }

    #[test]
    fn decision_execution_templates_includes_success_signal() {
        let result = decision_execution_templates(
            "take_snapshot",
            Some("p"),
            "available",
            "low",
            None,
            None,
        );
        let templates = result.as_array().unwrap();
        for t in templates {
            assert!(t["success_signal"].is_string());
            assert!(!t["success_signal"].as_str().unwrap().is_empty());
        }
    }

    #[test]
    fn decision_execution_templates_cleanup_ready_affects_blocking() {
        // cleanup_ready = false
        let result_blocked = decision_execution_templates(
            "review_unused_files",
            Some("p"),
            "available",
            "low",
            Some(false),
            None,
        );
        let templates_blocked = result_blocked.as_array().unwrap();
        let bc = templates_blocked[0]["blocking_conditions"]
            .as_array()
            .unwrap();
        assert!(!bc.is_empty());

        // cleanup_ready = true
        let result_clear = decision_execution_templates(
            "review_unused_files",
            Some("p"),
            "available",
            "low",
            Some(true),
            None,
        );
        let templates_clear = result_clear.as_array().unwrap();
        let bc2 = templates_clear[0]["blocking_conditions"]
            .as_array()
            .unwrap();
        assert!(bc2.is_empty());
    }

    #[test]
    fn decision_execution_templates_none_safe_flags_default_to_false() {
        let result = decision_execution_templates(
            "inspect_hot_files",
            Some("p"),
            "available",
            "low",
            None,
            None,
        );
        // None defaults to false for both cleanup and refactor readiness
        let templates = result.as_array().unwrap();
        let stats_t = &templates[0];
        let bc = stats_t["blocking_conditions"].as_array().unwrap();
        assert!(!bc.is_empty()); // refactor not ready => blocked
    }

    #[test]
    fn decision_execution_templates_refactor_ready_clears_blocking() {
        let result = decision_execution_templates(
            "inspect_hot_files",
            Some("p"),
            "available",
            "low",
            None,
            Some(true),
        );
        let templates = result.as_array().unwrap();
        let stats_t = &templates[0];
        let bc = stats_t["blocking_conditions"].as_array().unwrap();
        assert!(bc.is_empty()); // refactor ready => unblocked
    }

    #[test]
    fn decision_execution_templates_includes_retry_policy() {
        let result = decision_execution_templates(
            "review_failing_verification",
            Some("p"),
            "available",
            "low",
            None,
            None,
        );
        let templates = result.as_array().unwrap();
        for t in templates {
            assert!(t["retry_policy"].is_object());
            assert!(t["retry_policy"]["allowed"].is_boolean());
            assert!(t["retry_policy"]["max_attempts"].is_number());
        }
    }

    #[test]
    fn decision_execution_templates_includes_follow_ups() {
        let result = decision_execution_templates(
            "review_failing_verification",
            Some("p"),
            "available",
            "low",
            None,
            None,
        );
        let templates = result.as_array().unwrap();
        for t in templates {
            assert!(t["follow_up_on_success"].is_array());
            assert!(t["follow_up_on_failure"].is_array());
        }
    }
}
