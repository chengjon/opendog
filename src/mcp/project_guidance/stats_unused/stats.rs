use serde_json::{json, Value};
use std::path::Path;

use crate::core::stats;
use crate::storage::queries::{StatsEntry, VerificationRun};

use super::super::super::review_candidates::{
    build_review_candidate, CandidateFreshness, ReviewCandidateContext,
};
use super::super::super::{
    build_constraints_boundaries_layer, common_boundary_hints, detect_mock_data_report,
    detect_project_commands, project_readiness_snapshot, project_toolchain_layer,
    repo_status_risk_layer, tool_guidance, verification_status_layer,
};

pub(in crate::mcp) fn stats_guidance(
    root_path: &Path,
    summary: &stats::ProjectSummary,
    entries: &[StatsEntry],
    verification_runs: &[VerificationRun],
) -> Value {
    let project_commands = detect_project_commands(root_path);
    if summary.total_files == 0 {
        let mut guidance = tool_guidance(
            "No snapshot data exists yet. Take a snapshot or start monitoring before relying on stats.",
            &[
                "opendog snapshot --id <project>",
                "opendog start --id <project>",
                "rg --files .",
            ],
            &["take_snapshot", "start_monitor"],
            Some("Use shell file listing if you need to verify the registered root contains files."),
        );
        guidance["layers"]["workspace_observation"] = json!({
            "status": "available",
            "analysis_state": "not_ready",
            "snapshot_available": false,
            "activity_available": false,
            "total_files": summary.total_files,
            "accessed_files": summary.accessed_files,
            "unused_files": summary.unused_files,
        });
        guidance["layers"]["repo_status_risk"] = repo_status_risk_layer(root_path);
        guidance["layers"]["project_toolchain"] = project_toolchain_layer(root_path);
        guidance["layers"]["verification_evidence"] = verification_status_layer(verification_runs);
        guidance
    } else if summary.accessed_files == 0 {
        let mut guidance = tool_guidance(
            "Snapshot data exists but no file activity has been recorded yet. Run tests, edit files, or otherwise exercise the project, then query stats again.",
            &[&project_commands[0], "pytest", "opendog stats --id <project>"],
            &["start_monitor", "get_unused_files"],
            Some("Use shell commands to generate real project activity before interpreting file hotness."),
        );
        let boundary_hints = common_boundary_hints(root_path);
        guidance["layers"]["workspace_observation"] = json!({
            "status": "available",
            "analysis_state": "insufficient_activity",
            "snapshot_available": true,
            "activity_available": false,
            "total_files": summary.total_files,
            "accessed_files": summary.accessed_files,
            "unused_files": summary.unused_files,
        });
        guidance["layers"]["repo_status_risk"] = repo_status_risk_layer(root_path);
        guidance["layers"]["project_toolchain"] = project_toolchain_layer(root_path);
        guidance["layers"]["verification_evidence"] = verification_status_layer(verification_runs);
        let repo_risk = guidance["layers"]["repo_status_risk"].clone();
        let verification_layer = guidance["layers"]["verification_evidence"].clone();
        guidance["layers"]["constraints_boundaries"] = build_constraints_boundaries_layer(
            Some(&repo_risk),
            Some(&verification_layer),
            vec!["Snapshot exists but no activity-derived file usage has been recorded yet."
                .to_string()],
            Vec::new(),
            vec![
                "Without observed activity, OPENDOG cannot rank hot files or validate unused-file conclusions."
                    .to_string(),
            ],
            vec![project_commands[0].clone(), "git status".to_string()],
        );
        guidance["layers"]["constraints_boundaries"]["protected_paths"] =
            boundary_hints["protected_paths"].clone();
        guidance["layers"]["constraints_boundaries"]["generated_artifact_directories"] =
            boundary_hints["generated_artifact_directories"].clone();
        guidance
    } else if let Some(hottest) = entries.first() {
        let mut guidance = tool_guidance(
            &format!(
                "Stats loaded. Focus shell inspection on the hottest file first: {}.",
                hottest.file_path
            ),
            &[
                "opendog unused --id <project>",
                "rg \"<pattern>\" .",
                "git diff",
            ],
            &["get_unused_files", "list_projects"],
            Some("Use shell commands once opendog identifies candidate files; opendog itself does not inspect diffs or symbols."),
        );
        let repo_risk = repo_status_risk_layer(root_path);
        let boundary_hints = common_boundary_hints(root_path);
        let verification_layer = verification_status_layer(verification_runs);
        let readiness = project_readiness_snapshot(&repo_risk, &verification_layer);
        let mock_summary = detect_mock_data_report(root_path, entries).to_value(5);
        let context = ReviewCandidateContext {
            mock_summary: &mock_summary,
            freshness: CandidateFreshness::default(),
            repo_risk: &repo_risk,
        };
        let mut file_recommendations = Vec::new();
        file_recommendations.push(build_review_candidate(
            "hot_file",
            &hottest.file_path,
            "primary",
            "This file currently has the highest observed access activity.",
            vec![
                format!("rg \"{}\" .", hottest.file_path),
                "git diff".to_string(),
                project_commands[0].clone(),
            ],
            context,
        ));
        if let Some(unused_candidate) = entries.iter().find(|e| e.access_count == 0) {
            file_recommendations.push(build_review_candidate(
                "unused_candidate",
                &unused_candidate.file_path,
                "secondary",
                "This file appears in the snapshot but has no recorded accesses yet.",
                vec![
                    format!("rg \"{}\" .", unused_candidate.file_path),
                    "git grep <symbol>".to_string(),
                    project_commands[0].clone(),
                ],
                context,
            ));
        }
        guidance["file_recommendations"] = json!(file_recommendations.clone());
        guidance["layers"]["workspace_observation"] = json!({
            "status": "available",
            "analysis_state": "ready",
            "snapshot_available": true,
            "monitoring_required_for_freshness": true,
            "activity_available": summary.accessed_files > 0,
            "total_files": summary.total_files,
            "accessed_files": summary.accessed_files,
            "unused_files": summary.unused_files,
        });
        guidance["layers"]["repo_status_risk"] = repo_risk.clone();
        guidance["layers"]["cleanup_refactor_candidates"] = json!({
            "status": "available",
            "candidates": file_recommendations,
            "mock_data_candidates": mock_summary["mock_data_candidates"].clone(),
            "hardcoded_data_candidates": mock_summary["hardcoded_data_candidates"].clone(),
            "mixed_review_files": mock_summary["mixed_review_files"].clone(),
            "safe_for_cleanup": readiness["safe_for_cleanup"].clone(),
            "safe_for_cleanup_reason": readiness["safe_for_cleanup_reason"].clone(),
            "cleanup_blockers": readiness["cleanup_blockers"].clone(),
            "safe_for_refactor": readiness["safe_for_refactor"].clone(),
            "safe_for_refactor_reason": readiness["safe_for_refactor_reason"].clone(),
            "refactor_blockers": readiness["refactor_blockers"].clone(),
        });
        guidance["layers"]["project_toolchain"] = project_toolchain_layer(root_path);
        let mut evidence = verification_layer;
        evidence["direct_observations"] = json!([
            format!("Snapshot contains {} files.", summary.total_files),
            format!("Observed accessed files: {}.", summary.accessed_files),
            format!("Observed unused candidates: {}.", summary.unused_files),
        ]);
        evidence["inferences"] = json!([format!(
            "{} is currently the hottest observed file.",
            hottest.file_path
        )]);
        evidence["confidence"] = json!("medium");
        guidance["layers"]["verification_evidence"] = evidence;
        guidance["layers"]["execution_strategy"]["cleanup_ready_now"] =
            readiness["safe_for_cleanup"].clone();
        guidance["layers"]["execution_strategy"]["cleanup_ready_reason"] =
            readiness["safe_for_cleanup_reason"].clone();
        guidance["layers"]["execution_strategy"]["cleanup_gate_level"] =
            readiness["cleanup_gate_level"].clone();
        guidance["layers"]["execution_strategy"]["refactor_ready_now"] =
            readiness["safe_for_refactor"].clone();
        guidance["layers"]["execution_strategy"]["refactor_ready_reason"] =
            readiness["safe_for_refactor_reason"].clone();
        guidance["layers"]["execution_strategy"]["refactor_gate_level"] =
            readiness["refactor_gate_level"].clone();
        guidance["layers"]["execution_strategy"]["mock_candidate_count"] =
            mock_summary["mock_candidate_count"].clone();
        guidance["layers"]["execution_strategy"]["hardcoded_candidate_count"] =
            mock_summary["hardcoded_candidate_count"].clone();
        guidance["layers"]["execution_strategy"]["review_mock_data_before_cleanup"] = json!(
            mock_summary["hardcoded_candidate_count"]
                .as_u64()
                .unwrap_or(0)
                > 0
        );
        guidance["layers"]["constraints_boundaries"] = build_constraints_boundaries_layer(
            Some(&repo_risk),
            Some(&guidance["layers"]["verification_evidence"]),
            vec!["Access counts and durations are based on OPENDOG monitoring data.".to_string()],
            vec![
                "Hot files are likely good review targets but still require code inspection."
                    .to_string(),
            ],
            vec![
                "Sampling-based monitoring may miss very brief file accesses.".to_string(),
                "This response does not include git diff, test, or build evidence.".to_string(),
            ],
            vec!["git diff".to_string(), project_commands[0].clone()],
        );
        guidance["layers"]["constraints_boundaries"]["protected_paths"] =
            boundary_hints["protected_paths"].clone();
        guidance["layers"]["constraints_boundaries"]["generated_artifact_directories"] =
            boundary_hints["generated_artifact_directories"].clone();
        guidance["layers"]["constraints_boundaries"]["mock_candidate_count"] =
            mock_summary["mock_candidate_count"].clone();
        guidance["layers"]["constraints_boundaries"]["hardcoded_candidate_count"] =
            mock_summary["hardcoded_candidate_count"].clone();
        guidance["layers"]["constraints_boundaries"]["mixed_review_files"] =
            mock_summary["mixed_review_files"].clone();
        guidance
    } else {
        let mut guidance = tool_guidance(
            "Stats loaded. Query unused files next or inspect the repository with shell tools.",
            &[
                "opendog unused --id <project>",
                "rg \"<pattern>\" .",
                "git diff",
            ],
            &["get_unused_files", "list_projects"],
            Some("Use shell commands once opendog identifies candidate files; opendog itself does not inspect diffs or symbols."),
        );
        guidance["layers"]["workspace_observation"] = json!({
            "status": "available",
            "analysis_state": "ready",
            "snapshot_available": true,
            "activity_available": summary.accessed_files > 0,
            "total_files": summary.total_files,
            "accessed_files": summary.accessed_files,
            "unused_files": summary.unused_files,
        });
        guidance["layers"]["repo_status_risk"] = repo_status_risk_layer(root_path);
        guidance["layers"]["project_toolchain"] = project_toolchain_layer(root_path);
        guidance["layers"]["verification_evidence"] = verification_status_layer(verification_runs);
        guidance
    }
}
