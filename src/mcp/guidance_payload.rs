use serde_json::{json, Map, Value};

use crate::contracts::MCP_GUIDANCE_V1;
use crate::core::verification;
use crate::storage::database::Database;
use crate::storage::queries::{StatsEntry, VerificationRun};

use super::{
    agent_guidance_recommended_flow, base_guidance_layers, build_constraints_boundaries_layer,
    default_shell_verification_commands, sort_project_recommendations, storage_maintenance_layer,
    workspace_portfolio_layer, workspace_strategy_profile, workspace_toolchain_layer,
    workspace_verification_evidence_layer,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct ProjectGuidanceState {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) root_path: std::path::PathBuf,
    pub(crate) total_files: i64,
    pub(crate) accessed_files: i64,
    pub(crate) unused_files: i64,
    pub(crate) latest_snapshot_captured_at: Option<String>,
    pub(crate) latest_activity_at: Option<String>,
    pub(crate) latest_verification_at: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ProjectGuidanceData {
    pub(crate) total_files: i64,
    pub(crate) accessed_files: i64,
    pub(crate) unused_files: i64,
    pub(crate) latest_snapshot_captured_at: Option<String>,
    pub(crate) verification_runs: Vec<VerificationRun>,
    pub(crate) stats_entries: Vec<StatsEntry>,
}

pub(crate) fn latest_verification_runs_for_project(db: &Database) -> Vec<VerificationRun> {
    verification::get_latest_verification_runs(db).unwrap_or_default()
}

pub(crate) fn now_unix_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn execution_strategy_repo_truth_summary(project_recommendations: &[Value]) -> Value {
    let mut projects_with_repo_truth_gaps = 0_u64;
    let mut repo_truth_gap_distribution = Map::new();
    let mut mandatory_shell_check_examples = Vec::new();

    for recommendation in project_recommendations {
        let gaps = recommendation["repo_truth_gaps"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if !gaps.is_empty() {
            projects_with_repo_truth_gaps += 1;
        }
        for gap in gaps {
            if let Some(key) = gap.as_str() {
                let next = repo_truth_gap_distribution
                    .get(key)
                    .and_then(|value| value.as_u64())
                    .unwrap_or(0)
                    + 1;
                repo_truth_gap_distribution.insert(key.to_string(), json!(next));
            }
        }
        if let Some(checks) = recommendation["mandatory_shell_checks"].as_array() {
            for check in checks {
                if let Some(cmd) = check.as_str() {
                    if !mandatory_shell_check_examples
                        .iter()
                        .any(|item: &String| item == cmd)
                    {
                        mandatory_shell_check_examples.push(cmd.to_string());
                    }
                }
            }
        }
    }

    json!({
        "projects_with_repo_truth_gaps": projects_with_repo_truth_gaps,
        "repo_truth_gap_distribution": repo_truth_gap_distribution,
        "mandatory_shell_check_examples": mandatory_shell_check_examples,
    })
}

pub(crate) fn agent_guidance_payload(
    project_count: usize,
    monitoring_count: usize,
    monitored_projects: &[String],
    notes: &[String],
    project_recommendations: &[Value],
    project_overviews: &[Value],
) -> Value {
    let has_failing_verification = project_overviews.iter().any(|p| {
        p["verification_evidence"]["failing_runs"]
            .as_array()
            .map(|runs| !runs.is_empty())
            .unwrap_or(false)
    });
    let has_mid_operation_repo = project_overviews.iter().any(|p| {
        p["repo_status_risk"]["operation_states"]
            .as_array()
            .map(|states| !states.is_empty())
            .unwrap_or(false)
    });
    let missing_verification_projects = project_overviews
        .iter()
        .filter(|p| p["verification_evidence"]["status"] == "not_recorded")
        .count();
    let projects_not_ready_for_cleanup = project_overviews
        .iter()
        .filter(|p| !p["safe_for_cleanup"].as_bool().unwrap_or(false))
        .count();
    let projects_not_ready_for_refactor = project_overviews
        .iter()
        .filter(|p| !p["safe_for_refactor"].as_bool().unwrap_or(false))
        .count();
    let projects_with_hardcoded_data = project_overviews
        .iter()
        .filter(|p| {
            p["mock_data_summary"]["hardcoded_candidate_count"]
                .as_u64()
                .unwrap_or(0)
                > 0
        })
        .count();
    let projects_missing_snapshot = project_overviews
        .iter()
        .filter(|p| p["observation"]["freshness"]["snapshot"]["status"] == "missing")
        .count();
    let projects_with_stale_snapshot = project_overviews
        .iter()
        .filter(|p| {
            matches!(
                p["observation"]["freshness"]["snapshot"]["status"]
                    .as_str()
                    .unwrap_or(""),
                "stale" | "unknown"
            )
        })
        .count();
    let projects_missing_activity = project_overviews
        .iter()
        .filter(|p| p["observation"]["freshness"]["activity"]["status"] == "missing")
        .count();
    let projects_with_stale_activity = project_overviews
        .iter()
        .filter(|p| {
            matches!(
                p["observation"]["freshness"]["activity"]["status"]
                    .as_str()
                    .unwrap_or(""),
                "stale" | "unknown"
            )
        })
        .count();
    let projects_missing_verification = project_overviews
        .iter()
        .filter(|p| p["observation"]["freshness"]["verification"]["status"] == "missing")
        .count();
    let projects_with_stale_verification = project_overviews
        .iter()
        .filter(|p| {
            matches!(
                p["observation"]["freshness"]["verification"]["status"]
                    .as_str()
                    .unwrap_or(""),
                "stale" | "unknown"
            )
        })
        .count();
    let storage_maintenance = storage_maintenance_layer(project_overviews);
    let projects_with_storage_maintenance_candidates = storage_maintenance
        ["projects_with_candidates"]
        .as_u64()
        .unwrap_or(0);
    let projects_with_vacuum_candidates = storage_maintenance["projects_with_vacuum_candidates"]
        .as_u64()
        .unwrap_or(0);
    let workspace_strategy = workspace_strategy_profile(
        project_count,
        monitoring_count,
        has_failing_verification,
        has_mid_operation_repo,
        missing_verification_projects,
    );
    let sorted_project_recommendations =
        sort_project_recommendations(project_recommendations, project_overviews);
    let repo_truth_summary = execution_strategy_repo_truth_summary(&sorted_project_recommendations);
    let recommended_flow = agent_guidance_recommended_flow(
        project_count,
        monitoring_count,
        sorted_project_recommendations.first(),
        &workspace_strategy,
    );

    let mut value = json!({
        "guidance": {
            "schema_version": MCP_GUIDANCE_V1,
            "project_count": project_count,
            "monitoring_count": monitoring_count,
            "monitored_projects": monitored_projects,
            "recommended_flow": recommended_flow,
            "when_to_use_opendog": [
                "Use opendog MCP tools when deciding what files are active, unused, or currently monitored.",
                "Use opendog before cleanup or refactoring decisions that depend on real file activity."
            ],
            "when_to_use_shell": [
                "Use `rg` for repository-wide code search and symbol discovery.",
                "Use `git status`, `git diff`, and `git log` for change inspection.",
                "Use project-native test commands such as `cargo test`, `cargo clippy`, `npm test`, or `pytest` to verify behavior."
            ],
            "example_commands": [
                "opendog list",
                "opendog start --id <project>",
                "opendog stats --id <project>",
                "rg \"<pattern>\" .",
                "cargo test"
            ],
            "notes": notes,
            "project_recommendations": sorted_project_recommendations,
            "layers": base_guidance_layers(),
        }
    });
    value["guidance"]["layers"]["workspace_observation"] = json!({
        "status": "available",
        "project_count": project_count,
        "monitoring_count": monitoring_count,
        "analysis_state": if project_count == 0 {
            "empty"
        } else if monitoring_count == 0 {
            "insufficient_activity"
        } else if projects_with_stale_snapshot > 0
            || projects_with_stale_activity > 0
            || projects_with_stale_verification > 0
        {
            "stale"
        } else {
            "ready"
        },
        "projects_missing_snapshot": projects_missing_snapshot,
        "projects_with_stale_snapshot": projects_with_stale_snapshot,
        "projects_missing_activity": projects_missing_activity,
        "projects_with_stale_activity": projects_with_stale_activity,
        "projects_missing_verification": projects_missing_verification,
        "projects_with_stale_verification": projects_with_stale_verification,
        "projects_with_storage_maintenance_candidates": projects_with_storage_maintenance_candidates,
        "projects_with_vacuum_candidates": projects_with_vacuum_candidates,
        "total_storage_reclaimable_bytes": storage_maintenance["total_approx_reclaimable_bytes"].clone(),
        "notes": notes,
    });
    value["guidance"]["layers"]["execution_strategy"] = json!({
        "status": "available",
        "recommended_flow": recommended_flow,
        "project_recommendations": sorted_project_recommendations,
        "global_strategy_mode": workspace_strategy["global_strategy_mode"].clone(),
        "preferred_primary_tool": workspace_strategy["preferred_primary_tool"].clone(),
        "preferred_secondary_tool": workspace_strategy["preferred_secondary_tool"].clone(),
        "evidence_priority": workspace_strategy["evidence_priority"].clone(),
        "when_to_use_opendog": [
            "Choose OPENDOG when deciding which files are active, unused, or should be reviewed first.",
        ],
        "when_to_use_shell": [
            "Choose shell commands for git state, diffs, tests, lint, and builds.",
        ],
        "guardrails": [
            "Do not recommend broad cleanup or refactor work while recorded verification is failing.",
            "Do not recommend broad changes while a repository is mid-merge, rebase, cherry-pick, or bisect.",
            "When verification is missing, prefer running and recording test/lint/build evidence before high-risk edits.",
            "When snapshot, activity, or verification evidence is stale, refresh it before trusting OPENDOG-driven sequencing.",
        ],
        "projects_not_ready_for_cleanup": projects_not_ready_for_cleanup,
        "projects_not_ready_for_refactor": projects_not_ready_for_refactor,
        "projects_with_hardcoded_data_candidates": projects_with_hardcoded_data,
        "projects_missing_snapshot": projects_missing_snapshot,
        "projects_with_stale_snapshot": projects_with_stale_snapshot,
        "projects_missing_activity": projects_missing_activity,
        "projects_with_stale_activity": projects_with_stale_activity,
        "projects_missing_verification": projects_missing_verification,
        "projects_with_stale_verification": projects_with_stale_verification,
        "projects_with_storage_maintenance_candidates": projects_with_storage_maintenance_candidates,
        "projects_with_vacuum_candidates": projects_with_vacuum_candidates,
        "review_opendog_retention_before_large_cleanup": projects_with_storage_maintenance_candidates > 0,
        "recommend_manual_review_for_hardcoded_data": projects_with_hardcoded_data > 0,
        "projects_with_repo_truth_gaps": repo_truth_summary["projects_with_repo_truth_gaps"].clone(),
        "repo_truth_gap_distribution": repo_truth_summary["repo_truth_gap_distribution"].clone(),
        "mandatory_shell_check_examples": repo_truth_summary["mandatory_shell_check_examples"].clone(),
    });
    value["guidance"]["layers"]["multi_project_portfolio"] = json!({
        "status": "available",
        "project_count": project_count,
        "monitoring_count": monitoring_count,
        "monitored_projects": monitored_projects,
        "priority_candidates": sorted_project_recommendations,
        "project_overviews": project_overviews,
    });
    value["guidance"]["layers"]["multi_project_portfolio"] =
        workspace_portfolio_layer(project_overviews);
    value["guidance"]["layers"]["multi_project_portfolio"]["monitoring_count"] =
        json!(monitoring_count);
    value["guidance"]["layers"]["multi_project_portfolio"]["monitored_projects"] =
        json!(monitored_projects);
    value["guidance"]["layers"]["multi_project_portfolio"]["priority_candidates"] =
        json!(sorted_project_recommendations);
    value["guidance"]["layers"]["multi_project_portfolio"]["project_overviews"] =
        json!(project_overviews);
    value["guidance"]["layers"]["multi_project_portfolio"]
        ["projects_with_hardcoded_data_candidates"] = json!(projects_with_hardcoded_data);
    value["guidance"]["layers"]["storage_maintenance"] = storage_maintenance;
    value["guidance"]["layers"]["verification_evidence"] =
        workspace_verification_evidence_layer(project_overviews, project_count, monitoring_count);
    value["guidance"]["layers"]["project_toolchain"] = workspace_toolchain_layer(project_overviews);
    let mut constraints = build_constraints_boundaries_layer(
        None,
        None,
        vec!["This workspace summary is based on registered projects and OPENDOG monitoring state."
            .to_string()],
        vec![
            "Priority recommendations are advisory and should be combined with repository-specific verification."
                .to_string(),
        ],
        vec![
            "Verification evidence is only as current as the latest recorded test/lint/build results."
                .to_string(),
        ],
        default_shell_verification_commands(),
    );
    constraints["projects_not_ready_for_cleanup"] = json!(projects_not_ready_for_cleanup);
    constraints["projects_not_ready_for_refactor"] = json!(projects_not_ready_for_refactor);
    constraints["projects_with_hardcoded_data_candidates"] = json!(projects_with_hardcoded_data);
    constraints["projects_missing_snapshot"] = json!(projects_missing_snapshot);
    constraints["projects_with_stale_snapshot"] = json!(projects_with_stale_snapshot);
    constraints["projects_missing_activity"] = json!(projects_missing_activity);
    constraints["projects_with_stale_activity"] = json!(projects_with_stale_activity);
    constraints["projects_missing_verification"] = json!(projects_missing_verification);
    constraints["projects_with_stale_verification"] = json!(projects_with_stale_verification);
    constraints["projects_with_storage_maintenance_candidates"] =
        json!(projects_with_storage_maintenance_candidates);
    value["guidance"]["layers"]["constraints_boundaries"] = constraints;
    value
}
