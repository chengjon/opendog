use crate::contracts::{
    versioned_error_payload, versioned_project_error_payload, versioned_project_payload,
    MCP_ACTIVITY_ROLLUPS_V1, MCP_BUILD_INFO_V1, MCP_CLOSE_GOVERNANCE_LANE_V1,
    MCP_CREATE_GOVERNANCE_LANE_V1, MCP_DATA_RISK_V1, MCP_DECISION_BRIEF_V1, MCP_DELETE_PROJECT_V1,
    MCP_GET_GOVERNANCE_STATE_V1, MCP_GLOBAL_CONFIG_V1, MCP_GUIDANCE_V1, MCP_LIST_PROJECTS_V1,
    MCP_ORPHAN_DELETION_PLAN_V1, MCP_ORPHAN_SCAN_V1, MCP_PROJECT_CONFIG_V1,
    MCP_RECORD_VERIFICATION_V1, MCP_REGISTER_PROJECT_V1, MCP_RUN_VERIFICATION_V1,
    MCP_SNAPSHOT_COMPARE_V1, MCP_SNAPSHOT_V1, MCP_START_MONITOR_V1, MCP_STATS_V1,
    MCP_STOP_MONITOR_V1, MCP_TIME_WINDOW_REPORT_V1, MCP_UNUSED_FILES_V1,
    MCP_UPSERT_GOVERNANCE_NODE_V1, MCP_USAGE_TRENDS_V1, MCP_VERIFICATION_STATUS_V1,
    MCP_WORKSPACE_DATA_RISK_V1, OPENDOG_BUILD_TIME, OPENDOG_GIT_HASH, OPENDOG_VERSION,
};

mod analysis_handlers;
mod attention;
mod config_handlers;
mod constraints;
mod data_risk;
mod decision_support;
mod governance_handlers;
mod governance_layer;
mod guidance_handlers;
mod guidance_payload;
mod guidance_scaffold;
mod guidance_types;
mod mock_detection;
mod observation;
mod orphan_handlers;
mod params;
mod payloads;
mod project_guidance;
mod project_handlers;
mod project_recommendation;
mod repo_risk;
mod resource_handlers;
pub(crate) mod review_candidates;
mod risk_handlers;
mod serialization;
mod server_core;
mod server_handler;
mod server_tools;
mod storage_maintenance;
mod strategy;
mod tool_helpers;
#[cfg(test)]
mod tool_inventory;
mod toolchain;
mod verification_evidence;
mod verification_handlers;
mod workspace_decision;

#[cfg(test)]
pub(crate) use self::tool_inventory::mcp_tool_inventory;

use self::analysis_handlers::{
    handle_compare_snapshots, handle_get_activity_rollups, handle_get_stats,
    handle_get_time_window_report, handle_get_unused_files, handle_get_usage_trends,
};
use self::attention::{
    enrich_project_overview_with_attention, sort_project_recommendations, workspace_portfolio_layer,
};
use self::config_handlers::{
    handle_get_build_info, handle_get_global_config, handle_get_project_config,
};
use self::constraints::{
    build_constraints_boundaries_layer, common_boundary_hints,
    external_truth_boundary_for_top_project, project_readiness_snapshot,
    review_focus_projection_for_top_project, WorkspaceCounts,
};
#[cfg(test)]
use self::data_risk::data_risk_guidance;
use self::data_risk::path_kind_score;
pub(crate) use self::data_risk::{
    normalize_candidate_type, normalize_min_review_priority, project_data_risk_payload,
    review_priority_score, workspace_data_risk_overview_payload, DataCandidate, MockDataReport,
};
use self::decision_support::{
    decision_action_profile, decision_entrypoints_payload, decision_execution_templates,
    decision_risk_profile,
};
use self::governance_handlers::{
    handle_close_governance_lane, handle_create_governance_lane, handle_get_governance_state,
    handle_upsert_governance_node,
};
pub(crate) use self::governance_layer::build_governance_layer;
use self::guidance_handlers::handle_get_guidance;
#[cfg(test)]
pub(crate) use self::guidance_payload::default_governance_layer;
pub(crate) use self::guidance_payload::{
    agent_guidance_payload, latest_verification_runs_for_project, now_unix_secs,
    ProjectGuidanceData, ProjectGuidanceState,
};
use self::guidance_scaffold::{
    base_guidance_layers, default_shell_verification_commands, set_recommended_flow, tool_guidance,
};
pub(crate) use self::mock_detection::detect_mock_data_report;
use self::observation::{
    activity_is_stale, latest_activity_timestamp, latest_verification_timestamp,
    project_observation_layer, snapshot_is_stale, verification_is_stale,
};
use self::orphan_handlers::{handle_scan_orphans, handle_verify_deletion_plan};
pub use self::params::*;
pub(crate) use self::payloads::{
    activity_rollups_payload, build_info_payload, cleanup_project_data_payload,
    close_governance_lane_payload, create_governance_lane_payload, delete_project_payload,
    export_project_evidence_payload, get_governance_state_payload, global_config_payload,
    list_projects_payload, orphan_deletion_plan_payload, orphan_scan_payload,
    project_config_payload, project_config_reload_payload, project_config_update_payload,
    register_project_payload, snapshot_comparison_payload, snapshot_payload, start_monitor_payload,
    stats_payload_with_limit, stop_monitor_payload, time_window_report_payload,
    unused_files_payload_with_limit, update_global_config_payload, upsert_governance_node_payload,
    usage_trends_payload, BuildInfoPayloadInput, DEFAULT_OBSERVATION_PAYLOAD_LIMIT,
};
#[cfg(test)]
pub(crate) use self::payloads::{stats_payload, unused_files_payload};
use self::project_guidance::{
    register_project_guidance, snapshot_guidance, start_monitor_guidance, stats_guidance,
    unused_guidance,
};
use self::project_handlers::{
    handle_delete_project, handle_list_projects, handle_register_project, handle_start_monitor,
    handle_stop_monitor, handle_take_snapshot,
};
pub(crate) use self::project_recommendation::collect_project_guidance_context;
#[cfg(test)]
use self::project_recommendation::{project_overview, recommend_project_action};
use self::repo_risk::repo_status_risk_layer;
use self::risk_handlers::{
    handle_get_data_risk_candidates, handle_get_workspace_data_risk_overview,
};
pub use self::server_core::{run_stdio, OpenDogServer};
use self::storage_maintenance::{
    augment_entrypoints_for_storage_maintenance, project_storage_maintenance_with_policy,
    storage_maintenance_layer,
};
use self::strategy::{
    agent_guidance_recommended_flow, strategy_profile, workspace_strategy_profile,
};
use self::tool_helpers::{error_json_for, open_dog_error_code, validation_error_json};
use self::toolchain::{
    detect_project_commands, project_toolchain_layer, workspace_toolchain_layer,
};
pub(crate) use self::verification_evidence::verification_status_layer;
use self::verification_evidence::{
    record_verification_payload, run_verification_payload, verification_has_failures,
    verification_is_missing, verification_status_payload, workspace_verification_evidence_layer,
};
use self::verification_handlers::{
    handle_get_verification_status, handle_record_verification_result,
    handle_run_verification_command,
};
pub(crate) use self::workspace_decision::{
    collect_workspace_data_risk_summaries, decision_brief_payload, workspace_data_risk_payload,
};

#[cfg(test)]
mod tests;
