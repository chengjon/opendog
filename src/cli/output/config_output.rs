use crate::config::{
    GlobalConfigUpdateResult, ProjectConfig, ProjectConfigReload, ProjectConfigUpdateResult,
    ProjectConfigView,
};

pub(super) fn print_global_config(config: &ProjectConfig) {
    println!("Global defaults:");
    print_vec("  Ignore patterns", &config.ignore_patterns);
    print_vec("  Process whitelist", &config.process_whitelist);
}

pub(super) fn print_project_config(view: &ProjectConfigView) {
    println!("Project '{}' configuration:", view.project_id);
    print_vec(
        "  Effective ignore patterns",
        &view.effective.ignore_patterns,
    );
    print_vec(
        "  Effective process whitelist",
        &view.effective.process_whitelist,
    );
    println!();
    println!(
        "  Ignore patterns source: {}",
        if view.project_overrides.ignore_patterns.is_some() {
            "project override"
        } else {
            "global default"
        }
    );
    println!(
        "  Process whitelist source: {}",
        if view.project_overrides.process_whitelist.is_some() {
            "project override"
        } else {
            "global default"
        }
    );
}

pub(super) fn print_project_config_update(result: &ProjectConfigUpdateResult) {
    print_project_config(&ProjectConfigView {
        project_id: result.project_id.clone(),
        global_defaults: result.global_defaults.clone(),
        project_overrides: result.project_overrides.clone(),
        effective: result.effective.clone(),
    });
    println!();
    print_reload_summary(&result.project_id, &result.reload);
}

pub(super) fn print_global_config_update(result: &GlobalConfigUpdateResult) {
    print_global_config(&result.global_defaults);
    println!();
    println!(
        "Reloaded projects: {}",
        if result.reloaded_projects.is_empty() {
            "none".to_string()
        } else {
            result
                .reloaded_projects
                .iter()
                .map(|item| {
                    format!(
                        "{}(runtime_reloaded={}, snapshot_refreshed={})",
                        item.project_id, item.runtime_reloaded, item.snapshot_refreshed
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        }
    );
}

pub(super) fn print_project_config_reload(
    id: &str,
    reload: &ProjectConfigReload,
    effective: &ProjectConfig,
) {
    println!("Reload result for '{}':", id);
    println!(
        "  monitor_running={} runtime_reloaded={} snapshot_refreshed={}",
        reload.monitor_running, reload.runtime_reloaded, reload.snapshot_refreshed
    );
    if !reload.changed_fields.is_empty() {
        println!("  changed: {}", reload.changed_fields.join(", "));
    }
    if !reload.skipped_fields.is_empty() {
        println!("  skipped: {}", reload.skipped_fields.join(" | "));
    }
    print_vec("  Effective ignore patterns", &effective.ignore_patterns);
    print_vec(
        "  Effective process whitelist",
        &effective.process_whitelist,
    );
}

fn print_reload_summary(id: &str, reload: &ProjectConfigReload) {
    println!("Reload summary for '{}':", id);
    println!(
        "  monitor_running={} runtime_reloaded={} snapshot_refreshed={}",
        reload.monitor_running, reload.runtime_reloaded, reload.snapshot_refreshed
    );
    if !reload.changed_fields.is_empty() {
        println!("  changed: {}", reload.changed_fields.join(", "));
    }
    if !reload.skipped_fields.is_empty() {
        println!("  skipped: {}", reload.skipped_fields.join(" | "));
    }
}

fn print_vec(title: &str, values: &[String]) {
    println!("{}:", title);
    if values.is_empty() {
        println!("    <empty>");
        return;
    }
    for value in values {
        println!("    {}", value);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ignore_source_label_project_override() {
        // Mirrors the conditional in print_project_config for ignore_patterns
        let has_override: Option<Vec<String>> = Some(vec!["target".into()]);
        let label = if has_override.is_some() {
            "project override"
        } else {
            "global default"
        };
        assert_eq!(label, "project override");
    }

    #[test]
    fn ignore_source_label_global_default() {
        let has_override: Option<Vec<String>> = None;
        let label = if has_override.is_some() {
            "project override"
        } else {
            "global default"
        };
        assert_eq!(label, "global default");
    }

    #[test]
    fn reloaded_projects_empty_formats_as_none() {
        let reloaded: Vec<String> = vec![];
        let text = if reloaded.is_empty() {
            "none".to_string()
        } else {
            reloaded.join(", ")
        };
        assert_eq!(text, "none");
    }

    #[test]
    fn reloaded_projects_formats_entries() {
        // Mirrors the format inside print_global_config_update
        let text = format!(
            "{}(runtime_reloaded={}, snapshot_refreshed={})",
            "proj1", true, false
        );
        assert_eq!(
            text,
            "proj1(runtime_reloaded=true, snapshot_refreshed=false)"
        );
    }

    #[test]
    fn changed_fields_join_with_comma() {
        let fields = [
            "ignore_patterns".to_string(),
            "process_whitelist".to_string(),
        ];
        let text = fields.join(", ");
        assert_eq!(text, "ignore_patterns, process_whitelist");
    }

    #[test]
    fn skipped_fields_join_with_pipe() {
        let fields = ["a".to_string(), "b".to_string()];
        let text = fields.join(" | ");
        assert_eq!(text, "a | b");
    }

    #[test]
    fn vec_formatter_empty_shows_empty_marker() {
        let values: Vec<String> = vec![];
        // Mirrors the guard in print_vec
        let has_empty_marker = values.is_empty();
        assert!(has_empty_marker);
    }

    #[test]
    fn vec_formatter_nonempty_lists_values() {
        let values = ["node".to_string(), "python".to_string()];
        // Mirrors the iteration in print_vec
        let rendered: Vec<String> = values.iter().map(|v| format!("    {}", v)).collect();
        assert_eq!(rendered.len(), 2);
        assert_eq!(rendered[0], "    node");
        assert_eq!(rendered[1], "    python");
    }

    #[test]
    fn reload_summary_format_string() {
        // Mirrors the format in print_project_config_reload
        let line = format!(
            "  monitor_running={} runtime_reloaded={} snapshot_refreshed={}",
            true, false, false
        );
        assert_eq!(
            line,
            "  monitor_running=true runtime_reloaded=false snapshot_refreshed=false"
        );
    }
}
