use serde_json::{json, Value};
use std::path::Path;

use crate::storage::queries::{StatsEntry, VerificationRun};

use super::super::super::{
    build_constraints_boundaries_layer, common_boundary_hints, detect_mock_data_report,
    detect_project_commands, project_readiness_snapshot, project_toolchain_layer,
    repo_status_risk_layer, tool_guidance, verification_status_layer,
};

pub(in crate::mcp) fn unused_guidance(
    root_path: &Path,
    unused_entries: &[StatsEntry],
    verification_runs: &[VerificationRun],
) -> Value {
    let unused_count = unused_entries.len();
    let project_commands = detect_project_commands(root_path);
    if unused_count == 0 {
        let mut guidance = tool_guidance(
            "No unused files were found in the current snapshot. Shift to shell inspection or targeted stats analysis.",
            &[
                "opendog stats --id <project>",
                "rg \"<pattern>\" .",
                "git diff",
            ],
            &["get_stats", "list_projects"],
            Some("Use shell commands for symbol search, diffs, and tests once unused-file screening is complete."),
        );
        guidance["layers"]["workspace_observation"] = json!({
            "status": "available",
            "analysis_state": "ready",
            "snapshot_available": true,
            "activity_available": true,
            "unused_candidates": 0,
        });
        guidance["layers"]["repo_status_risk"] = repo_status_risk_layer(root_path);
        guidance["layers"]["project_toolchain"] = project_toolchain_layer(root_path);
        guidance["layers"]["verification_evidence"] = verification_status_layer(verification_runs);
        guidance
    } else {
        let mut guidance = tool_guidance(
            "Unused file list loaded. Verify with shell search or tests before deleting anything.",
            &["rg \"<pattern>\" .", "git grep <symbol>", "cargo test"],
            &["get_stats", "list_projects"],
            Some("Use shell commands to validate whether a supposedly unused file still matters through build, imports, or tests."),
        );
        let boundary_hints = common_boundary_hints(root_path);
        let repo_risk = repo_status_risk_layer(root_path);
        let verification_layer = verification_status_layer(verification_runs);
        let readiness = project_readiness_snapshot(&repo_risk, &verification_layer);
        let mock_summary = detect_mock_data_report(root_path, unused_entries).to_value(5);
        let file_recommendations: Vec<Value> = unused_entries
            .iter()
            .take(3)
            .map(|entry| {
                json!({
                    "kind": "unused_candidate",
                    "file_path": entry.file_path,
                    "reason": "This file has not been observed as accessed in the current snapshot window.",
                    "suggested_commands": [
                        format!("rg \"{}\" .", entry.file_path),
                        "git grep <symbol>".to_string(),
                        project_commands[0].clone()
                    ]
                })
            })
            .collect();
        guidance["file_recommendations"] = json!(file_recommendations.clone());
        guidance["layers"]["workspace_observation"] = json!({
            "status": "available",
            "analysis_state": "ready",
            "snapshot_available": true,
            "activity_available": true,
            "unused_candidates": unused_count,
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
        evidence["direct_observations"] =
            json!([format!("Current unused candidate count: {}.", unused_count)]);
        evidence["inferences"] = json!([
            "Unused candidates should be validated against imports, runtime use, and tests before cleanup.",
        ]);
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
            vec!["These files were not observed as accessed in the current snapshot window."
                .to_string()],
            vec![
                "Unused candidates may still matter through indirect runtime paths or infrequent workflows."
                    .to_string(),
            ],
            vec!["Lack of observed access is not proof that a file is safe to delete.".to_string()],
            vec!["git grep <symbol>".to_string(), project_commands[0].clone()],
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
    }
}
