use serde_json::{json, Value};
use std::path::Path;

mod external_truth;
mod model;
mod repo_truth;
mod review_focus;

pub(crate) use self::external_truth::external_truth_boundary_for_top_project;
pub(crate) use self::repo_truth::repo_truth_gap_projection;
pub(crate) use self::review_focus::review_focus_projection_for_top_project;
use super::guidance_types::{ConstraintsBoundariesLayer, ConstraintsBoundariesLayerStatus};
use super::serialization::to_value_or_error;
use model::{ProjectReadinessAssessment, ReadinessTarget};

pub(super) struct WorkspaceCounts {
    pub(super) projects_not_ready_for_cleanup: usize,
    pub(super) projects_not_ready_for_refactor: usize,
    pub(super) projects_with_hardcoded_data_candidates: usize,
    pub(super) projects_missing_snapshot: usize,
    pub(super) projects_with_stale_snapshot: usize,
    pub(super) projects_missing_activity: usize,
    pub(super) projects_with_stale_activity: usize,
    pub(super) projects_missing_verification: usize,
    pub(super) projects_with_stale_verification: usize,
    pub(super) projects_with_storage_maintenance_candidates: u64,
}

fn string_array_field(value: &Value, key: &str) -> Vec<String> {
    value[key]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn default_boundary_guardrails() -> Vec<String> {
    vec![
        "Do not treat activity-derived signals as proof of safety without shell verification."
            .to_string(),
        "Do not start broad cleanup or refactor work while verification is failing or missing."
            .to_string(),
        "Do not perform broad modifications while a repository is mid-merge, rebase, cherry-pick, or bisect."
            .to_string(),
    ]
}

pub(super) fn default_destructive_operations() -> Vec<String> {
    vec![
        "Deleting unused-file candidates without validating imports, runtime paths, and tests."
            .to_string(),
        "Running broad cleanup while verification evidence is missing, failing, or stale."
            .to_string(),
        "Starting large refactors while git reports merge/rebase/cherry-pick/bisect state."
            .to_string(),
    ]
}

#[cfg(test)]
pub(super) fn project_readiness_reasons(
    repo_risk: &Value,
    verification_layer: &Value,
    target: &str,
) -> Vec<String> {
    ProjectReadinessAssessment::from_layers(repo_risk, verification_layer)
        .reasons_for(ReadinessTarget::from_name(target))
        .to_vec()
}

pub(super) fn project_readiness_snapshot(repo_risk: &Value, verification_layer: &Value) -> Value {
    ProjectReadinessAssessment::from_layers(repo_risk, verification_layer).to_json()
}

pub(super) fn readiness_reason_summary(target: &str, safe: bool, reasons: &[String]) -> String {
    ReadinessTarget::from_name(target).reason_summary(safe, reasons)
}

pub(super) fn build_constraints_boundaries_layer(
    repo_risk: Option<&Value>,
    verification_layer: Option<&Value>,
    direct_observations: Vec<String>,
    inferences: Vec<String>,
    blind_spots: Vec<String>,
    requires_shell_verification: Vec<String>,
    workspace_counts: Option<WorkspaceCounts>,
) -> Value {
    let readiness = match (repo_risk, verification_layer) {
        (Some(repo_risk), Some(verification)) => {
            project_readiness_snapshot(repo_risk, verification)
        }
        _ => json!({
            "cleanup_blockers": [],
            "refactor_blockers": [],
        }),
    };
    let cleanup_blockers = string_array_field(&readiness, "cleanup_blockers");
    let refactor_blockers = string_array_field(&readiness, "refactor_blockers");

    let mut human_review_required_for = Vec::new();
    if !cleanup_blockers.is_empty() {
        human_review_required_for
            .push("Cleanup candidates blocked by verification or repository risk.".to_string());
    }
    if !refactor_blockers.is_empty() {
        human_review_required_for
            .push("Refactor candidates blocked by verification or repository risk.".to_string());
    }
    if repo_risk
        .and_then(|risk| risk["lockfile_anomalies"].as_array())
        .map(|items| !items.is_empty())
        .unwrap_or(false)
    {
        human_review_required_for
            .push("Dependency manifest/lockfile mismatch signals.".to_string());
    }

    to_value_or_error(
        "ConstraintsBoundariesLayer",
        ConstraintsBoundariesLayer {
            status: ConstraintsBoundariesLayerStatus::Available,
            direct_observations,
            inferences,
            blind_spots,
            guardrails: default_boundary_guardrails(),
            destructive_operations_requiring_confirmation: default_destructive_operations(),
            human_review_required_for,
            cleanup_blockers,
            refactor_blockers,
            requires_shell_verification,
            projects_not_ready_for_cleanup: workspace_counts
                .as_ref()
                .map(|c| c.projects_not_ready_for_cleanup),
            projects_not_ready_for_refactor: workspace_counts
                .as_ref()
                .map(|c| c.projects_not_ready_for_refactor),
            projects_with_hardcoded_data_candidates: workspace_counts
                .as_ref()
                .map(|c| c.projects_with_hardcoded_data_candidates),
            projects_missing_snapshot: workspace_counts
                .as_ref()
                .map(|c| c.projects_missing_snapshot),
            projects_with_stale_snapshot: workspace_counts
                .as_ref()
                .map(|c| c.projects_with_stale_snapshot),
            projects_missing_activity: workspace_counts
                .as_ref()
                .map(|c| c.projects_missing_activity),
            projects_with_stale_activity: workspace_counts
                .as_ref()
                .map(|c| c.projects_with_stale_activity),
            projects_missing_verification: workspace_counts
                .as_ref()
                .map(|c| c.projects_missing_verification),
            projects_with_stale_verification: workspace_counts
                .as_ref()
                .map(|c| c.projects_with_stale_verification),
            projects_with_storage_maintenance_candidates: workspace_counts
                .as_ref()
                .map(|c| c.projects_with_storage_maintenance_candidates),
        },
    )
}

pub(super) fn common_boundary_hints(root: &Path) -> Value {
    let protected_paths = [".git", ".opendog"]
        .iter()
        .filter(|path| root.join(path).exists())
        .map(|path| (*path).to_string())
        .collect::<Vec<_>>();
    let generated_artifact_directories = [
        "target",
        "node_modules",
        "dist",
        "build",
        ".next",
        "coverage",
        ".turbo",
    ]
    .iter()
    .filter(|path| root.join(path).exists())
    .map(|path| (*path).to_string())
    .collect::<Vec<_>>();

    json!({
        "protected_paths": protected_paths,
        "generated_artifact_directories": generated_artifact_directories,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- default_boundary_guardrails ---

    #[test]
    fn default_boundary_guardrails_has_three_entries() {
        let guardrails = default_boundary_guardrails();
        assert_eq!(guardrails.len(), 3);
        assert!(guardrails[0].contains("activity-derived signals"));
        assert!(guardrails[1].contains("verification is failing or missing"));
        assert!(guardrails[2].contains("mid-merge, rebase, cherry-pick, or bisect"));
    }

    // --- default_destructive_operations ---

    #[test]
    fn default_destructive_operations_has_three_entries() {
        let ops = default_destructive_operations();
        assert_eq!(ops.len(), 3);
        assert!(ops[0].contains("Deleting unused-file candidates"));
        assert!(ops[1].contains("verification evidence is missing, failing, or stale"));
        assert!(ops[2].contains("merge/rebase/cherry-pick/bisect state"));
    }

    // --- string_array_field ---

    #[test]
    fn string_array_field_extracts_strings() {
        let value = json!({"items": ["alpha", "beta", "gamma"]});
        let result = string_array_field(&value, "items");
        assert_eq!(result, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn string_array_field_missing_key_returns_empty() {
        let value = json!({"other_key": ["a"]});
        let result = string_array_field(&value, "items");
        assert!(result.is_empty());
    }

    #[test]
    fn string_array_field_non_array_returns_empty() {
        let value = json!({"items": "not_an_array"});
        let result = string_array_field(&value, "items");
        assert!(result.is_empty());
    }

    #[test]
    fn string_array_field_filters_non_string_items() {
        let value = json!({"items": ["valid", 42, true, null, "also_valid"]});
        let result = string_array_field(&value, "items");
        assert_eq!(result, vec!["valid", "also_valid"]);
    }

    // --- project_readiness_reasons ---

    fn clean_repo_risk() -> Value {
        json!({
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": false,
            "changed_file_count": 0,
        })
    }

    fn clean_verification_layer() -> Value {
        json!({
            "cleanup_blockers": [],
            "refactor_blockers": [],
            "safe_for_cleanup": true,
            "safe_for_refactor": true,
            "gate_assessment": {
                "cleanup": { "level": "allow" },
                "refactor": { "level": "allow" },
            },
        })
    }

    #[test]
    fn project_readiness_reasons_clean_state_no_reasons() {
        let repo = clean_repo_risk();
        let verification = clean_verification_layer();
        let reasons = project_readiness_reasons(&repo, &verification, "cleanup");
        assert!(reasons.is_empty());
    }

    #[test]
    fn project_readiness_reasons_has_cleanup_blockers() {
        let repo = clean_repo_risk();
        let mut verification = clean_verification_layer();
        verification["cleanup_blockers"] = json!(["Missing test evidence"]);
        let reasons = project_readiness_reasons(&repo, &verification, "cleanup");
        assert_eq!(reasons, vec!["Missing test evidence"]);
    }

    #[test]
    fn project_readiness_reasons_has_refactor_blockers() {
        let repo = clean_repo_risk();
        let mut verification = clean_verification_layer();
        verification["refactor_blockers"] = json!(["Lint is failing", "No build run"]);
        let reasons = project_readiness_reasons(&repo, &verification, "refactor");
        assert_eq!(reasons, vec!["Lint is failing", "No build run"]);
    }

    #[test]
    fn project_readiness_reasons_mid_operation() {
        let mut repo = clean_repo_risk();
        repo["operation_states"] = json!(["merge", "rebase"]);
        let verification = clean_verification_layer();
        let reasons = project_readiness_reasons(&repo, &verification, "cleanup");
        assert!(reasons
            .iter()
            .any(|r| r.contains("mid-operation") && r.contains("merge, rebase")));
    }

    #[test]
    fn project_readiness_reasons_conflicted_paths() {
        let mut repo = clean_repo_risk();
        repo["conflicted_count"] = json!(3);
        let verification = clean_verification_layer();
        let reasons = project_readiness_reasons(&repo, &verification, "cleanup");
        assert!(reasons.iter().any(|r| r.contains("3 conflicted paths")));
    }

    #[test]
    fn project_readiness_reasons_lockfile_anomalies() {
        let mut repo = clean_repo_risk();
        repo["lockfile_anomalies"] = json!([{"file": "package.json"}]);
        let verification = clean_verification_layer();
        let reasons = project_readiness_reasons(&repo, &verification, "cleanup");
        assert!(reasons.iter().any(|r| r.contains("mismatches are present")));
    }

    #[test]
    fn project_readiness_reasons_large_diff_only_for_refactor() {
        let mut repo = clean_repo_risk();
        repo["large_diff"] = json!(true);
        repo["changed_file_count"] = json!(50);
        let verification = clean_verification_layer();
        // cleanup target should NOT include large_diff reason
        let cleanup_reasons = project_readiness_reasons(&repo, &verification, "cleanup");
        assert!(!cleanup_reasons.iter().any(|r| r.contains("large diff")));
        // refactor target SHOULD include large_diff reason
        let refactor_reasons = project_readiness_reasons(&repo, &verification, "refactor");
        assert!(refactor_reasons.iter().any(|r| r.contains("large diff")));
    }

    // --- project_readiness_snapshot ---

    #[test]
    fn project_readiness_snapshot_all_clear() {
        let repo = clean_repo_risk();
        let verification = clean_verification_layer();
        let snap = project_readiness_snapshot(&repo, &verification);
        assert_eq!(snap["safe_for_cleanup"], true);
        assert_eq!(snap["safe_for_refactor"], true);
        assert_eq!(snap["verification_safe_for_cleanup"], true);
        assert_eq!(snap["verification_safe_for_refactor"], true);
        assert_eq!(snap["cleanup_gate_level"], "allow");
        assert_eq!(snap["refactor_gate_level"], "allow");
        let cleanup_blockers = snap["cleanup_blockers"].as_array().unwrap();
        assert!(cleanup_blockers.is_empty());
    }

    #[test]
    fn project_readiness_snapshot_blocked_cleanup() {
        let repo = clean_repo_risk();
        let mut verification = clean_verification_layer();
        verification["safe_for_cleanup"] = json!(false);
        verification["cleanup_blockers"] = json!(["Tests are failing"]);
        let snap = project_readiness_snapshot(&repo, &verification);
        assert_eq!(snap["safe_for_cleanup"], false);
        let reason = snap["safe_for_cleanup_reason"].as_str().unwrap();
        assert_eq!(reason, "Tests are failing");
    }

    #[test]
    fn project_readiness_snapshot_blocked_refactor_with_repo_risk() {
        let mut repo = clean_repo_risk();
        repo["operation_states"] = json!(["rebase"]);
        let verification = clean_verification_layer();
        let snap = project_readiness_snapshot(&repo, &verification);
        assert_eq!(snap["safe_for_refactor"], false);
        let reason = snap["safe_for_refactor_reason"].as_str().unwrap();
        assert!(reason.contains("mid-operation"));
    }

    // --- readiness_reason_summary ---

    #[test]
    fn readiness_reason_summary_safe_cleanup() {
        let summary = readiness_reason_summary("cleanup", true, &[]);
        assert!(summary.contains("cleanup review"));
        assert!(summary.contains("verification gates passed"));
    }

    #[test]
    fn readiness_reason_summary_safe_refactor() {
        let summary = readiness_reason_summary("refactor", true, &[]);
        assert!(summary.contains("scoped refactor work"));
    }

    #[test]
    fn readiness_reason_summary_unsafe_with_reasons() {
        let reasons = vec!["Tests are failing".to_string()];
        let summary = readiness_reason_summary("cleanup", false, &reasons);
        assert_eq!(summary, "Tests are failing");
    }

    #[test]
    fn readiness_reason_summary_unsafe_without_reasons_cleanup() {
        let summary = readiness_reason_summary("cleanup", false, &[]);
        assert!(summary.contains("blocked by missing evidence"));
    }

    #[test]
    fn readiness_reason_summary_unsafe_without_reasons_refactor() {
        let summary = readiness_reason_summary("refactor", false, &[]);
        assert!(summary.contains("Refactor readiness is blocked"));
    }

    // --- build_constraints_boundaries_layer ---

    #[test]
    fn build_constraints_boundaries_layer_no_repo_or_verification() {
        let result = build_constraints_boundaries_layer(
            None,
            None,
            vec!["obs1".to_string()],
            vec!["inf1".to_string()],
            vec!["blind1".to_string()],
            vec!["verify1".to_string()],
            None,
        );
        assert_eq!(result["status"], "available");
        assert_eq!(result["direct_observations"].as_array().unwrap().len(), 1);
        assert_eq!(result["inferences"].as_array().unwrap().len(), 1);
        assert_eq!(result["blind_spots"].as_array().unwrap().len(), 1);
        assert_eq!(
            result["requires_shell_verification"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        // No workspace counts — field is skipped when Option is None
        assert!(result.get("projects_not_ready_for_cleanup").is_none());
    }

    #[test]
    fn build_constraints_boundaries_layer_with_workspace_counts() {
        let counts = WorkspaceCounts {
            projects_not_ready_for_cleanup: 2,
            projects_not_ready_for_refactor: 3,
            projects_with_hardcoded_data_candidates: 1,
            projects_missing_snapshot: 4,
            projects_with_stale_snapshot: 5,
            projects_missing_activity: 6,
            projects_with_stale_activity: 7,
            projects_missing_verification: 8,
            projects_with_stale_verification: 9,
            projects_with_storage_maintenance_candidates: 10,
        };
        let result = build_constraints_boundaries_layer(
            None,
            None,
            vec![],
            vec![],
            vec![],
            vec![],
            Some(counts),
        );
        assert_eq!(result["projects_not_ready_for_cleanup"], 2);
        assert_eq!(result["projects_not_ready_for_refactor"], 3);
        assert_eq!(result["projects_with_hardcoded_data_candidates"], 1);
        assert_eq!(result["projects_missing_snapshot"], 4);
        assert_eq!(result["projects_with_storage_maintenance_candidates"], 10);
    }

    #[test]
    fn build_constraints_boundaries_layer_with_blockers_adds_human_review() {
        let repo = json!({
            "operation_states": ["merge"],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": false,
            "changed_file_count": 0,
        });
        let verification = json!({
            "cleanup_blockers": ["Missing verification"],
            "refactor_blockers": [],
            "safe_for_cleanup": false,
            "safe_for_refactor": true,
            "gate_assessment": {
                "cleanup": { "level": "blocked" },
                "refactor": { "level": "allow" },
            },
        });
        let result = build_constraints_boundaries_layer(
            Some(&repo),
            Some(&verification),
            vec![],
            vec![],
            vec![],
            vec![],
            None,
        );
        let human_review = result["human_review_required_for"].as_array().unwrap();
        // cleanup_blockers: verification blocker + mid-operation => non-empty
        // refactor_blockers: mid-operation => non-empty
        // So both "Cleanup candidates blocked" and "Refactor candidates blocked" are added
        assert_eq!(human_review.len(), 2);
        assert!(human_review
            .iter()
            .any(|h| h.as_str().unwrap().contains("Cleanup candidates blocked")));
        assert!(human_review
            .iter()
            .any(|h| h.as_str().unwrap().contains("Refactor candidates blocked")));
    }

    #[test]
    fn build_constraints_boundaries_layer_lockfile_anomalies_adds_human_review() {
        let repo = json!({
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [{"file": "Cargo.lock"}],
            "large_diff": false,
            "changed_file_count": 0,
        });
        let verification = json!({
            "cleanup_blockers": [],
            "refactor_blockers": [],
            "safe_for_cleanup": true,
            "safe_for_refactor": true,
            "gate_assessment": {
                "cleanup": { "level": "allow" },
                "refactor": { "level": "allow" },
            },
        });
        let result = build_constraints_boundaries_layer(
            Some(&repo),
            Some(&verification),
            vec![],
            vec![],
            vec![],
            vec![],
            None,
        );
        let human_review = result["human_review_required_for"].as_array().unwrap();
        // lockfile anomalies produce cleanup_blockers and refactor_blockers entries
        // via project_readiness_reasons, plus the direct lockfile entry
        assert_eq!(human_review.len(), 3);
        assert!(human_review
            .iter()
            .any(|h| h.as_str().unwrap().contains("lockfile mismatch")));
    }

    #[test]
    fn build_constraints_boundaries_layer_guardrails_always_present() {
        let result =
            build_constraints_boundaries_layer(None, None, vec![], vec![], vec![], vec![], None);
        let guardrails = result["guardrails"].as_array().unwrap();
        assert_eq!(guardrails.len(), 3);
        let destructive = result["destructive_operations_requiring_confirmation"]
            .as_array()
            .unwrap();
        assert_eq!(destructive.len(), 3);
    }
}
