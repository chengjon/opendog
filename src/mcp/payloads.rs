mod analysis_payloads;
mod config_payloads;
mod orphan_payloads;
mod project_payloads;

pub(crate) use self::analysis_payloads::{
    cleanup_project_data_payload, snapshot_comparison_payload, stats_payload_with_limit,
    time_window_report_payload, unused_files_payload_with_limit, usage_trends_payload,
    DEFAULT_OBSERVATION_PAYLOAD_LIMIT,
};
#[cfg(test)]
pub(crate) use self::analysis_payloads::{stats_payload, unused_files_payload};
pub(crate) use self::config_payloads::{
    build_info_payload, export_project_evidence_payload, global_config_payload,
    project_config_payload, project_config_reload_payload, project_config_update_payload,
    update_global_config_payload,
};
pub(crate) use self::orphan_payloads::{orphan_deletion_plan_payload, orphan_scan_payload};
pub(crate) use self::project_payloads::{
    delete_project_payload, list_projects_payload, register_project_payload, snapshot_payload,
    start_monitor_payload, stop_monitor_payload,
};
