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
use super::guidance_types::{ProjectOverview, Recommendation};
use super::serialization::to_value_or_error;
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

pub(crate) fn review_focus_for_action(selected_action: &str, repo_risk: &Value) -> Value {
    match selected_action {
        "inspect_hot_files" => {
            let mut risk_hints = Vec::new();
            if repo_risk["risk_level"].as_str().unwrap_or("low") != "low"
                || repo_risk["large_diff"].as_bool().unwrap_or(false)
            {
                risk_hints.push("repo_risk_elevated");
            }
            json!({
                "candidate_family": "hot_file",
                "candidate_basis": ["highest_access_activity", "activity_present"],
                "candidate_risk_hints": risk_hints,
            })
        }
        "review_unused_files" => json!({
            "candidate_family": "unused_candidate",
            "candidate_basis": ["zero_recorded_access", "snapshot_present"],
            "candidate_risk_hints": [],
        }),
        _ => Value::Null,
    }
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

    let blockers = || Some(Value::Array(cleanup_blockers.clone()));
    let refactor_b = || Some(Value::Array(refactor_blockers.clone()));

    let rec = if eligibility.forced_action == Some("review_failing_verification") {
        Recommendation {
            project_id: project.id.clone(),
            recommended_next_action: "review_failing_verification".to_string(),
            recommended_flow: vec![
                "Inspect the latest failing or uncertain verification evidence first.".to_string(),
                "Use shell diff and project-native verification commands to stabilize the project.".to_string(),
                "Return to cleanup or refactor review only after verification is passing again.".to_string(),
            ],
            reason: "Recent verification evidence includes failing or uncertain runs, so review and stabilize those results before broader cleanup or refactoring.".to_string(),
            confidence: "high".to_string(),
            strategy_mode: "verify_before_modify".to_string(),
            strategy_profile: strategy_profile(
                "verify_before_modify",
                "shell",
                "opendog",
                &["verification", "repository_risk", "activity_signals"],
            ),
            verification_gate_levels: gate_levels,
            cleanup_blockers: blockers(),
            refactor_blockers: refactor_b(),
            repo_truth_gaps: repo_truth_gaps_json.clone(),
            mandatory_shell_checks: mandatory_shell_checks_json.clone(),
            suggested_commands: vec![
                "opendog verification --id <project>".to_string(),
                primary_verification_command,
                "git diff".to_string(),
            ],
        }
    } else if eligibility.forced_action == Some("stabilize_repository_state") {
        Recommendation {
            project_id: project.id.clone(),
            recommended_next_action: "stabilize_repository_state".to_string(),
            recommended_flow: vec![
                "Stabilize the repository before broader code changes.".to_string(),
                "Use git status and diff to understand the in-progress operation.".to_string(),
                "Only return to OPENDOG-guided cleanup or review after the repository state is stable.".to_string(),
            ],
            reason: "The repository is mid-operation (merge/rebase/cherry-pick/bisect), so avoid broad modifications until that state is resolved.".to_string(),
            confidence: "high".to_string(),
            strategy_mode: "stabilize_before_modify".to_string(),
            strategy_profile: strategy_profile(
                "stabilize_before_modify",
                "shell",
                "opendog",
                &["repository_risk", "verification", "activity_signals"],
            ),
            verification_gate_levels: gate_levels,
            cleanup_blockers: blockers(),
            refactor_blockers: refactor_b(),
            repo_truth_gaps: repo_truth_gaps_json.clone(),
            mandatory_shell_checks: mandatory_shell_checks_json.clone(),
            suggested_commands: vec![
                "git status".to_string(),
                "git diff".to_string(),
                "opendog verification --id <project>".to_string(),
            ],
        }
    } else if project.status != "monitoring" {
        Recommendation {
            project_id: project.id.clone(),
            recommended_next_action: "start_monitor".to_string(),
            recommended_flow: vec![
                "Start monitoring because fresh activity evidence does not exist yet.".to_string(),
                "Let real workflow activity happen after monitoring is active.".to_string(),
                "Inspect stats only after OPENDOG has observed meaningful activity.".to_string(),
            ],
            reason: "This project is not currently being monitored, so opendog cannot collect fresh activity data yet.".to_string(),
            confidence: "medium".to_string(),
            strategy_mode: "collect_evidence_first".to_string(),
            strategy_profile: strategy_profile(
                "collect_evidence_first",
                "opendog",
                "shell",
                &["activity_signals", "repository_risk"],
            ),
            verification_gate_levels: gate_levels,
            cleanup_blockers: None,
            refactor_blockers: None,
            repo_truth_gaps: repo_truth_gaps_json.clone(),
            mandatory_shell_checks: mandatory_shell_checks_json.clone(),
            suggested_commands: vec![
                format!("opendog start --id {}", project.id),
                format!("opendog stats --id {}", project.id),
            ],
        }
    } else if project.total_files == 0 || snapshot_stale {
        Recommendation {
            project_id: project.id.clone(),
            recommended_next_action: "take_snapshot".to_string(),
            recommended_flow: vec![
                "Take a snapshot to establish the project baseline.".to_string(),
                "Use stats only after the baseline inventory exists.".to_string(),
                "If monitoring is already active, keep it running so activity can accumulate after snapshot.".to_string(),
            ],
            reason: if project.total_files == 0 {
                "Monitoring is active but no snapshot data exists yet, so file inventory and stats are incomplete.".to_string()
            } else {
                "Snapshot evidence exists but is stale, so refresh the baseline before trusting cleanup or hotspot conclusions.".to_string()
            },
            confidence: "medium".to_string(),
            strategy_mode: "collect_evidence_first".to_string(),
            strategy_profile: strategy_profile(
                "collect_evidence_first",
                "opendog",
                "shell",
                &["activity_signals", "repository_risk"],
            ),
            verification_gate_levels: gate_levels,
            cleanup_blockers: None,
            refactor_blockers: None,
            repo_truth_gaps: repo_truth_gaps_json.clone(),
            mandatory_shell_checks: mandatory_shell_checks_json.clone(),
            suggested_commands: vec![
                format!("opendog snapshot --id {}", project.id),
                format!("opendog stats --id {}", project.id),
            ],
        }
    } else if project.accessed_files == 0 || activity_stale {
        Recommendation {
            project_id: project.id.clone(),
            recommended_next_action: "generate_activity_then_stats".to_string(),
            recommended_flow: vec![
                "Generate real project activity with edits, tests, or builds.".to_string(),
                "Avoid drawing hotspot or cleanup conclusions before activity exists.".to_string(),
                "Inspect stats after the observation window is meaningful.".to_string(),
            ],
            reason: if project.accessed_files == 0 {
                "Snapshot data exists, but no file access activity has been recorded yet."
                    .to_string()
            } else {
                "Activity evidence exists but is stale, so generate fresh workflow activity before trusting current hotspot or cleanup signals.".to_string()
            },
            confidence: "medium".to_string(),
            strategy_mode: "collect_evidence_first".to_string(),
            strategy_profile: strategy_profile(
                "collect_evidence_first",
                "shell",
                "opendog",
                &["activity_signals", "verification", "repository_risk"],
            ),
            verification_gate_levels: gate_levels,
            cleanup_blockers: None,
            refactor_blockers: None,
            repo_truth_gaps: repo_truth_gaps_json.clone(),
            mandatory_shell_checks: mandatory_shell_checks_json.clone(),
            suggested_commands: vec![
                project_commands[0].clone(),
                format!("opendog stats --id {}", project.id),
            ],
        }
    } else if eligibility.forced_action == Some("run_verification_before_high_risk_changes") {
        Recommendation {
            project_id: project.id.clone(),
            recommended_next_action: "run_verification_before_high_risk_changes".to_string(),
            recommended_flow: vec![
                "Run and record project-native verification before risky changes.".to_string(),
                "Use OPENDOG to persist the resulting evidence for later decisions.".to_string(),
                "Return to cleanup or refactor review only after verification evidence exists."
                    .to_string(),
            ],
            reason: if verification_is_missing(verification_runs) {
                "Activity evidence exists, but no recorded test/lint/build results are available yet. Verify first before risky cleanup or refactor work.".to_string()
            } else {
                "Recorded verification evidence exists but is stale, so refresh test/lint/build results before risky cleanup or refactor work.".to_string()
            },
            confidence: "medium".to_string(),
            strategy_mode: "verify_before_modify".to_string(),
            strategy_profile: strategy_profile(
                "verify_before_modify",
                "shell",
                "opendog",
                &["verification", "activity_signals", "repository_risk"],
            ),
            verification_gate_levels: gate_levels,
            cleanup_blockers: blockers(),
            refactor_blockers: refactor_b(),
            repo_truth_gaps: repo_truth_gaps_json.clone(),
            mandatory_shell_checks: mandatory_shell_checks_json.clone(),
            suggested_commands: vec![
                primary_verification_command,
                "opendog run-verification --id <project> --kind test --command '<cmd>'".to_string(),
                format!("opendog stats --id {}", project.id),
            ],
        }
    } else if best_review_action == "review_unused_files" {
        let strategy_mode = if safe_for_cleanup {
            "review_then_modify"
        } else {
            "verify_before_modify"
        };
        Recommendation {
            project_id: project.id.clone(),
            recommended_next_action: "review_unused_files".to_string(),
            recommended_flow: vec![
                "Inspect unused-file candidates before proposing cleanup.".to_string(),
                "Validate each candidate with shell search, imports, and tests.".to_string(),
                "Only delete or refactor after cleanup blockers are cleared.".to_string(),
            ],
            reason: shared_review_reason.clone(),
            confidence: shared_review_confidence.to_string(),
            strategy_mode: strategy_mode.to_string(),
            strategy_profile: strategy_profile(
                strategy_mode,
                "opendog",
                "shell",
                &["activity_signals", "verification", "repository_risk"],
            ),
            verification_gate_levels: gate_levels,
            cleanup_blockers: blockers(),
            refactor_blockers: refactor_b(),
            repo_truth_gaps: repo_truth_gaps_json.clone(),
            mandatory_shell_checks: mandatory_shell_checks_json.clone(),
            suggested_commands: vec![
                format!("opendog unused --id {}", project.id),
                "rg \"<pattern>\" .".to_string(),
                project_commands[0].clone(),
            ],
        }
    } else {
        let strategy_mode = if safe_for_refactor {
            "inspect_then_modify"
        } else {
            "verify_before_modify"
        };
        Recommendation {
            project_id: project.id.clone(),
            recommended_next_action: "inspect_hot_files".to_string(),
            recommended_flow: vec![
                "Inspect the hottest observed files first.".to_string(),
                "Use shell diff and symbol search after OPENDOG narrows the review target."
                    .to_string(),
                "Treat hotspot review as a precursor to targeted refactor, not broad cleanup."
                    .to_string(),
            ],
            reason: shared_review_reason,
            confidence: shared_review_confidence.to_string(),
            strategy_mode: strategy_mode.to_string(),
            strategy_profile: strategy_profile(
                strategy_mode,
                if safe_for_refactor {
                    "opendog"
                } else {
                    "shell"
                },
                if safe_for_refactor {
                    "shell"
                } else {
                    "opendog"
                },
                &["activity_signals", "verification", "repository_risk"],
            ),
            verification_gate_levels: gate_levels,
            cleanup_blockers: blockers(),
            refactor_blockers: refactor_b(),
            repo_truth_gaps: repo_truth_gaps_json,
            mandatory_shell_checks: mandatory_shell_checks_json,
            suggested_commands: vec![
                format!("opendog stats --id {}", project.id),
                "git diff".to_string(),
                "rg \"<pattern>\" .".to_string(),
            ],
        }
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
mod tests {
    use super::*;
    use crate::storage::queries::VerificationRun;
    use serde_json::json;
    use std::path::PathBuf;

    fn make_state(id: &str, status: &str, root: &str) -> ProjectGuidanceState {
        // Use a recent unix timestamp so the snapshot/activity are not considered stale.
        // snapshot_is_stale checks for "stale" or "unknown" status; fresh timestamps avoid both.
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let recent_ts = (now_secs - 3600).to_string(); // 1 hour ago
        ProjectGuidanceState {
            id: id.to_string(),
            status: status.to_string(),
            root_path: PathBuf::from(root),
            total_files: 10,
            accessed_files: 5,
            unused_files: 2,
            latest_snapshot_captured_at: Some(recent_ts.clone()),
            latest_activity_at: Some(recent_ts.clone()),
            latest_verification_at: Some(recent_ts),
        }
    }

    fn clean_repo_risk() -> Value {
        json!({
            "risk_level": "low",
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": false,
            "changed_file_count": 0,
        })
    }

    fn make_verification_run(status: &str, kind: &str) -> VerificationRun {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let recent_ts = (now_secs - 3600).to_string();
        VerificationRun {
            id: 1,
            kind: kind.to_string(),
            status: status.to_string(),
            command: "cargo test".to_string(),
            exit_code: Some(0),
            summary: None,
            source: "cli".to_string(),
            started_at: None,
            finished_at: recent_ts,
        }
    }

    // --- review_focus_for_action ---

    #[test]
    fn review_focus_for_action_inspect_hot_files_low_risk() {
        let repo_risk = clean_repo_risk();
        let result = review_focus_for_action("inspect_hot_files", &repo_risk);
        assert_eq!(result["candidate_family"], "hot_file");
        assert_eq!(
            result["candidate_basis"],
            json!(["highest_access_activity", "activity_present"])
        );
        assert!(result["candidate_risk_hints"]
            .as_array()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn review_focus_for_action_inspect_hot_files_elevated_risk() {
        let repo_risk = json!({
            "risk_level": "high",
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": false,
            "changed_file_count": 0,
        });
        let result = review_focus_for_action("inspect_hot_files", &repo_risk);
        assert_eq!(result["candidate_family"], "hot_file");
        let hints = result["candidate_risk_hints"].as_array().unwrap();
        assert_eq!(hints, &vec![json!("repo_risk_elevated")]);
    }

    #[test]
    fn review_focus_for_action_inspect_hot_files_large_diff() {
        // low risk but large diff
        let repo_risk = json!({
            "risk_level": "low",
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": true,
            "changed_file_count": 50,
        });
        let result = review_focus_for_action("inspect_hot_files", &repo_risk);
        let hints = result["candidate_risk_hints"].as_array().unwrap();
        assert_eq!(hints, &vec![json!("repo_risk_elevated")]);
    }

    #[test]
    fn review_focus_for_action_review_unused_files() {
        let repo_risk = clean_repo_risk();
        let result = review_focus_for_action("review_unused_files", &repo_risk);
        assert_eq!(result["candidate_family"], "unused_candidate");
        assert_eq!(
            result["candidate_basis"],
            json!(["zero_recorded_access", "snapshot_present"])
        );
        assert!(result["candidate_risk_hints"]
            .as_array()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn review_focus_for_action_unknown_returns_null() {
        let repo_risk = clean_repo_risk();
        let result = review_focus_for_action("take_snapshot", &repo_risk);
        assert!(result.is_null());
    }

    // --- recommend_project_action: start_monitor path ---

    #[test]
    fn recommend_project_action_start_monitor_when_not_monitoring() {
        let dir = tempfile::tempdir().unwrap();
        let state = make_state("proj-1", "stopped", dir.path().to_str().unwrap());
        let repo_risk = clean_repo_risk();
        let result = recommend_project_action(&state, &repo_risk, &[]);
        assert_eq!(result["recommended_next_action"], "start_monitor");
        assert_eq!(
            result["strategy_mode"],
            "collect_evidence_first"
        );
        assert!(result["suggested_commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c.as_str().unwrap().contains("opendog start")));
    }

    // --- recommend_project_action: take_snapshot path ---

    #[test]
    fn recommend_project_action_take_snapshot_when_no_files() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = make_state("proj-2", "monitoring", dir.path().to_str().unwrap());
        state.total_files = 0;
        let repo_risk = clean_repo_risk();
        let result = recommend_project_action(&state, &repo_risk, &[]);
        assert_eq!(result["recommended_next_action"], "take_snapshot");
        assert!(result["reason"]
            .as_str()
            .unwrap()
            .contains("no snapshot data exists"));
    }

    // --- recommend_project_action: generate_activity_then_stats path ---

    #[test]
    fn recommend_project_action_generate_activity_when_no_accessed_files() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = make_state("proj-3", "monitoring", dir.path().to_str().unwrap());
        state.accessed_files = 0;
        let repo_risk = clean_repo_risk();
        let result = recommend_project_action(&state, &repo_risk, &[]);
        assert_eq!(
            result["recommended_next_action"],
            "generate_activity_then_stats"
        );
        assert!(result["reason"]
            .as_str()
            .unwrap()
            .contains("no file access activity"));
    }

    // --- recommend_project_action: review_unused_files or inspect_hot_files ---

    #[test]
    fn recommend_project_action_returns_valid_json_with_active_project() {
        let dir = tempfile::tempdir().unwrap();
        // Create a Cargo.toml so detect_project_commands finds something
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"test\"").unwrap();
        let state = make_state("proj-4", "monitoring", dir.path().to_str().unwrap());
        let repo_risk = clean_repo_risk();
        let runs = vec![make_verification_run("passed", "test")];
        let result = recommend_project_action(&state, &repo_risk, &runs);
        // Should be one of the review/inspect actions
        let action = result["recommended_next_action"].as_str().unwrap();
        assert!(
            ["inspect_hot_files", "review_unused_files"].contains(&action),
            "unexpected action: {}",
            action
        );
        assert!(result["recommended_flow"].is_array());
        assert!(result["reason"].is_string());
        assert!(result["confidence"].is_string());
        assert!(result["strategy_mode"].is_string());
        assert!(result["suggested_commands"].is_array());
        // execution_sequence is only non-null for certain action types;
        // inspect_hot_files and review_unused_files return Null from the
        // execution_sequence dispatch, so just verify the field exists.
        assert!(result.get("execution_sequence").is_some());
    }

    #[test]
    fn recommend_project_action_includes_review_focus() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"test\"").unwrap();
        let state = make_state("proj-5", "monitoring", dir.path().to_str().unwrap());
        let repo_risk = clean_repo_risk();
        let runs = vec![make_verification_run("passed", "test")];
        let result = recommend_project_action(&state, &repo_risk, &runs);
        let action = result["recommended_next_action"].as_str().unwrap();
        if action == "inspect_hot_files" || action == "review_unused_files" {
            assert!(result["review_focus"].is_object());
        }
    }

    // --- recommend_project_action: forced verification failure ---

    #[test]
    fn recommend_project_action_forces_failing_verification_review() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"test\"").unwrap();
        let mut state = make_state("proj-6", "monitoring", dir.path().to_str().unwrap());
        // Make the project look healthy
        state.total_files = 100;
        state.accessed_files = 50;
        // But verification is failing
        let mut repo_risk = clean_repo_risk();
        repo_risk["risk_level"] = json!("critical");
        repo_risk["operation_states"] = json!(["merge"]);
        let runs = vec![VerificationRun {
            id: 1,
            kind: "test".to_string(),
            status: "failed".to_string(),
            command: "cargo test".to_string(),
            exit_code: Some(1),
            summary: None,
            source: "cli".to_string(),
            started_at: None,
            finished_at: {
                let now_secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                (now_secs - 3600).to_string()
            },
        }];
        let result = recommend_project_action(&state, &repo_risk, &runs);
        // Should force review_failing_verification
        assert_eq!(
            result["recommended_next_action"],
            "review_failing_verification"
        );
    }

    // --- recommend_project_action: stabilize_repository_state ---

    #[test]
    fn recommend_project_action_forces_stabilize_repository() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"test\"").unwrap();
        let state = make_state("proj-7", "monitoring", dir.path().to_str().unwrap());
        let mut repo_risk = clean_repo_risk();
        repo_risk["risk_level"] = json!("critical");
        repo_risk["operation_states"] = json!(["rebase"]);
        // Must provide passing verification runs so that eligibility does not
        // force run_verification_before_high_risk_changes instead.
        let runs = vec![make_verification_run("passed", "test")];
        let result = recommend_project_action(&state, &repo_risk, &runs);
        assert_eq!(
            result["recommended_next_action"],
            "stabilize_repository_state"
        );
    }

    // --- project_overview ---

    #[test]
    fn project_overview_assembles_correctly() {
        let dir = tempfile::tempdir().unwrap();
        let state = make_state("ov-1", "monitoring", dir.path().to_str().unwrap());
        let repo_risk = clean_repo_risk();
        let recommendation = json!({
            "recommended_next_action": "inspect_hot_files",
            "recommended_flow": ["Inspect hot files."],
            "reason": "Active project.",
            "confidence": "medium",
        });
        let verification_layer = json!({
            "status": "available",
            "safe_for_cleanup": true,
            "safe_for_refactor": false,
            "cleanup_blockers": [],
            "refactor_blockers": ["No lint evidence."],
            "gate_assessment": {
                "cleanup": { "level": "allow" },
                "refactor": { "level": "blocked" },
            },
        });
        let mock_data_summary = json!({"mock_candidate_count": 0, "hardcoded_candidate_count": 0});
        let storage_maintenance = json!({
            "maintenance_candidate": false,
            "vacuum_candidate": false,
            "approx_reclaimable_bytes": 0,
            "approx_db_size_bytes": 2048,
        });

        let result = project_overview(
            &state,
            &repo_risk,
            &recommendation,
            &verification_layer,
            &mock_data_summary,
            &storage_maintenance,
        );

        assert_eq!(result["project_id"], "ov-1");
        assert_eq!(result["status"], "monitoring");
        assert_eq!(result["snapshot_available"], true);
        assert_eq!(result["activity_available"], true);
        assert_eq!(result["unused_files"], 2);
        assert!(result["repo_status_risk"].is_object());
        assert!(result["verification_evidence"].is_object());
        assert!(result["mock_data_summary"].is_object());
        assert!(result["storage_maintenance"].is_object());
        assert!(result["project_toolchain"].is_object());
        assert_eq!(result["recommended_next_action"], "inspect_hot_files");
        // Attention enrichment should add attention fields
        assert!(result["attention_score"].is_number());
    }
}
