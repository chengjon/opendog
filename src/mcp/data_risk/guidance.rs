use serde_json::{json, Value};
use std::path::Path;

use crate::storage::queries::StatsEntry;

use super::super::detect_mock_data_report;
use super::super::versioned_project_payload;
use super::super::{common_boundary_hints, set_recommended_flow, tool_guidance};
use super::MockDataReport;

pub(crate) fn data_risk_guidance(root_path: &Path, report: &MockDataReport) -> Value {
    let boundary_hints = common_boundary_hints(root_path);
    let rendered = report.to_value(10);
    let focus = rendered["data_risk_focus"].clone();
    let summary = if report.hardcoded_candidates.is_empty() && report.mock_candidates.is_empty() {
        "No mock or hardcoded data candidates were detected in the current snapshot-derived file set."
    } else if !report.hardcoded_candidates.is_empty() {
        "Hardcoded data candidates detected. Review runtime-shared files before cleanup or refactor work."
    } else {
        "Mock-style data candidates detected. Confirm whether they are test-only artifacts before acting on cleanup suggestions."
    };

    let mut guidance = tool_guidance(
        summary,
        &[
            "rg \"mock|fixture|fake|stub|sample|demo|seed\" .",
            "rg \"customer|invoice|email|address|payment|tenant\" .",
            "git diff",
        ],
        &["get_stats", "get_unused_files", "get_agent_guidance"],
        Some("Use shell commands to confirm whether detected data candidates are test-only artifacts or real runtime liabilities."),
    );
    if report.hardcoded_candidates.is_empty() && report.mock_candidates.is_empty() {
        set_recommended_flow(
            &mut guidance,
            &[
                "No mock or hardcoded-data candidates were detected.",
                "If cleanup is still planned, inspect unused files and verification evidence next.",
                "Use shell verification before broad edits or deletions.",
            ],
        );
    } else if !report.hardcoded_candidates.is_empty() {
        set_recommended_flow(
            &mut guidance,
            &[
                "Review high-priority hardcoded-data candidates first.",
                "Inspect mixed-review files before cleanup or refactor.",
                "Use shell search and diff to confirm whether findings are runtime liabilities.",
                "Check verification evidence before broad edits.",
            ],
        );
    } else {
        set_recommended_flow(
            &mut guidance,
            &[
                "Review mock-style candidates and confirm whether they are test-only artifacts.",
                "Inspect any shared-runtime paths before dismissing findings.",
                "Use shell search if you need direct file context before cleanup.",
            ],
        );
    }
    guidance["data_risk_focus"] = focus.clone();
    guidance["layers"]["workspace_observation"] = json!({
        "status": "available",
        "analysis_state": "ready",
        "mock_candidate_count": rendered["mock_candidate_count"].clone(),
        "hardcoded_candidate_count": rendered["hardcoded_candidate_count"].clone(),
        "mixed_review_file_count": rendered["mixed_review_file_count"].clone(),
    });
    guidance["layers"]["cleanup_refactor_candidates"] = json!({
        "status": "available",
        "data_risk_focus": focus,
        "mock_data_candidates": rendered["mock_data_candidates"].clone(),
        "hardcoded_data_candidates": rendered["hardcoded_data_candidates"].clone(),
        "mixed_review_files": rendered["mixed_review_files"].clone(),
    });
    guidance["layers"]["execution_strategy"]["mock_candidate_count"] =
        rendered["mock_candidate_count"].clone();
    guidance["layers"]["execution_strategy"]["hardcoded_candidate_count"] =
        rendered["hardcoded_candidate_count"].clone();
    guidance["layers"]["execution_strategy"]["review_mock_data_before_cleanup"] =
        json!(rendered["hardcoded_candidate_count"].as_u64().unwrap_or(0) > 0);
    guidance["layers"]["constraints_boundaries"]["protected_paths"] =
        boundary_hints["protected_paths"].clone();
    guidance["layers"]["constraints_boundaries"]["generated_artifact_directories"] =
        boundary_hints["generated_artifact_directories"].clone();
    guidance["layers"]["constraints_boundaries"]["mock_candidate_count"] =
        rendered["mock_candidate_count"].clone();
    guidance["layers"]["constraints_boundaries"]["hardcoded_candidate_count"] =
        rendered["hardcoded_candidate_count"].clone();
    guidance["layers"]["constraints_boundaries"]["mixed_review_files"] =
        rendered["mixed_review_files"].clone();
    guidance
}

pub(crate) fn project_data_risk_payload(
    schema_version: &str,
    id: &str,
    candidate_type: &str,
    min_review_priority: &str,
    limit: usize,
    root_path: &Path,
    entries: &[StatsEntry],
) -> Value {
    let report = detect_mock_data_report(root_path, entries);
    let filtered = report.filtered(candidate_type, Some(min_review_priority));
    let rendered = filtered.to_value(limit.max(1));
    versioned_project_payload(
        schema_version,
        id,
        [
            ("candidate_type", json!(candidate_type)),
            ("min_review_priority", json!(min_review_priority)),
            (
                "mock_candidate_count",
                rendered["mock_candidate_count"].clone(),
            ),
            (
                "hardcoded_candidate_count",
                rendered["hardcoded_candidate_count"].clone(),
            ),
            (
                "mixed_review_file_count",
                rendered["mixed_review_file_count"].clone(),
            ),
            ("data_risk_focus", rendered["data_risk_focus"].clone()),
            (
                "rule_groups_summary",
                rendered["rule_groups_summary"].clone(),
            ),
            ("rule_hits_summary", rendered["rule_hits_summary"].clone()),
            (
                "mock_data_candidates",
                rendered["mock_data_candidates"].clone(),
            ),
            (
                "hardcoded_data_candidates",
                rendered["hardcoded_data_candidates"].clone(),
            ),
            ("mixed_review_files", rendered["mixed_review_files"].clone()),
            ("guidance", data_risk_guidance(root_path, &filtered)),
        ],
    )
}
