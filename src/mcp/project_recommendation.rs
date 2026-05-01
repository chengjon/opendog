use serde_json::{json, Value};

pub(crate) mod eligibility;
pub(crate) mod reasoning;
pub(crate) mod scoring;
pub(crate) mod sequencing;

use crate::config::ProjectInfo;
use crate::core::retention;
use crate::storage::database::Database;
use crate::storage::queries::VerificationRun;

use self::eligibility::{determine_action_eligibility, GateLevel, RecommendationSignals};
use self::reasoning::{build_reason, derive_confidence};
use self::scoring::score_review_actions;
use self::sequencing::execution_sequence_for_recommendation;
use super::constraints::repo_truth_gap_projection;
use super::{
    activity_is_stale, detect_mock_data_report, detect_project_commands,
    enrich_project_overview_with_attention, latest_activity_timestamp,
    latest_verification_timestamp, now_unix_secs, project_observation_layer,
    project_readiness_snapshot, project_storage_maintenance, project_toolchain_layer,
    repo_status_risk_layer, snapshot_is_stale, strategy_profile, verification_has_failures,
    verification_is_missing, verification_is_stale, verification_status_layer, ProjectGuidanceData,
    ProjectGuidanceState,
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

    enrich_project_overview_with_attention(&json!({
        "project_id": project.id,
        "status": project.status,
        "snapshot_available": project.total_files > 0,
        "activity_available": project.accessed_files > 0,
        "unused_files": project.unused_files,
        "observation": observation,
        "repo_status_risk": repo_risk,
        "verification_evidence": verification_layer,
        "mock_data_summary": mock_data_summary,
        "storage_maintenance": storage_maintenance,
        "project_toolchain": project_toolchain_layer(&project.root_path),
        "verification_safe_for_cleanup": readiness["verification_safe_for_cleanup"].clone(),
        "verification_safe_for_refactor": readiness["verification_safe_for_refactor"].clone(),
        "verification_gate_levels": {
            "cleanup": readiness["cleanup_gate_level"].clone(),
            "refactor": readiness["refactor_gate_level"].clone(),
        },
        "safe_for_cleanup": readiness["safe_for_cleanup"].clone(),
        "safe_for_cleanup_reason": readiness["safe_for_cleanup_reason"].clone(),
        "cleanup_blockers": readiness["cleanup_blockers"].clone(),
        "safe_for_refactor": readiness["safe_for_refactor"].clone(),
        "safe_for_refactor_reason": readiness["safe_for_refactor_reason"].clone(),
        "refactor_blockers": readiness["refactor_blockers"].clone(),
        "recommended_next_action": recommendation["recommended_next_action"].clone(),
        "recommended_flow": recommendation["recommended_flow"].clone(),
        "recommended_reason": recommendation["reason"].clone(),
        "strategy_confidence": recommendation["confidence"].clone(),
    }))
}

