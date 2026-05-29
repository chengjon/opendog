use serde_json::{json, Value};

pub(crate) mod eligibility;
mod evidence;
mod forced;
pub(crate) mod reasoning;
mod review;
mod review_focus;
pub(crate) mod scoring;
pub(crate) mod sequencing;

use crate::config::{resolve_project_config, ProjectConfig, ProjectInfo};
use crate::core::retention;
use crate::storage::database::Database;
use crate::storage::queries::VerificationRun;

use self::eligibility::{determine_action_eligibility, GateLevel, RecommendationSignals};
use self::evidence::{
    EvidenceCollectionAction, EvidenceCollectionContext, EvidenceCollectionRecommendation,
};
use self::forced::{ForcedProjectRecommendation, ForcedRecommendationContext};
use self::reasoning::{build_reason, derive_confidence};
use self::review::{ProjectReviewAction, ProjectReviewContext, ProjectReviewRecommendation};
use self::review_focus::RecommendationReviewFocus;
use self::scoring::score_review_actions;
use self::sequencing::execution_sequence_for_recommendation;
use super::constraints::repo_truth_gap_projection;
use super::guidance_types::ProjectOverview;
use super::serialization::to_value_or_error;
use super::{
    activity_is_stale, detect_mock_data_report, detect_project_commands,
    enrich_project_overview_with_attention, latest_activity_timestamp,
    latest_verification_timestamp, now_unix_secs, project_observation_layer,
    project_readiness_snapshot, project_storage_maintenance_with_policy, project_toolchain_layer,
    repo_status_risk_layer, snapshot_is_stale, verification_has_failures, verification_is_missing,
    verification_is_stale, verification_status_layer, ProjectGuidanceData, ProjectGuidanceState,
};

pub(crate) fn project_overview(
    project: &ProjectGuidanceState,
    repo_risk: &Value,
    recommendation: &Value,
    verification_layer: &Value,
    mock_data_summary: &Value,
    storage_maintenance: &Value,
) -> Value {
    let readiness = project_readiness_snapshot(repo_risk, verification_layer);
    let observation = project_observation_layer(project);

    let overview = ProjectOverview {
        project_id: project.id.clone(),
        status: project.status.clone(),
        snapshot_available: project.total_files > 0,
        activity_available: project.accessed_files > 0,
        unused_files: project.unused_files,
        observation,
        repo_status_risk: repo_risk.clone(),
        verification_evidence: verification_layer.clone(),
        mock_data_summary: mock_data_summary.clone(),
        storage_maintenance: storage_maintenance.clone(),
        project_toolchain: project_toolchain_layer(&project.root_path),
        verification_safe_for_cleanup: readiness["verification_safe_for_cleanup"].clone(),
        verification_safe_for_refactor: readiness["verification_safe_for_refactor"].clone(),
        verification_gate_levels: json!({
            "cleanup": readiness["cleanup_gate_level"].clone(),
            "refactor": readiness["refactor_gate_level"].clone(),
        }),
        safe_for_cleanup: readiness["safe_for_cleanup"].clone(),
        safe_for_cleanup_reason: readiness["safe_for_cleanup_reason"].clone(),
        cleanup_blockers: readiness["cleanup_blockers"].clone(),
        safe_for_refactor: readiness["safe_for_refactor"].clone(),
        safe_for_refactor_reason: readiness["safe_for_refactor_reason"].clone(),
        refactor_blockers: readiness["refactor_blockers"].clone(),
        recommended_next_action: recommendation["recommended_next_action"].clone(),
        recommended_flow: recommendation["recommended_flow"].clone(),
        recommended_reason: recommendation["reason"].clone(),
        strategy_confidence: recommendation["confidence"].clone(),
    };

    enrich_project_overview_with_attention(&to_value_or_error("ProjectOverview", overview))
}

