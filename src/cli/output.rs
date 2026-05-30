mod config_output;
mod governance_output;
mod guidance_output;
mod project_output;
mod report_output;
mod verification_output;

pub use self::config_output::{
    print_global_config, print_global_config_update, print_project_config,
    print_project_config_reload, print_project_config_update,
};
pub use self::governance_output::{
    print_governance_state, print_lane_closed, print_lane_created, print_node_upserted,
};
pub use self::guidance_output::{
    print_agent_guidance, print_data_risk, print_decision_brief, print_workspace_data_risk,
};
pub use self::project_output::{
    print_cleanup_data_result, print_project_list, print_registered, print_snapshot_result,
    print_stats, print_unused,
};
pub use self::report_output::{
    print_activity_rollups, print_snapshot_comparison, print_time_window_report, print_usage_trends,
};
pub use self::verification_output::{
    print_verification_executed, print_verification_recorded, print_verification_status,
};

pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("...{}", &s[s.len() - max + 3..])
    }
}

#[cfg(test)]
mod tests;
