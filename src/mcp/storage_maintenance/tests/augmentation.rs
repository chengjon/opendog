use super::*;

#[test]
fn augment_no_op_for_non_maintenance() {
    let mut entrypoints = json!({
        "next_cli_commands": ["original_cmd"],
        "selection_reasons": [{"kind": "other", "why": "original"}],
        "execution_templates": [{"template_id": "existing", "priority": 1}],
    });
    let sm = json!({"maintenance_candidate": false});
    augment_entrypoints_for_storage_maintenance(&mut entrypoints, Some("proj"), &sm);
    assert_eq!(entrypoints["next_cli_commands"][0], "original_cmd");
    assert_eq!(entrypoints["selection_reasons"][0]["why"], "original");
}

#[test]
fn augment_prepends_cleanup_command() {
    let mut entrypoints = json!({
        "next_cli_commands": ["existing_cmd"],
        "selection_reasons": [],
        "execution_templates": [],
    });
    let sm = json!({
        "maintenance_candidate": true,
        "vacuum_candidate": false,
        "approx_reclaimable_bytes": 0,
        "summary": "needs cleanup",
    });
    augment_entrypoints_for_storage_maintenance(&mut entrypoints, Some("myproj"), &sm);
    let cmds = entrypoints["next_cli_commands"].as_array().unwrap();
    assert!(cmds[0].as_str().unwrap().contains("myproj"));
    assert!(cmds[0].as_str().unwrap().contains("cleanup-data"));
}

#[test]
fn augment_prepends_selection_reason() {
    let mut entrypoints = json!({
        "next_cli_commands": [],
        "selection_reasons": [],
        "execution_templates": [],
    });
    let sm = json!({
        "maintenance_candidate": true,
        "vacuum_candidate": false,
        "approx_reclaimable_bytes": 0,
        "summary": "database is large",
    });
    augment_entrypoints_for_storage_maintenance(&mut entrypoints, Some("proj"), &sm);
    let reasons = entrypoints["selection_reasons"].as_array().unwrap();
    assert_eq!(reasons[0]["why"], "database is large");
}

#[test]
fn augment_prepends_execution_templates_and_renumbers() {
    let mut entrypoints = json!({
        "next_cli_commands": [],
        "selection_reasons": [],
        "execution_templates": [{"template_id": "existing", "priority": 99}],
    });
    let sm = json!({
        "maintenance_candidate": true,
        "vacuum_candidate": true,
        "approx_reclaimable_bytes": 10_000_000,
        "summary": "needs vacuum",
    });
    augment_entrypoints_for_storage_maintenance(&mut entrypoints, Some("proj"), &sm);
    let templates = entrypoints["execution_templates"].as_array().unwrap();
    assert_eq!(templates.len(), 3);
    // Reverse insertion: compact inserted at 0, then preview inserted at 0, pushing compact to 1
    assert_eq!(templates[0]["template_id"], "storage.cleanup.preview");
    assert_eq!(templates[1]["template_id"], "storage.cleanup.compact");
    assert_eq!(templates[2]["template_id"], "existing");
    // All priorities renumbered sequentially
    assert_eq!(templates[0]["priority"], 1);
    assert_eq!(templates[1]["priority"], 2);
    assert_eq!(templates[2]["priority"], 3);
}

#[test]
fn augment_uses_project_placeholder_when_none() {
    let mut entrypoints = json!({
        "next_cli_commands": [],
        "selection_reasons": [],
        "execution_templates": [],
    });
    let sm = json!({
        "maintenance_candidate": true,
        "vacuum_candidate": false,
        "approx_reclaimable_bytes": 0,
        "summary": "needs cleanup",
    });
    augment_entrypoints_for_storage_maintenance(&mut entrypoints, None, &sm);
    let cmds = entrypoints["next_cli_commands"].as_array().unwrap();
    assert!(cmds[0].as_str().unwrap().contains("<project>"));
}

#[test]
fn augment_handles_missing_arrays_gracefully() {
    let mut entrypoints = json!({
        "unrelated_key": "preserved",
    });
    let sm = json!({
        "maintenance_candidate": true,
        "vacuum_candidate": true,
        "approx_reclaimable_bytes": 10_000_000,
        "summary": "needs work",
    });
    // Should not panic and should not remove existing keys
    augment_entrypoints_for_storage_maintenance(&mut entrypoints, Some("proj"), &sm);
    assert_eq!(entrypoints["unrelated_key"], "preserved");
    // No next_cli_commands, selection_reasons, or execution_templates arrays created
    assert!(entrypoints
        .get("next_cli_commands")
        .is_none_or(|v| v.is_null()));
}