pub(crate) fn collect_project_guidance_context<F>(
    projects: &[ProjectInfo],
    global_defaults: &ProjectConfig,
    mut load_project_state: F,
) -> (Vec<String>, Vec<Value>, Vec<Value>)
where
    F: FnMut(&ProjectInfo) -> ProjectGuidanceData,
{
    let monitored_projects: Vec<String> = projects
        .iter()
        .filter(|p| p.status == "monitoring")
        .map(|p| p.id.clone())
        .collect();
    let mut recommendations = Vec::new();
    let mut project_overviews = Vec::new();

    for project in projects {
        let guidance_data = load_project_state(project);
        let effective_config = resolve_project_config(global_defaults, &project.config);
        let (storage_metrics, storage_evidence_counts) = if project.db_path.exists() {
            match Database::open_project(&project.db_path) {
                Ok(db) => (
                    retention::collect_storage_metrics(&db).ok(),
                    retention::collect_storage_evidence_counts(&db).ok(),
                ),
                Err(_) => (None, None),
            }
        } else {
            (None, None)
        };
        let guidance_state = ProjectGuidanceState {
            id: project.id.clone(),
            status: project.status.clone(),
            root_path: project.root_path.clone(),
            total_files: guidance_data.total_files,
            accessed_files: guidance_data.accessed_files,
            unused_files: guidance_data.unused_files,
            latest_snapshot_captured_at: guidance_data.latest_snapshot_captured_at.clone(),
            latest_activity_at: latest_activity_timestamp(&guidance_data.stats_entries),
            latest_verification_at: latest_verification_timestamp(&guidance_data.verification_runs),
        };
        let repo_risk = repo_status_risk_layer(&guidance_state.root_path);
        let recommendation = recommend_project_action(
            &guidance_state,
            &repo_risk,
            &guidance_data.verification_runs,
        );
        let verification_layer = verification_status_layer(&guidance_data.verification_runs);
        let mock_data_summary =
            detect_mock_data_report(&guidance_state.root_path, &guidance_data.stats_entries)
                .to_value(3);
        let storage_maintenance = project_storage_maintenance_with_policy(
            storage_metrics.as_ref(),
            storage_evidence_counts.as_ref(),
            &effective_config.retention,
        );
        project_overviews.push(project_overview(
            &guidance_state,
            &repo_risk,
            &recommendation,
            &verification_layer,
            &mock_data_summary,
            &storage_maintenance,
        ));
        recommendations.push(recommendation);
    }

    (monitored_projects, recommendations, project_overviews)
}

pub(crate) fn review_focus_for_action(selected_action: &str, repo_risk: &Value) -> Value {
    RecommendationReviewFocus::from_action(selected_action, repo_risk)
        .map(|focus| focus.to_json())
        .unwrap_or(Value::Null)
}

