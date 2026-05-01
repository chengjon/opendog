use serde_json::{json, Value};
use std::path::Path;

mod repo_truth;

pub(crate) use self::repo_truth::repo_truth_gap_projection;

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

pub(super) fn project_readiness_reasons(
    repo_risk: &Value,
    verification_layer: &Value,
    target: &str,
) -> Vec<String> {
    let mut reasons = match target {
        "refactor" => string_array_field(verification_layer, "refactor_blockers"),
        _ => string_array_field(verification_layer, "cleanup_blockers"),
    };

    let operation_states = string_array_field(repo_risk, "operation_states");
    if !operation_states.is_empty() {
        reasons.push(format!(
            "Repository is mid-operation: {}.",
            operation_states.join(", ")
        ));
    }

    let conflicted_count = repo_risk["conflicted_count"].as_u64().unwrap_or(0);
    if conflicted_count > 0 {
        reasons.push(format!(
            "Repository has {} conflicted paths in the working tree.",
            conflicted_count
        ));
    }

    let lockfile_anomalies = repo_risk["lockfile_anomalies"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    if lockfile_anomalies > 0 {
        reasons.push(format!(
            "Dependency manifest/lockfile mismatches are present ({} signals).",
            lockfile_anomalies
        ));
    }

    if target == "refactor" && repo_risk["large_diff"].as_bool().unwrap_or(false) {
        let changed_file_count = repo_risk["changed_file_count"].as_u64().unwrap_or(0);
        reasons.push(format!(
            "Working tree already has a large diff ({} changed files), so broad refactors should wait.",
            changed_file_count
        ));
    }

    reasons
}

pub(super) fn project_readiness_snapshot(repo_risk: &Value, verification_layer: &Value) -> Value {
    let cleanup_blockers = project_readiness_reasons(repo_risk, verification_layer, "cleanup");
    let refactor_blockers = project_readiness_reasons(repo_risk, verification_layer, "refactor");
    let verification_safe_for_cleanup = verification_layer["safe_for_cleanup"]
        .as_bool()
        .unwrap_or(false);
    let verification_safe_for_refactor = verification_layer["safe_for_refactor"]
        .as_bool()
        .unwrap_or(false);
    let cleanup_level = verification_layer["gate_assessment"]["cleanup"]["level"]
        .as_str()
        .unwrap_or(if verification_safe_for_cleanup {
            "allow"
        } else {
            "blocked"
        });
    let refactor_level = verification_layer["gate_assessment"]["refactor"]["level"]
        .as_str()
        .unwrap_or(if verification_safe_for_refactor {
            "allow"
        } else {
            "blocked"
        });
    let safe_for_cleanup = verification_safe_for_cleanup && cleanup_blockers.is_empty();
    let safe_for_refactor = verification_safe_for_refactor && refactor_blockers.is_empty();

    json!({
        "verification_safe_for_cleanup": verification_safe_for_cleanup,
        "verification_safe_for_refactor": verification_safe_for_refactor,
        "cleanup_gate_level": cleanup_level,
        "refactor_gate_level": refactor_level,
        "safe_for_cleanup": safe_for_cleanup,
        "safe_for_cleanup_reason": readiness_reason_summary("cleanup", safe_for_cleanup, &cleanup_blockers),
        "cleanup_blockers": cleanup_blockers,
        "safe_for_refactor": safe_for_refactor,
        "safe_for_refactor_reason": readiness_reason_summary("refactor", safe_for_refactor, &refactor_blockers),
        "refactor_blockers": refactor_blockers,
    })
}

pub(super) fn readiness_reason_summary(target: &str, safe: bool, reasons: &[String]) -> String {
    if safe {
        match target {
            "refactor" => {
                "Current evidence supports scoped refactor work: verification gates passed and no repository-level blocker is active."
                    .to_string()
            }
            _ => {
                "Current evidence supports cleanup review: required verification gates passed and no repository-level blocker is active."
                    .to_string()
            }
        }
    } else if let Some(reason) = reasons.first() {
        reason.clone()
    } else {
        match target {
            "refactor" => {
                "Refactor readiness is blocked by missing evidence or repository risk.".to_string()
            }
            _ => "Cleanup readiness is blocked by missing evidence or repository risk.".to_string(),
        }
    }
}

pub(super) fn build_constraints_boundaries_layer(
    repo_risk: Option<&Value>,
    verification_layer: Option<&Value>,
    direct_observations: Vec<String>,
    inferences: Vec<String>,
    blind_spots: Vec<String>,
    requires_shell_verification: Vec<String>,
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

    json!({
        "status": "available",
        "direct_observations": direct_observations,
        "inferences": inferences,
        "blind_spots": blind_spots,
        "guardrails": default_boundary_guardrails(),
        "destructive_operations_requiring_confirmation": default_destructive_operations(),
        "human_review_required_for": human_review_required_for,
        "cleanup_blockers": cleanup_blockers,
        "refactor_blockers": refactor_blockers,
        "requires_shell_verification": requires_shell_verification,
    })
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
