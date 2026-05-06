use super::repo_risk::{
    detect_lockfile_anomalies, parse_status_porcelain, repo_risk_findings, GitStatusEntry,
    RepoRiskSnapshot,
};
use super::{
    agent_guidance_payload, build_constraints_boundaries_layer, cleanup_project_data_payload,
    collect_workspace_data_risk_summaries, create_project_guidance, create_project_payload,
    data_risk_guidance, decision_brief_payload, decision_entrypoints_payload,
    delete_project_payload, detect_mock_data_report, detect_project_commands, error_json_for,
    export_project_evidence_payload, global_config_payload, list_projects_payload,
    normalize_candidate_type, normalize_min_review_priority, now_unix_secs, project_config_payload,
    project_config_reload_payload, project_config_update_payload, project_overview,
    project_toolchain_layer, recommend_project_action, record_verification_payload,
    repo_status_risk_layer, review_priority_score, run_verification_payload,
    scoped_projects_or_error, snapshot_payload, start_monitor_guidance, start_monitor_payload,
    stats_guidance, stats_payload, stop_monitor_payload, tool_guidance, unused_files_payload,
    unused_guidance, update_global_config_payload, validation_error_json,
    verification_status_layer, verification_status_payload, workspace_data_risk_overview_payload,
    workspace_portfolio_layer, workspace_strategy_profile, AgentGuidanceParams, DataCandidate,
    GuidanceParams, MockDataReport, ProjectGuidanceState,
};
use crate::config::{
    GlobalConfigUpdateResult, ProjectConfig, ProjectConfigOverrides, ProjectConfigReload,
    ProjectConfigUpdateResult, ProjectConfigView, ProjectInfo, ProjectReloadStatus,
};
use crate::contracts::{
    MCP_CLEANUP_PROJECT_DATA_V1, MCP_CREATE_PROJECT_V1, MCP_DATA_RISK_V1, MCP_DECISION_BRIEF_V1,
    MCP_DELETE_PROJECT_V1, MCP_EXPORT_PROJECT_EVIDENCE_V1, MCP_GLOBAL_CONFIG_V1, MCP_GUIDANCE_V1,
    MCP_LIST_PROJECTS_V1, MCP_PROJECT_CONFIG_V1, MCP_RECORD_VERIFICATION_V1,
    MCP_RELOAD_PROJECT_CONFIG_V1, MCP_RUN_VERIFICATION_V1, MCP_SNAPSHOT_V1, MCP_START_MONITOR_V1,
    MCP_STATS_V1, MCP_STOP_MONITOR_V1, MCP_UNUSED_FILES_V1, MCP_UPDATE_GLOBAL_CONFIG_V1,
    MCP_UPDATE_PROJECT_CONFIG_V1, MCP_VERIFICATION_STATUS_V1,
};
use crate::core::retention::{
    CleanupCountBreakdown, CleanupMaintenanceStatus, ProjectDataCleanupResult, StorageMetrics,
};
use crate::core::stats::ProjectSummary;
use crate::error::OpenDogError;
use crate::storage::queries::{StatsEntry, VerificationRun};
use rmcp::handler::server::wrapper::Json;
use serde_json::json;
use tempfile::TempDir;

#[path = "tests/data_risk_cases.rs"]
mod data_risk_cases;
#[path = "tests/guidance_basics.rs"]
mod guidance_basics;
#[path = "tests/overview_constraints.rs"]
mod overview_constraints;
#[path = "tests/payload_contracts.rs"]
mod payload_contracts;
#[path = "tests/portfolio_commands.rs"]
mod portfolio_commands;
#[path = "tests/repo_and_readiness.rs"]
mod repo_and_readiness;
#[path = "tests/tool_surface.rs"]
mod tool_surface;

fn fresh_ts() -> String {
    now_unix_secs().to_string()
}

fn stale_ts() -> String {
    (now_unix_secs() - 8 * 24 * 60 * 60).to_string()
}

