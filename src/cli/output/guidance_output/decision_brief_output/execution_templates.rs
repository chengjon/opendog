use serde_json::Value;

pub(super) fn print_execution_templates(execution_templates: &[Value]) {
    if execution_templates.is_empty() {
        return;
    }

    println!();
    println!("Execution templates:");
    for item in execution_templates.iter().take(2) {
        println!(
            "  {} [{}] priority={} stage={}",
            item["template_id"].as_str().unwrap_or("-"),
            item["kind"].as_str().unwrap_or("-"),
            item["priority"].as_u64().unwrap_or(0),
            item["plan_stage"].as_str().unwrap_or("-"),
        );
        if let Some(tool) = item["tool"].as_str() {
            println!("    tool: {}", tool);
        }
        if let Some(command) = item["command_template"].as_str() {
            println!("    command: {}", command);
        }
        if let Some(signal) = item["success_signal"].as_str() {
            println!("    success: {}", signal);
        }
        println!(
            "    parallel={} human_confirmation={} evidence_to_opendog={}",
            item["can_run_in_parallel"].as_bool().unwrap_or(false),
            item["requires_human_confirmation"]
                .as_bool()
                .unwrap_or(false),
            item["evidence_written_to_opendog"]
                .as_bool()
                .unwrap_or(false),
        );
        if let Some(terminality) = item["terminality"].as_str() {
            println!("    terminality: {}", terminality);
        }
        if let Some(defaults) = item["default_values"].as_object() {
            if !defaults.is_empty() {
                println!(
                    "    defaults: {}",
                    serde_json::Value::Object(defaults.clone())
                );
            }
        }
        if let Some(hints) = item["placeholder_hints"].as_array() {
            if !hints.is_empty() {
                println!(
                    "    placeholders: {}",
                    hints[0]["placeholder"].as_str().unwrap_or("-")
                );
            }
        }
        if let Some(conditions) = item["should_run_if"].as_array() {
            if !conditions.is_empty() {
                println!(
                    "    should_run_if: {}",
                    conditions[0].as_str().unwrap_or("-")
                );
            }
        }
        if let Some(conditions) = item["skip_if"].as_array() {
            if !conditions.is_empty() {
                println!("    skip_if: {}", conditions[0].as_str().unwrap_or("-"));
            }
        }
        if let Some(fields) = item["expected_output_fields"].as_array() {
            if !fields.is_empty() {
                println!("    expected_output: {}", fields[0].as_str().unwrap_or("-"));
            }
        }
        if let Some(followups) = item["follow_up_on_success"].as_array() {
            if !followups.is_empty() {
                println!("    on_success: {}", followups[0].as_str().unwrap_or("-"));
            }
        }
        if let Some(followups) = item["follow_up_on_failure"].as_array() {
            if !followups.is_empty() {
                println!("    on_failure: {}", followups[0].as_str().unwrap_or("-"));
            }
        }
        if item["retry_policy"].is_object() {
            println!("    retry_policy: {}", item["retry_policy"]);
        }
    }
}