pub(crate) fn collect_project_guidance_context<F>(
    projects: &[ProjectInfo],
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
        let storage_metrics = if project.db_path.exists() {
            Database::open_project(&project.db_path)
                .ok()
                .and_then(|db| retention::collect_storage_metrics(&db).ok())
        } else {
            None
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
        let storage_maintenance = project_storage_maintenance(storage_metrics.as_ref());
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
    let execution_sequence =
        execution_sequence_for_recommendation(eligibility.forced_action, repo_risk);
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

    if eligibility.forced_action == Some("review_failing_verification") {
        json!({
            "project_id": project.id,
            "recommended_next_action": "review_failing_verification",
            "recommended_flow": [
                "Inspect the latest failing or uncertain verification evidence first.",
                "Use shell diff and project-native verification commands to stabilize the project.",
                "Return to cleanup or refactor review only after verification is passing again."
            ],
            "reason": "Recent verification evidence includes failing or uncertain runs, so review and stabilize those results before broader cleanup or refactoring.",
            "confidence": "high",
            "strategy_mode": "verify_before_modify",
            "strategy_profile": strategy_profile(
                "verify_before_modify",
                "shell",
                "opendog",
                &["verification", "repository_risk", "activity_signals"],
            ),
            "verification_gate_levels": gate_levels,
            "cleanup_blockers": cleanup_blockers,
            "refactor_blockers": refactor_blockers,
            "repo_truth_gaps": repo_truth_gaps_json.clone(),
            "mandatory_shell_checks": mandatory_shell_checks_json.clone(),
            "execution_sequence": execution_sequence.clone(),
            "suggested_commands": [
                "opendog verification --id <project>".to_string(),
                primary_verification_command,
                "git diff".to_string()
            ]
        })
    } else if eligibility.forced_action == Some("stabilize_repository_state") {
        json!({
            "project_id": project.id,
            "recommended_next_action": "stabilize_repository_state",
            "recommended_flow": [
                "Stabilize the repository before broader code changes.",
                "Use git status and diff to understand the in-progress operation.",
                "Only return to OPENDOG-guided cleanup or review after the repository state is stable."
            ],
            "reason": "The repository is mid-operation (merge/rebase/cherry-pick/bisect), so avoid broad modifications until that state is resolved.",
            "confidence": "high",
            "strategy_mode": "stabilize_before_modify",
            "strategy_profile": strategy_profile(
                "stabilize_before_modify",
                "shell",
                "opendog",
                &["repository_risk", "verification", "activity_signals"],
            ),
            "verification_gate_levels": gate_levels,
            "cleanup_blockers": cleanup_blockers,
            "refactor_blockers": refactor_blockers,
            "repo_truth_gaps": repo_truth_gaps_json.clone(),
            "mandatory_shell_checks": mandatory_shell_checks_json.clone(),
            "execution_sequence": execution_sequence.clone(),
            "suggested_commands": [
                "git status".to_string(),
                "git diff".to_string(),
                "opendog verification --id <project>".to_string()
            ]
        })
    } else if project.status != "monitoring" {
        json!({
            "project_id": project.id,
            "recommended_next_action": "start_monitor",
            "recommended_flow": [
                "Start monitoring because fresh activity evidence does not exist yet.",
                "Let real workflow activity happen after monitoring is active.",
                "Inspect stats only after OPENDOG has observed meaningful activity."
            ],
            "reason": "This project is not currently being monitored, so opendog cannot collect fresh activity data yet.",
            "confidence": "medium",
            "strategy_mode": "collect_evidence_first",
            "strategy_profile": strategy_profile(
                "collect_evidence_first",
                "opendog",
                "shell",
                &["activity_signals", "repository_risk"],
            ),
            "verification_gate_levels": gate_levels,
            "repo_truth_gaps": repo_truth_gaps_json.clone(),
            "mandatory_shell_checks": mandatory_shell_checks_json.clone(),
            "execution_sequence": execution_sequence.clone(),
            "suggested_commands": [
                format!("opendog start --id {}", project.id),
                format!("opendog stats --id {}", project.id)
            ]
        })
    } else if project.total_files == 0 || snapshot_stale {
        json!({
            "project_id": project.id,
            "recommended_next_action": "take_snapshot",
            "recommended_flow": [
                "Take a snapshot to establish the project baseline.",
                "Use stats only after the baseline inventory exists.",
                "If monitoring is already active, keep it running so activity can accumulate after snapshot."
            ],
            "reason": if project.total_files == 0 {
                "Monitoring is active but no snapshot data exists yet, so file inventory and stats are incomplete.".to_string()
            } else {
                "Snapshot evidence exists but is stale, so refresh the baseline before trusting cleanup or hotspot conclusions.".to_string()
            },
            "confidence": "medium",
            "strategy_mode": "collect_evidence_first",
            "strategy_profile": strategy_profile(
                "collect_evidence_first",
                "opendog",
                "shell",
                &["activity_signals", "repository_risk"],
            ),
            "verification_gate_levels": gate_levels,
            "repo_truth_gaps": repo_truth_gaps_json.clone(),
            "mandatory_shell_checks": mandatory_shell_checks_json.clone(),
            "execution_sequence": execution_sequence.clone(),
            "suggested_commands": [
                format!("opendog snapshot --id {}", project.id),
                format!("opendog stats --id {}", project.id)
            ]
        })
    } else if project.accessed_files == 0 || activity_stale {
        json!({
            "project_id": project.id,
            "recommended_next_action": "generate_activity_then_stats",
            "recommended_flow": [
                "Generate real project activity with edits, tests, or builds.",
                "Avoid drawing hotspot or cleanup conclusions before activity exists.",
                "Inspect stats after the observation window is meaningful."
            ],
            "reason": if project.accessed_files == 0 {
                "Snapshot data exists, but no file access activity has been recorded yet.".to_string()
            } else {
                "Activity evidence exists but is stale, so generate fresh workflow activity before trusting current hotspot or cleanup signals.".to_string()
            },
            "confidence": "medium",
            "strategy_mode": "collect_evidence_first",
            "strategy_profile": strategy_profile(
                "collect_evidence_first",
                "shell",
                "opendog",
                &["activity_signals", "verification", "repository_risk"],
            ),
            "verification_gate_levels": gate_levels,
            "repo_truth_gaps": repo_truth_gaps_json.clone(),
            "mandatory_shell_checks": mandatory_shell_checks_json.clone(),
            "execution_sequence": execution_sequence.clone(),
            "suggested_commands": [
                project_commands[0].clone(),
                format!("opendog stats --id {}", project.id)
            ]
        })
    } else if eligibility.forced_action == Some("run_verification_before_high_risk_changes") {
        json!({
            "project_id": project.id,
            "recommended_next_action": "run_verification_before_high_risk_changes",
            "recommended_flow": [
                "Run and record project-native verification before risky changes.",
                "Use OPENDOG to persist the resulting evidence for later decisions.",
                "Return to cleanup or refactor review only after verification evidence exists."
            ],
            "reason": if verification_is_missing(verification_runs) {
                "Activity evidence exists, but no recorded test/lint/build results are available yet. Verify first before risky cleanup or refactor work.".to_string()
            } else {
                "Recorded verification evidence exists but is stale, so refresh test/lint/build results before risky cleanup or refactor work.".to_string()
            },
            "confidence": "medium",
            "strategy_mode": "verify_before_modify",
            "strategy_profile": strategy_profile(
                "verify_before_modify",
                "shell",
                "opendog",
                &["verification", "activity_signals", "repository_risk"],
            ),
            "verification_gate_levels": gate_levels,
            "cleanup_blockers": cleanup_blockers,
            "refactor_blockers": refactor_blockers,
            "repo_truth_gaps": repo_truth_gaps_json.clone(),
            "mandatory_shell_checks": mandatory_shell_checks_json.clone(),
            "execution_sequence": execution_sequence.clone(),
            "suggested_commands": [
                primary_verification_command,
                "opendog run-verification --id <project> --kind test --command '<cmd>'".to_string(),
                format!("opendog stats --id {}", project.id)
            ]
        })
    } else if best_review_action == "review_unused_files" {
        json!({
            "project_id": project.id,
            "recommended_next_action": "review_unused_files",
            "recommended_flow": [
                "Inspect unused-file candidates before proposing cleanup.",
                "Validate each candidate with shell search, imports, and tests.",
                "Only delete or refactor after cleanup blockers are cleared."
            ],
            "reason": shared_review_reason.clone(),
            "confidence": shared_review_confidence,
            "strategy_mode": if safe_for_cleanup {
                "review_then_modify"
            } else {
                "verify_before_modify"
            },
            "strategy_profile": strategy_profile(
                if safe_for_cleanup {
                    "review_then_modify"
                } else {
                    "verify_before_modify"
                },
                "opendog",
                "shell",
                &["activity_signals", "verification", "repository_risk"],
            ),
            "verification_gate_levels": gate_levels,
            "cleanup_blockers": cleanup_blockers,
            "refactor_blockers": refactor_blockers,
            "repo_truth_gaps": repo_truth_gaps_json.clone(),
            "mandatory_shell_checks": mandatory_shell_checks_json.clone(),
            "execution_sequence": execution_sequence.clone(),
            "suggested_commands": [
                format!("opendog unused --id {}", project.id),
                "rg \"<pattern>\" .".to_string(),
                project_commands[0].clone()
            ]
        })
    } else {
        json!({
            "project_id": project.id,
            "recommended_next_action": "inspect_hot_files",
            "recommended_flow": [
                "Inspect the hottest observed files first.",
                "Use shell diff and symbol search after OPENDOG narrows the review target.",
                "Treat hotspot review as a precursor to targeted refactor, not broad cleanup."
            ],
            "reason": shared_review_reason,
            "confidence": shared_review_confidence,
            "strategy_mode": if safe_for_refactor {
                "inspect_then_modify"
            } else {
                "verify_before_modify"
            },
            "strategy_profile": strategy_profile(
                if safe_for_refactor {
                    "inspect_then_modify"
                } else {
                    "verify_before_modify"
                },
                if safe_for_refactor { "opendog" } else { "shell" },
                if safe_for_refactor { "shell" } else { "opendog" },
                &["activity_signals", "verification", "repository_risk"],
            ),
            "verification_gate_levels": gate_levels,
            "cleanup_blockers": cleanup_blockers,
            "refactor_blockers": refactor_blockers,
            "repo_truth_gaps": repo_truth_gaps_json,
            "mandatory_shell_checks": mandatory_shell_checks_json,
            "execution_sequence": execution_sequence,
            "suggested_commands": [
                format!("opendog stats --id {}", project.id),
                "git diff".to_string(),
                "rg \"<pattern>\" .".to_string()
            ]
        })
    }
}
