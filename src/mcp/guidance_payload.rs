use serde_json::{json, Value};

use crate::contracts::MCP_GUIDANCE_V1;
use crate::core::verification;
use crate::storage::database::Database;
use crate::storage::queries::{StatsEntry, VerificationRun};

use super::{
    agent_guidance_recommended_flow, base_guidance_layers, build_constraints_boundaries_layer,
    default_shell_verification_commands, external_truth_boundary_for_top_project,
    guidance_types::{
        DataRiskFocusDistribution, DataRiskFocusSummary, ExecutionStrategyLayer,
        ObservationSummary, RecommendedNextAction, RepoRiskCoupling, RepoRiskFinding,
        RepoRiskPreferredTool, RepoRiskStrategyMode, RepoTruthGapDistribution, RepoTruthSummary,
        StabilizationSummary, VerificationSummary, WorkspaceObservationLayer,
    },
    review_focus_projection_for_top_project,
    serialization::to_value_or_error,
    sort_project_recommendations, storage_maintenance_layer, workspace_portfolio_layer,
    workspace_strategy_profile, workspace_toolchain_layer, workspace_verification_evidence_layer,
    WorkspaceCounts,
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

fn string_field(value: &Value, fallback: &str) -> String {
    value.as_str().unwrap_or(fallback).to_string()
}

fn string_list_field(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn execution_strategy_repo_truth_summary(project_recommendations: &[Value]) -> RepoTruthSummary {
    let mut projects_with_repo_truth_gaps = 0_u64;
    let mut repo_truth_gap_distribution = RepoTruthGapDistribution::default();
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
                repo_truth_gap_distribution.increment_gap(key);
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

    RepoTruthSummary {
        projects_with_repo_truth_gaps,
        repo_truth_gap_distribution,
        mandatory_shell_check_examples,
    }
}

fn execution_strategy_repo_risk_coupling(
    project_recommendations: &[Value],
    project_overviews: &[Value],
    workspace_strategy: &Value,
) -> RepoRiskCoupling {
    let recommended_next_action = project_recommendations
        .first()
        .and_then(|recommendation| recommendation["recommended_next_action"].as_str())
        .map(RecommendedNextAction::from_action);
    let strategy_mode = workspace_strategy["global_strategy_mode"]
        .as_str()
        .map(RepoRiskStrategyMode::from_mode);
    let preferred_primary_tool = workspace_strategy["preferred_primary_tool"]
        .as_str()
        .map(RepoRiskPreferredTool::from_tool);

    let no_signal = || {
        RepoRiskCoupling::no_signal(
            recommended_next_action.clone(),
            strategy_mode.clone(),
            preferred_primary_tool.clone(),
        )
    };

    let Some(recommendation) = project_recommendations.first() else {
        return no_signal();
    };
    let Some(project_id) = recommendation["project_id"].as_str() else {
        return no_signal();
    };
    let Some(overview) = project_overviews
        .iter()
        .find(|overview| overview["project_id"].as_str() == Some(project_id))
    else {
        return no_signal();
    };

    let Some(primary_repo_risk_finding) =
        RepoRiskFinding::from_value(&overview["repo_status_risk"]["highest_priority_finding"])
    else {
        return no_signal();
    };

    let strategy_mode_text = strategy_mode
        .as_ref()
        .map(RepoRiskStrategyMode::as_str)
        .unwrap_or("current_strategy")
        .to_string();
    let preferred_primary_tool_text = preferred_primary_tool
        .as_ref()
        .map(RepoRiskPreferredTool::as_str)
        .unwrap_or("current_tool")
        .to_string();

    RepoRiskCoupling::coupled(
        project_id,
        recommendation["recommended_next_action"]
            .as_str()
            .map(RecommendedNextAction::from_action),
        strategy_mode,
        preferred_primary_tool,
        primary_repo_risk_finding,
        format!(
            "Top repository risk keeps the workspace in {} mode and {}-first handling.",
            strategy_mode_text, preferred_primary_tool_text
        ),
    )
}

fn execution_strategy_stabilization_summary(
    project_recommendations: &[Value],
) -> StabilizationSummary {
    let mut project_ids = Vec::new();

    for recommendation in project_recommendations {
        if recommendation["recommended_next_action"] == "stabilize_repository_state"
            && !recommendation["execution_sequence"].is_null()
        {
            if let Some(project_id) = recommendation["project_id"].as_str() {
                project_ids.push(project_id.to_string());
            }
        }
    }

    StabilizationSummary {
        projects_requiring_repo_stabilization: project_ids.len() as u64,
        repo_stabilization_priority_projects: project_ids,
    }
}

fn execution_strategy_verification_summary(
    project_recommendations: &[Value],
) -> VerificationSummary {
    let projects_requiring_verification_run = project_recommendations
        .iter()
        .filter(|recommendation| {
            recommendation["recommended_next_action"] == "run_verification_before_high_risk_changes"
                && recommendation["execution_sequence"]["mode"]
                    == "run_project_verification_then_resume"
        })
        .count() as u64;

    let projects_requiring_failing_verification_repair = project_recommendations
        .iter()
        .filter(|recommendation| {
            recommendation["recommended_next_action"] == "review_failing_verification"
                && recommendation["execution_sequence"]["mode"]
                    == "resolve_failing_verification_then_resume"
        })
        .count() as u64;

    VerificationSummary {
        projects_requiring_verification_run,
        projects_requiring_failing_verification_repair,
    }
}

fn execution_strategy_observation_summary(project_recommendations: &[Value]) -> ObservationSummary {
    let projects_requiring_monitor_start = project_recommendations
        .iter()
        .filter(|recommendation| {
            recommendation["recommended_next_action"] == "start_monitor"
                && recommendation["execution_sequence"]["mode"] == "start_monitor_then_resume"
        })
        .count() as u64;

    let projects_requiring_snapshot_refresh = project_recommendations
        .iter()
        .filter(|recommendation| {
            recommendation["recommended_next_action"] == "take_snapshot"
                && recommendation["execution_sequence"]["mode"] == "refresh_snapshot_then_resume"
        })
        .count() as u64;

    let projects_requiring_activity_generation = project_recommendations
        .iter()
        .filter(|recommendation| {
            recommendation["recommended_next_action"] == "generate_activity_then_stats"
                && recommendation["execution_sequence"]["mode"] == "generate_activity_then_resume"
        })
        .count() as u64;

    ObservationSummary {
        projects_requiring_monitor_start,
        projects_requiring_snapshot_refresh,
        projects_requiring_activity_generation,
    }
}

fn execution_strategy_data_risk_focus_summary(project_overviews: &[Value]) -> DataRiskFocusSummary {
    let mut distribution = DataRiskFocusDistribution::default();
    let mut projects_requiring_hardcoded_review = 0_u64;
    let mut projects_requiring_mock_review = 0_u64;
    let mut projects_requiring_mixed_file_review = 0_u64;

    for overview in project_overviews {
        let focus = overview["mock_data_summary"]["data_risk_focus"]["primary_focus"]
            .as_str()
            .unwrap_or("none");
        distribution.increment_focus(focus);

        match focus {
            "hardcoded" => {
                projects_requiring_hardcoded_review += 1;
            }
            "mixed" => {
                projects_requiring_mixed_file_review += 1;
            }
            "mock" => {
                projects_requiring_mock_review += 1;
            }
            _ => {}
        }
    }

    DataRiskFocusSummary {
        data_risk_focus_distribution: distribution,
        projects_requiring_hardcoded_review,
        projects_requiring_mock_review,
        projects_requiring_mixed_file_review,
    }
}

#[cfg(test)]
pub(crate) fn default_governance_layer() -> Value {
    serde_json::json!({
        "has_governance_state": false,
        "project_governance": [],
        "workspace_summary": {
            "total_active_lanes": 0,
            "total_active_nodes": 0,
            "projects_with_governance": 0,
            "projects_without_governance": 0,
        }
    })
}

pub(crate) fn agent_guidance_payload(
    project_count: usize,
    monitoring_count: usize,
    monitored_projects: &[String],
    notes: &[String],
    project_recommendations: &[Value],
    project_overviews: &[Value],
    governance: Value,
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
    let verification_summary =
        execution_strategy_verification_summary(&sorted_project_recommendations);
    let observation_summary =
        execution_strategy_observation_summary(&sorted_project_recommendations);
    let repo_truth_summary = execution_strategy_repo_truth_summary(&sorted_project_recommendations);
    let stabilization_summary =
        execution_strategy_stabilization_summary(&sorted_project_recommendations);
    let data_risk_focus_summary = execution_strategy_data_risk_focus_summary(project_overviews);
    let risk_strategy_coupling = execution_strategy_repo_risk_coupling(
        &sorted_project_recommendations,
        project_overviews,
        &workspace_strategy,
    );
    let risk_strategy_coupling_value = risk_strategy_coupling.to_value();
    let external_truth_boundary =
        external_truth_boundary_for_top_project(sorted_project_recommendations.first());
    let review_focus_projection =
        review_focus_projection_for_top_project(sorted_project_recommendations.first());
    let recommended_flow = agent_guidance_recommended_flow(
        project_count,
        monitoring_count,
        sorted_project_recommendations.first(),
        &workspace_strategy,
        Some(&risk_strategy_coupling_value),
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
    let analysis_state = if project_count == 0 {
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
    };

    value["guidance"]["layers"]["workspace_observation"] = to_value_or_error(
        "WorkspaceObservationLayer",
        WorkspaceObservationLayer {
            status: "available".to_string(),
            project_count,
            monitoring_count,
            analysis_state: analysis_state.to_string(),
            projects_missing_snapshot,
            projects_with_stale_snapshot,
            projects_missing_activity,
            projects_with_stale_activity,
            projects_missing_verification,
            projects_with_stale_verification,
            projects_with_storage_maintenance_candidates,
            projects_with_vacuum_candidates,
            total_storage_reclaimable_bytes: storage_maintenance["total_approx_reclaimable_bytes"]
                .clone(),
            data_risk_focus_distribution: data_risk_focus_summary
                .data_risk_focus_distribution
                .to_value(),
            projects_requiring_hardcoded_review: json!(
                data_risk_focus_summary.projects_requiring_hardcoded_review
            ),
            projects_requiring_mock_review: json!(
                data_risk_focus_summary.projects_requiring_mock_review
            ),
            projects_requiring_mixed_file_review: json!(
                data_risk_focus_summary.projects_requiring_mixed_file_review
            ),
            notes: notes.to_vec(),
        },
    );
    value["guidance"]["layers"]["execution_strategy"] =
        to_value_or_error("ExecutionStrategyLayer", ExecutionStrategyLayer {
            status: "available".to_string(),
            recommended_flow: string_list_field(&recommended_flow),
            project_recommendations: sorted_project_recommendations.clone(),
            global_strategy_mode: RepoRiskStrategyMode::from_mode(&string_field(
                &workspace_strategy["global_strategy_mode"],
                "current_strategy",
            )),
            preferred_primary_tool: string_field(
                &workspace_strategy["preferred_primary_tool"],
                "current_tool",
            ),
            preferred_secondary_tool: string_field(
                &workspace_strategy["preferred_secondary_tool"],
                "shell",
            ),
            evidence_priority: string_list_field(&workspace_strategy["evidence_priority"]),
            risk_strategy_coupling,
            external_truth_boundary,
            review_focus_projection,
            when_to_use_opendog: vec![
                "Choose OPENDOG when deciding which files are active, unused, or should be reviewed first.",
            ],
            when_to_use_shell: vec![
                "Choose shell commands for git state, diffs, tests, lint, and builds.",
            ],
            guardrails: vec![
                "Do not recommend broad cleanup or refactor work while recorded verification is failing.",
                "Do not recommend broad changes while a repository is mid-merge, rebase, cherry-pick, or bisect.",
                "When verification is missing, prefer running and recording test/lint/build evidence before high-risk edits.",
                "When snapshot, activity, or verification evidence is stale, refresh it before trusting OPENDOG-driven sequencing.",
            ],
            projects_not_ready_for_cleanup,
            projects_not_ready_for_refactor,
            projects_with_hardcoded_data_candidates: projects_with_hardcoded_data,
            projects_missing_snapshot,
            projects_with_stale_snapshot,
            projects_missing_activity,
            projects_with_stale_activity,
            projects_missing_verification,
            projects_with_stale_verification,
            projects_with_storage_maintenance_candidates,
            projects_with_vacuum_candidates,
            review_opendog_retention_before_large_cleanup:
                projects_with_storage_maintenance_candidates > 0,
            recommend_manual_review_for_hardcoded_data: projects_with_hardcoded_data > 0,
            data_risk_focus_distribution: data_risk_focus_summary
                .data_risk_focus_distribution
                .clone(),
            projects_requiring_hardcoded_review: data_risk_focus_summary
                .projects_requiring_hardcoded_review,
            projects_requiring_mock_review: data_risk_focus_summary.projects_requiring_mock_review,
            projects_requiring_mixed_file_review: data_risk_focus_summary
                .projects_requiring_mixed_file_review,
            projects_requiring_monitor_start: observation_summary.projects_requiring_monitor_start,
            projects_requiring_snapshot_refresh: observation_summary
                .projects_requiring_snapshot_refresh,
            projects_requiring_activity_generation: observation_summary
                .projects_requiring_activity_generation,
            projects_with_repo_truth_gaps: repo_truth_summary.projects_with_repo_truth_gaps,
            repo_truth_gap_distribution: repo_truth_summary.repo_truth_gap_distribution,
            mandatory_shell_check_examples: repo_truth_summary.mandatory_shell_check_examples,
            projects_requiring_verification_run: verification_summary.projects_requiring_verification_run,
            projects_requiring_failing_verification_repair: verification_summary
                .projects_requiring_failing_verification_repair,
            projects_requiring_repo_stabilization: stabilization_summary
                .projects_requiring_repo_stabilization,
            repo_stabilization_priority_projects: stabilization_summary
                .repo_stabilization_priority_projects,
        });
    value["guidance"]["layers"]["multi_project_portfolio"] = to_value_or_error(
        "WorkspacePortfolioLayer",
        workspace_portfolio_layer(
            project_overviews,
            monitoring_count,
            monitored_projects,
            sorted_project_recommendations,
            projects_with_hardcoded_data,
        ),
    );
    value["guidance"]["layers"]["storage_maintenance"] = storage_maintenance;
    value["guidance"]["layers"]["verification_evidence"] =
        workspace_verification_evidence_layer(project_overviews, project_count, monitoring_count);
    value["guidance"]["layers"]["project_toolchain"] = workspace_toolchain_layer(project_overviews);
    value["guidance"]["layers"]["constraints_boundaries"] = build_constraints_boundaries_layer(
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
        Some(WorkspaceCounts {
            projects_not_ready_for_cleanup,
            projects_not_ready_for_refactor,
            projects_with_hardcoded_data_candidates: projects_with_hardcoded_data,
            projects_missing_snapshot,
            projects_with_stale_snapshot,
            projects_missing_activity,
            projects_with_stale_activity,
            projects_missing_verification,
            projects_with_stale_verification,
            projects_with_storage_maintenance_candidates,
        }),
    );
    value["guidance"]["layers"]["governance"] = governance;
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---------------------------------------------------------------------------
    // execution_strategy_repo_truth_summary
    // ---------------------------------------------------------------------------

    #[test]
    fn repo_truth_summary_empty_input() {
        let summary = execution_strategy_repo_truth_summary(&[]);
        assert_eq!(summary.projects_with_repo_truth_gaps, 0);
        assert_eq!(
            serde_json::to_value(&summary.repo_truth_gap_distribution).unwrap(),
            json!({})
        );
        assert!(summary.mandatory_shell_check_examples.is_empty());
    }

    #[test]
    fn repo_truth_summary_empty_gaps() {
        let recs = vec![json!({"repo_truth_gaps": []})];
        let summary = execution_strategy_repo_truth_summary(&recs);
        assert_eq!(summary.projects_with_repo_truth_gaps, 0);
        assert_eq!(
            serde_json::to_value(&summary.repo_truth_gap_distribution).unwrap(),
            json!({})
        );
        assert!(summary.mandatory_shell_check_examples.is_empty());
    }

    #[test]
    fn repo_truth_summary_single_gap_with_shell_checks() {
        let recs = vec![json!({
            "repo_truth_gaps": ["missing_test"],
            "mandatory_shell_checks": ["cargo test"]
        })];
        let summary = execution_strategy_repo_truth_summary(&recs);
        assert_eq!(summary.projects_with_repo_truth_gaps, 1);
        assert_eq!(summary.repo_truth_gap_distribution.count("missing_test"), 1);
        assert_eq!(summary.mandatory_shell_check_examples, vec!["cargo test"]);
    }

    #[test]
    fn repo_truth_summary_two_same_gap_key() {
        let recs = vec![
            json!({"repo_truth_gaps": ["missing_test"]}),
            json!({"repo_truth_gaps": ["missing_test"]}),
        ];
        let summary = execution_strategy_repo_truth_summary(&recs);
        assert_eq!(summary.projects_with_repo_truth_gaps, 2);
        assert_eq!(summary.repo_truth_gap_distribution.count("missing_test"), 2);
    }

    #[test]
    fn repo_truth_summary_non_string_gaps_skipped() {
        let recs = vec![json!({
            "repo_truth_gaps": [42, true, null, {"a": 1}]
        })];
        let summary = execution_strategy_repo_truth_summary(&recs);
        // The array is non-empty so the project counts as having gaps
        assert_eq!(summary.projects_with_repo_truth_gaps, 1);
        // But none of the entries are strings, so distribution is empty
        assert_eq!(
            serde_json::to_value(&summary.repo_truth_gap_distribution).unwrap(),
            json!({})
        );
    }

    #[test]
    fn repo_truth_summary_duplicate_shell_checks_deduplicated() {
        let recs = vec![
            json!({"repo_truth_gaps": [], "mandatory_shell_checks": ["cargo test", "cargo clippy"]}),
            json!({"repo_truth_gaps": [], "mandatory_shell_checks": ["cargo test"]}),
        ];
        let summary = execution_strategy_repo_truth_summary(&recs);
        assert_eq!(
            summary.mandatory_shell_check_examples,
            vec!["cargo test", "cargo clippy"]
        );
    }

    // ---------------------------------------------------------------------------
    // execution_strategy_repo_risk_coupling
    // ---------------------------------------------------------------------------

    #[test]
    fn risk_coupling_empty_recommendations() {
        let ws = json!({"global_strategy_mode": "defensive", "preferred_primary_tool": "opendog"});
        let result = execution_strategy_repo_risk_coupling(&[], &[], &ws).to_value();
        assert_eq!(result["status"], "no_repo_risk_signal");
        assert!(result["source"].is_null());
    }

    #[test]
    fn risk_coupling_no_project_id() {
        let recs = vec![json!({"recommended_next_action": "start_monitor"})];
        let ws = json!({"global_strategy_mode": "defensive", "preferred_primary_tool": "opendog"});
        let result = execution_strategy_repo_risk_coupling(&recs, &[], &ws).to_value();
        assert_eq!(result["status"], "no_repo_risk_signal");
    }

    #[test]
    fn risk_coupling_no_matching_overview() {
        let recs =
            vec![json!({"project_id": "proj_a", "recommended_next_action": "start_monitor"})];
        let overviews = vec![json!({"project_id": "proj_b"})];
        let ws = json!({"global_strategy_mode": "defensive", "preferred_primary_tool": "opendog"});
        let result = execution_strategy_repo_risk_coupling(&recs, &overviews, &ws).to_value();
        assert_eq!(result["status"], "no_repo_risk_signal");
    }

    #[test]
    fn risk_coupling_null_risk_finding() {
        let recs =
            vec![json!({"project_id": "proj_a", "recommended_next_action": "start_monitor"})];
        let overviews = vec![json!({
            "project_id": "proj_a",
            "repo_status_risk": {"highest_priority_finding": null}
        })];
        let ws = json!({"global_strategy_mode": "defensive", "preferred_primary_tool": "opendog"});
        let result = execution_strategy_repo_risk_coupling(&recs, &overviews, &ws).to_value();
        assert_eq!(result["status"], "no_repo_risk_signal");
    }

    #[test]
    fn risk_coupling_full_coupling() {
        let recs = vec![json!({
            "project_id": "proj_a",
            "recommended_next_action": "stabilize_repository_state"
        })];
        let overviews = vec![json!({
            "project_id": "proj_a",
            "repo_status_risk": {
                "highest_priority_finding": {
                    "kind": "repository_operation_in_progress",
                    "severity": "high",
                    "priority": "immediate",
                    "confidence": "high",
                    "summary": "Repository is mid-operation: rebase.",
                    "evidence": ["Git metadata indicates an in-progress operation: rebase."],
                    "source": "git_metadata"
                }
            }
        })];
        let ws = json!({
            "global_strategy_mode": "defensive",
            "preferred_primary_tool": "opendog"
        });
        let result = execution_strategy_repo_risk_coupling(&recs, &overviews, &ws).to_value();
        assert_eq!(result["status"], "coupled");
        assert_eq!(result["source"], "primary_repo_risk_finding");
        assert_eq!(result["source_project_id"], "proj_a");
        assert_eq!(
            result["primary_repo_risk_finding"]["kind"],
            "repository_operation_in_progress"
        );
    }

    #[test]
    fn risk_coupling_summary_includes_strategy_fields() {
        let recs = vec![json!({
            "project_id": "proj_a",
            "recommended_next_action": "stabilize_repository_state"
        })];
        let overviews = vec![json!({
            "project_id": "proj_a",
            "repo_status_risk": {
                "highest_priority_finding": {
                    "kind": "conflicted_paths",
                    "severity": "high",
                    "priority": "immediate",
                    "confidence": "high",
                    "summary": "1 conflicted paths detected in the working tree.",
                    "evidence": ["git status reported 1 conflicted paths."],
                    "source": "git_status"
                }
            }
        })];
        let ws = json!({
            "global_strategy_mode": "stabilize_first",
            "preferred_primary_tool": "shell_verification"
        });
        let result = execution_strategy_repo_risk_coupling(&recs, &overviews, &ws).to_value();
        let summary = result["summary"].as_str().unwrap();
        assert!(
            summary.contains("stabilize_first"),
            "summary should contain strategy_mode: {summary}"
        );
        assert!(
            summary.contains("shell_verification"),
            "summary should contain preferred_primary_tool: {summary}"
        );
    }

    // ---------------------------------------------------------------------------
    // execution_strategy_stabilization_summary
    // ---------------------------------------------------------------------------

    #[test]
    fn stabilization_summary_empty() {
        let summary = execution_strategy_stabilization_summary(&[]);
        assert_eq!(summary.projects_requiring_repo_stabilization, 0);
        assert!(summary.repo_stabilization_priority_projects.is_empty());
    }

    #[test]
    fn stabilization_summary_matching_action_with_execution_sequence() {
        let recs = vec![json!({
            "project_id": "proj_a",
            "recommended_next_action": "stabilize_repository_state",
            "execution_sequence": {"mode": "resolve_rebase_then_resume"}
        })];
        let summary = execution_strategy_stabilization_summary(&recs);
        assert_eq!(summary.projects_requiring_repo_stabilization, 1);
        assert_eq!(summary.repo_stabilization_priority_projects, vec!["proj_a"]);
    }

    #[test]
    fn stabilization_summary_matching_action_null_execution_sequence() {
        let recs = vec![json!({
            "project_id": "proj_a",
            "recommended_next_action": "stabilize_repository_state",
            "execution_sequence": null
        })];
        let summary = execution_strategy_stabilization_summary(&recs);
        assert_eq!(summary.projects_requiring_repo_stabilization, 0);
    }

    #[test]
    fn stabilization_summary_different_action() {
        let recs = vec![json!({
            "project_id": "proj_a",
            "recommended_next_action": "start_monitor",
            "execution_sequence": {"mode": "start_monitor_then_resume"}
        })];
        let summary = execution_strategy_stabilization_summary(&recs);
        assert_eq!(summary.projects_requiring_repo_stabilization, 0);
    }

    #[test]
    fn stabilization_summary_mixed() {
        let recs = vec![
            json!({
                "project_id": "proj_a",
                "recommended_next_action": "stabilize_repository_state",
                "execution_sequence": {"mode": "resolve_rebase_then_resume"}
            }),
            json!({
                "project_id": "proj_b",
                "recommended_next_action": "start_monitor",
                "execution_sequence": {"mode": "start_monitor_then_resume"}
            }),
            json!({
                "project_id": "proj_c",
                "recommended_next_action": "stabilize_repository_state",
                "execution_sequence": {"mode": "resolve_merge_then_resume"}
            }),
        ];
        let summary = execution_strategy_stabilization_summary(&recs);
        assert_eq!(summary.projects_requiring_repo_stabilization, 2);
        assert_eq!(
            summary.repo_stabilization_priority_projects,
            vec!["proj_a", "proj_c"]
        );
    }

    // ---------------------------------------------------------------------------
    // execution_strategy_verification_summary
    // ---------------------------------------------------------------------------

    #[test]
    fn verification_summary_empty() {
        let summary = execution_strategy_verification_summary(&[]);
        assert_eq!(summary.projects_requiring_verification_run, 0);
        assert_eq!(summary.projects_requiring_failing_verification_repair, 0);
    }

    #[test]
    fn verification_summary_matching_verification_run() {
        let recs = vec![json!({
            "recommended_next_action": "run_verification_before_high_risk_changes",
            "execution_sequence": {"mode": "run_project_verification_then_resume"}
        })];
        let summary = execution_strategy_verification_summary(&recs);
        assert_eq!(summary.projects_requiring_verification_run, 1);
        assert_eq!(summary.projects_requiring_failing_verification_repair, 0);
    }

    #[test]
    fn verification_summary_matching_failing_verification() {
        let recs = vec![json!({
            "recommended_next_action": "review_failing_verification",
            "execution_sequence": {"mode": "resolve_failing_verification_then_resume"}
        })];
        let summary = execution_strategy_verification_summary(&recs);
        assert_eq!(summary.projects_requiring_verification_run, 0);
        assert_eq!(summary.projects_requiring_failing_verification_repair, 1);
    }

    #[test]
    fn verification_summary_wrong_mode() {
        let recs = vec![json!({
            "recommended_next_action": "run_verification_before_high_risk_changes",
            "execution_sequence": {"mode": "wrong_mode"}
        })];
        let summary = execution_strategy_verification_summary(&recs);
        assert_eq!(summary.projects_requiring_verification_run, 0);
        assert_eq!(summary.projects_requiring_failing_verification_repair, 0);
    }

    #[test]
    fn verification_summary_mixed() {
        let recs = vec![
            json!({
                "recommended_next_action": "run_verification_before_high_risk_changes",
                "execution_sequence": {"mode": "run_project_verification_then_resume"}
            }),
            json!({
                "recommended_next_action": "review_failing_verification",
                "execution_sequence": {"mode": "resolve_failing_verification_then_resume"}
            }),
            json!({
                "recommended_next_action": "run_verification_before_high_risk_changes",
                "execution_sequence": {"mode": "run_project_verification_then_resume"}
            }),
            json!({
                "recommended_next_action": "start_monitor",
                "execution_sequence": {"mode": "start_monitor_then_resume"}
            }),
        ];
        let summary = execution_strategy_verification_summary(&recs);
        assert_eq!(summary.projects_requiring_verification_run, 2);
        assert_eq!(summary.projects_requiring_failing_verification_repair, 1);
    }

    // ---------------------------------------------------------------------------
    // execution_strategy_observation_summary
    // ---------------------------------------------------------------------------

    #[test]
    fn observation_summary_empty() {
        let summary = execution_strategy_observation_summary(&[]);
        assert_eq!(summary.projects_requiring_monitor_start, 0);
        assert_eq!(summary.projects_requiring_snapshot_refresh, 0);
        assert_eq!(summary.projects_requiring_activity_generation, 0);
    }

    #[test]
    fn observation_summary_start_monitor() {
        let recs = vec![json!({
            "recommended_next_action": "start_monitor",
            "execution_sequence": {"mode": "start_monitor_then_resume"}
        })];
        let summary = execution_strategy_observation_summary(&recs);
        assert_eq!(summary.projects_requiring_monitor_start, 1);
        assert_eq!(summary.projects_requiring_snapshot_refresh, 0);
        assert_eq!(summary.projects_requiring_activity_generation, 0);
    }

    #[test]
    fn observation_summary_take_snapshot() {
        let recs = vec![json!({
            "recommended_next_action": "take_snapshot",
            "execution_sequence": {"mode": "refresh_snapshot_then_resume"}
        })];
        let summary = execution_strategy_observation_summary(&recs);
        assert_eq!(summary.projects_requiring_monitor_start, 0);
        assert_eq!(summary.projects_requiring_snapshot_refresh, 1);
        assert_eq!(summary.projects_requiring_activity_generation, 0);
    }

    #[test]
    fn observation_summary_generate_activity() {
        let recs = vec![json!({
            "recommended_next_action": "generate_activity_then_stats",
            "execution_sequence": {"mode": "generate_activity_then_resume"}
        })];
        let summary = execution_strategy_observation_summary(&recs);
        assert_eq!(summary.projects_requiring_monitor_start, 0);
        assert_eq!(summary.projects_requiring_snapshot_refresh, 0);
        assert_eq!(summary.projects_requiring_activity_generation, 1);
    }

    #[test]
    fn observation_summary_wrong_mode() {
        let recs = vec![json!({
            "recommended_next_action": "start_monitor",
            "execution_sequence": {"mode": "wrong_mode"}
        })];
        let summary = execution_strategy_observation_summary(&recs);
        assert_eq!(summary.projects_requiring_monitor_start, 0);
        assert_eq!(summary.projects_requiring_snapshot_refresh, 0);
        assert_eq!(summary.projects_requiring_activity_generation, 0);
    }

    #[test]
    fn observation_summary_mixed() {
        let recs = vec![
            json!({
                "recommended_next_action": "start_monitor",
                "execution_sequence": {"mode": "start_monitor_then_resume"}
            }),
            json!({
                "recommended_next_action": "take_snapshot",
                "execution_sequence": {"mode": "refresh_snapshot_then_resume"}
            }),
            json!({
                "recommended_next_action": "generate_activity_then_stats",
                "execution_sequence": {"mode": "generate_activity_then_resume"}
            }),
            json!({
                "recommended_next_action": "start_monitor",
                "execution_sequence": {"mode": "start_monitor_then_resume"}
            }),
        ];
        let summary = execution_strategy_observation_summary(&recs);
        assert_eq!(summary.projects_requiring_monitor_start, 2);
        assert_eq!(summary.projects_requiring_snapshot_refresh, 1);
        assert_eq!(summary.projects_requiring_activity_generation, 1);
    }

    // ---------------------------------------------------------------------------
    // execution_strategy_data_risk_focus_summary
    // ---------------------------------------------------------------------------

    #[test]
    fn data_risk_summary_empty() {
        let summary = execution_strategy_data_risk_focus_summary(&[]);
        assert_eq!(
            summary.data_risk_focus_distribution.to_value(),
            json!({"hardcoded": 0, "mixed": 0, "mock": 0, "none": 0})
        );
        assert_eq!(summary.projects_requiring_hardcoded_review, 0);
        assert_eq!(summary.projects_requiring_mock_review, 0);
        assert_eq!(summary.projects_requiring_mixed_file_review, 0);
    }

    #[test]
    fn data_risk_summary_hardcoded() {
        let overviews = vec![json!({
            "mock_data_summary": {"data_risk_focus": {"primary_focus": "hardcoded"}}
        })];
        let summary = execution_strategy_data_risk_focus_summary(&overviews);
        assert_eq!(summary.projects_requiring_hardcoded_review, 1);
        assert_eq!(summary.projects_requiring_mock_review, 0);
        assert_eq!(summary.projects_requiring_mixed_file_review, 0);
        assert_eq!(summary.data_risk_focus_distribution.hardcoded, 1);
    }

    #[test]
    fn data_risk_summary_mock() {
        let overviews = vec![json!({
            "mock_data_summary": {"data_risk_focus": {"primary_focus": "mock"}}
        })];
        let summary = execution_strategy_data_risk_focus_summary(&overviews);
        assert_eq!(summary.projects_requiring_hardcoded_review, 0);
        assert_eq!(summary.projects_requiring_mock_review, 1);
        assert_eq!(summary.projects_requiring_mixed_file_review, 0);
        assert_eq!(summary.data_risk_focus_distribution.mock, 1);
    }

    #[test]
    fn data_risk_summary_mixed() {
        let overviews = vec![json!({
            "mock_data_summary": {"data_risk_focus": {"primary_focus": "mixed"}}
        })];
        let summary = execution_strategy_data_risk_focus_summary(&overviews);
        assert_eq!(summary.projects_requiring_hardcoded_review, 0);
        assert_eq!(summary.projects_requiring_mock_review, 0);
        assert_eq!(summary.projects_requiring_mixed_file_review, 1);
        assert_eq!(summary.data_risk_focus_distribution.mixed, 1);
    }

    #[test]
    fn data_risk_summary_none_focus() {
        let overviews = vec![json!({
            "mock_data_summary": {"data_risk_focus": {"primary_focus": "none"}}
        })];
        let summary = execution_strategy_data_risk_focus_summary(&overviews);
        assert_eq!(summary.projects_requiring_hardcoded_review, 0);
        assert_eq!(summary.projects_requiring_mock_review, 0);
        assert_eq!(summary.projects_requiring_mixed_file_review, 0);
        assert_eq!(summary.data_risk_focus_distribution.none, 1);
    }

    #[test]
    fn data_risk_summary_missing_focus() {
        // Missing primary_focus field defaults to "none"
        let overviews = vec![json!({
            "mock_data_summary": {"data_risk_focus": {}}
        })];
        let summary = execution_strategy_data_risk_focus_summary(&overviews);
        assert_eq!(summary.data_risk_focus_distribution.none, 1);
    }

    #[test]
    fn data_risk_summary_mixed_overviews() {
        let overviews = vec![
            json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "hardcoded"}}}),
            json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "mock"}}}),
            json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "hardcoded"}}}),
            json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "mixed"}}}),
            json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "none"}}}),
            json!({"mock_data_summary": {}}),
        ];
        let summary = execution_strategy_data_risk_focus_summary(&overviews);
        assert_eq!(summary.projects_requiring_hardcoded_review, 2);
        assert_eq!(summary.projects_requiring_mock_review, 1);
        assert_eq!(summary.projects_requiring_mixed_file_review, 1);
        assert_eq!(summary.data_risk_focus_distribution.hardcoded, 2);
        assert_eq!(summary.data_risk_focus_distribution.mock, 1);
        assert_eq!(summary.data_risk_focus_distribution.mixed, 1);
        assert_eq!(summary.data_risk_focus_distribution.none, 2);
    }
}