pub(crate) fn recommend_project_action(
    project: &ProjectGuidanceState,
    repo_risk: &Value,
    verification_runs: &[VerificationRun],
) -> Value {
    let now_secs = now_unix_secs();
    let snapshot_stale = snapshot_is_stale(project, now_secs);
    let activity_stale = activity_is_stale(project, now_secs);
    let verification_stale = verification_is_stale(verification_runs, now_secs);
    let project_commands = detect_project_commands(&project.root_path);
    let project_toolchain = project_toolchain_layer(&project.root_path);
    let verification_layer = verification_status_layer(verification_runs);
    let readiness = project_readiness_snapshot(repo_risk, &verification_layer);
    let cleanup_blockers = readiness["cleanup_blockers"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let refactor_blockers = readiness["refactor_blockers"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let safe_for_cleanup = readiness["safe_for_cleanup"].as_bool().unwrap_or(false);
    let safe_for_refactor = readiness["safe_for_refactor"].as_bool().unwrap_or(false);
    let cleanup_gate_level = readiness["cleanup_gate_level"]
        .as_str()
        .unwrap_or("blocked");
    let refactor_gate_level = readiness["refactor_gate_level"]
        .as_str()
        .unwrap_or("blocked");
    let gate_levels = json!({
        "cleanup": cleanup_gate_level,
        "refactor": refactor_gate_level,
    });
    let (repo_truth_gaps, mandatory_shell_checks) = repo_truth_gap_projection(repo_risk);
    let repo_truth_gaps_json = json!(repo_truth_gaps);
    let mandatory_shell_checks_json = json!(mandatory_shell_checks);
    let signals = RecommendationSignals {
        cleanup_gate_level: GateLevel::from_str(cleanup_gate_level),
        refactor_gate_level: GateLevel::from_str(refactor_gate_level),
        monitoring_active: project.status == "monitoring",
        snapshot_available: project.total_files > 0,
        activity_available: project.accessed_files > 0,
        snapshot_stale,
        activity_stale,
        verification_missing: verification_is_missing(verification_runs),
        verification_stale,
        verification_failing: verification_has_failures(verification_runs),
        unused_files: project.unused_files,
    };
    let eligibility = determine_action_eligibility(&signals, repo_risk);
    let scores = score_review_actions(&signals, repo_risk, &eligibility);
    let best_review_action = scores
        .first()
        .map(|score| score.action)
        .unwrap_or("inspect_hot_files");
    let selected_review_score = scores.first();
    let runner_up_review_score = scores.get(1);
    let shared_review_reason = selected_review_score
        .map(|score| build_reason(score, runner_up_review_score, &signals, repo_risk))
        .unwrap_or_else(|| {
            "Current evidence does not yet support a stronger review recommendation.".to_string()
        });
    let shared_review_confidence = selected_review_score
        .map(|score| derive_confidence(score, &signals, repo_risk))
        .unwrap_or("medium");
    let primary_verification_command = project_commands
        .first()
        .cloned()
        .unwrap_or_else(|| "git status".to_string());

    let forced_context = ForcedRecommendationContext {
        project_id: project.id.clone(),
        primary_verification_command: primary_verification_command.clone(),
        verification_missing: signals.verification_missing,
        verification_gate_levels: gate_levels.clone(),
        cleanup_blockers: cleanup_blockers.clone(),
        refactor_blockers: refactor_blockers.clone(),
        repo_truth_gaps: repo_truth_gaps_json.clone(),
        mandatory_shell_checks: mandatory_shell_checks_json.clone(),
    };
    let evidence_context = EvidenceCollectionContext {
        project_id: project.id.clone(),
        project_command: project_commands[0].clone(),
        snapshot_missing: project.total_files == 0,
        activity_missing: project.accessed_files == 0,
        verification_gate_levels: gate_levels.clone(),
        repo_truth_gaps: repo_truth_gaps_json.clone(),
        mandatory_shell_checks: mandatory_shell_checks_json.clone(),
    };
    let review_context = ProjectReviewContext {
        project_id: project.id.clone(),
        project_command: project_commands[0].clone(),
        reason: shared_review_reason,
        confidence: shared_review_confidence.to_string(),
        safe_for_cleanup,
        safe_for_refactor,
        verification_gate_levels: gate_levels,
        cleanup_blockers,
        refactor_blockers,
        repo_truth_gaps: repo_truth_gaps_json,
        mandatory_shell_checks: mandatory_shell_checks_json,
    };

    let rec = if let Some(forced_recommendation) = eligibility.forced_action.and_then(|action| {
        ForcedProjectRecommendation::from_immediate_action(action, forced_context.clone())
    }) {
        forced_recommendation.into_recommendation()
    } else if project.status != "monitoring" {
        EvidenceCollectionRecommendation::new(
            EvidenceCollectionAction::StartMonitor,
            evidence_context.clone(),
        )
        .into_recommendation()
    } else if project.total_files == 0 || snapshot_stale {
        EvidenceCollectionRecommendation::new(
            EvidenceCollectionAction::TakeSnapshot,
            evidence_context.clone(),
        )
        .into_recommendation()
    } else if project.accessed_files == 0 || activity_stale {
        EvidenceCollectionRecommendation::new(
            EvidenceCollectionAction::GenerateActivityThenStats,
            evidence_context.clone(),
        )
        .into_recommendation()
    } else if let Some(forced_recommendation) = eligibility
        .forced_action
        .and_then(|action| ForcedProjectRecommendation::from_action(action, forced_context.clone()))
    {
        forced_recommendation.into_recommendation()
    } else if best_review_action == "review_unused_files" {
        ProjectReviewRecommendation::new(
            ProjectReviewAction::ReviewUnusedFiles,
            review_context.clone(),
        )
        .into_recommendation()
    } else {
        ProjectReviewRecommendation::new(ProjectReviewAction::InspectHotFiles, review_context)
            .into_recommendation()
    };

    let mut payload = to_value_or_error("Recommendation", rec);
    let selected_action = payload["recommended_next_action"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    payload["execution_sequence"] = execution_sequence_for_recommendation(
        &selected_action,
        repo_risk,
        verification_runs,
        &project_toolchain,
    );
    payload["review_focus"] = review_focus_for_action(&selected_action, repo_risk);
    payload
}

#[cfg(test)]
mod tests;
