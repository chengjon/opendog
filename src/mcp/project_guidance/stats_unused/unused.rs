use serde_json::{json, Value};
use std::path::Path;

use crate::core::file_classification::{
    classify_file_path, FilePathClassification, FilePathClassificationFilter,
};
use crate::storage::queries::{StatsEntry, VerificationRun};

use super::super::super::review_candidates::{
    build_review_candidate, CandidateFreshness, ReviewCandidateContext,
};
use super::super::super::{
    build_constraints_boundaries_layer, common_boundary_hints, detect_mock_data_report,
    detect_project_commands, project_readiness_snapshot, project_toolchain_layer,
    repo_status_risk_layer, tool_guidance, verification_status_layer,
};

pub(in crate::mcp) fn unused_guidance(
    root_path: &Path,
    unused_entries: &[StatsEntry],
    verification_runs: &[VerificationRun],
    path_filter: FilePathClassificationFilter,
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
        apply_path_filter_observation(&mut guidance, path_filter, unused_count);
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
        let context = ReviewCandidateContext {
            mock_summary: &mock_summary,
            freshness: CandidateFreshness::default(),
            repo_risk: &repo_risk,
        };
        let file_recommendations: Vec<Value> = unused_entries
            .iter()
            .filter(|entry| classify_file_path(&entry.file_path) == FilePathClassification::Source)
            .chain(unused_entries.iter().filter(|entry| {
                classify_file_path(&entry.file_path) != FilePathClassification::Source
            }))
            .enumerate()
            .take(3)
            .map(|(idx, entry)| {
                build_review_candidate(
                    "unused_candidate",
                    &entry.file_path,
                    if idx == 0 { "primary" } else { "secondary" },
                    "This file has not been observed as accessed in the current snapshot window.",
                    vec![
                        format!("rg \"{}\" .", entry.file_path),
                        "git grep <symbol>".to_string(),
                        project_commands[0].clone(),
                    ],
                    context,
                )
            })
            .collect();
        guidance["file_recommendations"] = json!(file_recommendations.clone());
        guidance["layers"]["workspace_observation"] = json!({
            "status": "available",
            "analysis_state": "ready",
            "snapshot_available": true,
            "activity_available": true,
            "unused_candidates": unused_count,
            "source_candidates": count_classification(unused_entries, FilePathClassification::Source),
            "infrastructure_candidates": count_classification(unused_entries, FilePathClassification::Infrastructure),
            "backup_candidates": count_classification(unused_entries, FilePathClassification::Backup),
        });
        apply_path_filter_observation(&mut guidance, path_filter, unused_count);
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
            vec!["access_count=0 means OPENDOG did not observe an open descriptor; it is not proof that the file was never read or is safe to delete.".to_string()],
            vec!["git grep <symbol>".to_string(), project_commands[0].clone()],
            None,
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

fn apply_path_filter_observation(
    guidance: &mut Value,
    path_filter: FilePathClassificationFilter,
    filtered_rows: usize,
) {
    if path_filter == FilePathClassificationFilter::All {
        return;
    }

    guidance["layers"]["workspace_observation"]["path_classification_filter"] =
        json!(path_filter.as_str());

    if filtered_rows == 0 {
        let note = "selected path_classification filter returned no rows; this does not mean the project has no files or no unused candidates";
        guidance["layers"]["workspace_observation"]["filter_note"] = json!(note);
        guidance["layers"]["verification_evidence"]["inferences"] = json!([note]);
    }
}

fn count_classification(entries: &[StatsEntry], classification: FilePathClassification) -> usize {
    entries
        .iter()
        .filter(|entry| classify_file_path(&entry.file_path) == classification)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_entry(path: &str, access_count: i64) -> StatsEntry {
        StatsEntry {
            file_path: path.to_string(),
            size: 100,
            file_type: "file".to_string(),
            access_count,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }
    }

    // --- count_classification ---

    #[test]
    fn count_classification_empty_entries() {
        let entries: Vec<StatsEntry> = vec![];
        assert_eq!(
            count_classification(&entries, FilePathClassification::Source),
            0
        );
        assert_eq!(
            count_classification(&entries, FilePathClassification::Infrastructure),
            0
        );
        assert_eq!(
            count_classification(&entries, FilePathClassification::Backup),
            0
        );
        assert_eq!(
            count_classification(&entries, FilePathClassification::Project),
            0
        );
    }

    #[test]
    fn count_classification_counts_source_files() {
        let entries = vec![
            make_entry("src/main.rs", 5),
            make_entry("lib/app.py", 3),
            make_entry("index.js", 1),
        ];
        assert_eq!(
            count_classification(&entries, FilePathClassification::Source),
            3
        );
        assert_eq!(
            count_classification(&entries, FilePathClassification::Infrastructure),
            0
        );
    }

    #[test]
    fn count_classification_counts_infrastructure_files() {
        let entries = vec![
            make_entry(".claude/settings.json", 0),
            make_entry(".cursor/rules/guide.mdc", 0),
            make_entry("src/main.rs", 5),
        ];
        assert_eq!(
            count_classification(&entries, FilePathClassification::Infrastructure),
            2
        );
        assert_eq!(
            count_classification(&entries, FilePathClassification::Source),
            1
        );
    }

    #[test]
    fn count_classification_counts_backup_files() {
        let entries = vec![
            make_entry("notes.txt~", 0),
            make_entry("config.yaml.bak", 0),
            make_entry("src/main.rs", 5),
        ];
        assert_eq!(
            count_classification(&entries, FilePathClassification::Backup),
            2
        );
    }

    #[test]
    fn count_classification_counts_project_files() {
        let entries = vec![
            make_entry("README", 0),
            make_entry("LICENSE", 0),
            make_entry("Makefile", 0),
            make_entry("src/main.rs", 5),
        ];
        assert_eq!(
            count_classification(&entries, FilePathClassification::Project),
            3
        );
    }

    #[test]
    fn count_classification_mixed_entries() {
        let entries = vec![
            make_entry("src/main.rs", 10),
            make_entry("lib/utils.py", 5),
            make_entry(".claude/CLAUDE.md", 0),
            make_entry("config.toml.bak", 0),
            make_entry("Cargo.toml", 0),
            make_entry("README", 0),
        ];
        assert_eq!(
            count_classification(&entries, FilePathClassification::Source),
            2
        );
        assert_eq!(
            count_classification(&entries, FilePathClassification::Infrastructure),
            1
        );
        assert_eq!(
            count_classification(&entries, FilePathClassification::Backup),
            1
        );
        assert_eq!(
            count_classification(&entries, FilePathClassification::Project),
            2
        );
    }

    // --- apply_path_filter_observation ---

    #[test]
    fn apply_path_filter_all_does_not_mutate_guidance() {
        let mut guidance = json!({
            "layers": {
                "workspace_observation": {},
                "verification_evidence": {}
            }
        });
        apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::All, 5);
        assert!(guidance["layers"]["workspace_observation"]
            .get("path_classification_filter")
            .is_none());
        assert!(guidance["layers"]["workspace_observation"]
            .get("filter_note")
            .is_none());
    }

    #[test]
    fn apply_path_filter_source_sets_filter_field() {
        let mut guidance = json!({
            "layers": {
                "workspace_observation": {},
                "verification_evidence": {}
            }
        });
        apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::Source, 5);
        assert_eq!(
            guidance["layers"]["workspace_observation"]["path_classification_filter"],
            "source"
        );
    }

    #[test]
    fn apply_path_filter_infrastructure_sets_filter_field() {
        let mut guidance = json!({
            "layers": {
                "workspace_observation": {},
                "verification_evidence": {}
            }
        });
        apply_path_filter_observation(
            &mut guidance,
            FilePathClassificationFilter::Infrastructure,
            3,
        );
        assert_eq!(
            guidance["layers"]["workspace_observation"]["path_classification_filter"],
            "infrastructure"
        );
    }

    #[test]
    fn apply_path_filter_with_zero_rows_adds_filter_note() {
        let mut guidance = json!({
            "layers": {
                "workspace_observation": {},
                "verification_evidence": {}
            }
        });
        apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::Backup, 0);
        assert!(guidance["layers"]["workspace_observation"]["filter_note"]
            .as_str()
            .unwrap()
            .contains("filter returned no rows"));
        assert!(guidance["layers"]["verification_evidence"]["inferences"]
            .as_array()
            .is_some());
    }

    #[test]
    fn apply_path_filter_with_nonzero_rows_does_not_add_filter_note() {
        let mut guidance = json!({
            "layers": {
                "workspace_observation": {},
                "verification_evidence": {}
            }
        });
        apply_path_filter_observation(&mut guidance, FilePathClassificationFilter::Project, 7);
        assert!(guidance["layers"]["workspace_observation"]
            .get("filter_note")
            .is_none());
    }
}
