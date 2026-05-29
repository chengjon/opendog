use serde_json::Value;

use super::super::guidance_types::{
    DataRiskFocusDistribution, DataRiskFocusSummary, ObservationSummary, RecommendedNextAction,
    RepoRiskCoupling, RepoRiskFinding, RepoRiskPreferredTool, RepoRiskStrategyMode,
    RepoTruthGapDistribution, RepoTruthSummary, StabilizationSummary, VerificationSummary,
};
pub(super) fn execution_strategy_repo_truth_summary(
    project_recommendations: &[Value],
) -> RepoTruthSummary {
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

pub(super) fn execution_strategy_repo_risk_coupling(
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

pub(super) fn execution_strategy_stabilization_summary(
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

pub(super) fn execution_strategy_verification_summary(
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

pub(super) fn execution_strategy_observation_summary(
    project_recommendations: &[Value],
) -> ObservationSummary {
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

pub(super) fn execution_strategy_data_risk_focus_summary(
    project_overviews: &[Value],
) -> DataRiskFocusSummary {
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