fn workspace_toolchain_overview(
    project_id: &str,
    project_type: &str,
    confidence: &str,
    test_commands: &[&str],
    lint_commands: &[&str],
    build_commands: &[&str],
) -> serde_json::Value {
    json!({
        "project_id": project_id,
        "status": "monitoring",
        "unused_files": 0,
        "recommended_next_action": "inspect_hot_files",
        "recommended_flow": ["Inspect the hottest files first."],
        "recommended_reason": "Activity exists.",
        "strategy_confidence": "medium",
        "safe_for_cleanup": true,
        "safe_for_refactor": true,
        "observation": {
            "coverage_state": "ready",
            "freshness": {
                "snapshot": { "status": "fresh" },
                "activity": { "status": "fresh" },
                "verification": { "status": "fresh" }
            }
        },
        "repo_status_risk": {
            "status": "available",
            "risk_level": "low",
            "is_dirty": false,
            "operation_states": []
        },
        "verification_evidence": {
            "status": "available",
            "failing_runs": []
        },
        "mock_data_summary": {
            "hardcoded_candidate_count": 0,
            "mock_candidate_count": 0
        },
        "storage_maintenance": {
            "status": "available",
            "maintenance_candidate": false,
            "vacuum_candidate": false,
            "approx_db_size_bytes": 0,
            "approx_reclaimable_bytes": 0,
            "reclaim_ratio": 0.0,
            "suggested_mode": "none"
        },
        "project_toolchain": {
            "status": "available",
            "project_type": project_type,
            "confidence": confidence,
            "recommended_test_commands": test_commands,
            "recommended_lint_commands": lint_commands,
            "recommended_build_commands": build_commands,
            "recommended_search_commands": ["rg \"<pattern>\" ."]
        }
    })
}

fn workspace_verification_overview(
    project_id: &str,
    verification_status: &str,
    verification_freshness: &str,
    failing_runs: &[serde_json::Value],
    safe_for_cleanup: bool,
    safe_for_refactor: bool,
) -> serde_json::Value {
    let cleanup_level = if safe_for_cleanup { "allow" } else { "blocked" };
    let refactor_level = if safe_for_refactor {
        "allow"
    } else {
        "blocked"
    };
    json!({
        "project_id": project_id,
        "status": "monitoring",
        "unused_files": 0,
        "recommended_next_action": "inspect_hot_files",
        "recommended_flow": ["Inspect the hottest files first."],
        "recommended_reason": "Activity exists.",
        "strategy_confidence": "medium",
        "safe_for_cleanup": safe_for_cleanup,
        "safe_for_cleanup_reason": if safe_for_cleanup {
            "Current evidence supports cleanup review."
        } else {
            "Cleanup readiness is blocked by verification or repository risk."
        },
        "safe_for_refactor": safe_for_refactor,
        "safe_for_refactor_reason": if safe_for_refactor {
            "Current evidence supports scoped refactor work."
        } else {
            "Refactor readiness is blocked by verification or repository risk."
        },
        "observation": {
            "coverage_state": if verification_freshness == "missing" {
                "missing_verification"
            } else if verification_freshness == "stale" {
                "stale_evidence"
            } else {
                "ready"
            },
            "freshness": {
                "snapshot": { "status": "fresh" },
                "activity": { "status": "fresh" },
                "verification": { "status": verification_freshness }
            }
        },
        "repo_status_risk": {
            "status": "available",
            "risk_level": "low",
            "is_dirty": false,
            "operation_states": []
        },
        "verification_evidence": {
            "status": verification_status,
            "failing_runs": failing_runs,
            "safe_for_cleanup": safe_for_cleanup,
            "safe_for_refactor": safe_for_refactor,
            "gate_assessment": {
                "cleanup": {
                    "level": cleanup_level
                },
                "refactor": {
                    "level": refactor_level
                }
            }
        },
        "mock_data_summary": {
            "hardcoded_candidate_count": 0,
            "mock_candidate_count": 0
        },
        "storage_maintenance": {
            "status": "available",
            "maintenance_candidate": false,
            "vacuum_candidate": false,
            "approx_db_size_bytes": 0,
            "approx_reclaimable_bytes": 0,
            "reclaim_ratio": 0.0,
            "suggested_mode": "none"
        },
        "project_toolchain": {
            "status": "available",
            "project_type": "rust",
            "confidence": "high",
            "recommended_test_commands": ["cargo test"],
            "recommended_lint_commands": ["cargo clippy --all-targets --all-features -- -D warnings"],
            "recommended_build_commands": ["cargo check"],
            "recommended_search_commands": ["rg \"<pattern>\" ."]
        }
    })
}
