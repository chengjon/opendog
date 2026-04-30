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
