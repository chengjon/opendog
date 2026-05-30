use super::*;

#[test]
fn workspace_verification_evidence_layer_empty() {
    let result = workspace_verification_evidence_layer(&[], 0, 0);
    assert_eq!(result["status"], "available");
    assert_eq!(result["projects_with_recorded_verification"], 0);
    assert_eq!(result["projects_missing_verification"], 0);
    assert_eq!(result["confidence"], "low");
    assert!(result["blocking_projects"].as_array().unwrap().is_empty());
}

#[test]
fn workspace_verification_evidence_layer_single_project_all_passing() {
    let project = json!({
        "project_id": "proj-a",
        "verification_evidence": {
            "status": "available",
            "failing_runs": [],
        },
        "observation": {
            "freshness": {
                "verification": { "status": "fresh" }
            }
        },
        "safe_for_cleanup": true,
        "safe_for_refactor": true,
        "safe_for_cleanup_reason": "ok",
        "safe_for_refactor_reason": "ok",
    });
    let result = workspace_verification_evidence_layer(&[project], 1, 1);
    assert_eq!(result["projects_with_recorded_verification"], 1);
    assert_eq!(result["projects_missing_verification"], 0);
    assert_eq!(result["projects_with_failing_verification"], 0);
    assert_eq!(result["projects_safe_for_cleanup"], 1);
    assert_eq!(result["projects_safe_for_refactor"], 1);
    assert_eq!(result["confidence"], "high");
    assert!(result["blocking_projects"].as_array().unwrap().is_empty());
}

#[test]
fn workspace_verification_evidence_layer_mixed_projects() {
    let passing = json!({
        "project_id": "proj-good",
        "verification_evidence": {
            "status": "available",
            "failing_runs": [],
        },
        "observation": {
            "freshness": {
                "verification": { "status": "fresh" }
            }
        },
        "safe_for_cleanup": true,
        "safe_for_refactor": true,
        "safe_for_cleanup_reason": "ok",
        "safe_for_refactor_reason": "ok",
    });
    let failing = json!({
        "project_id": "proj-bad",
        "verification_evidence": {
            "status": "available",
            "failing_runs": [{"kind": "test", "status": "failed", "command": "make test"}],
        },
        "observation": {
            "freshness": {
                "verification": { "status": "fresh" }
            }
        },
        "safe_for_cleanup": false,
        "safe_for_refactor": false,
        "safe_for_cleanup_reason": "blocked",
        "safe_for_refactor_reason": "blocked",
    });
    let result = workspace_verification_evidence_layer(&[passing, failing], 2, 1);
    assert_eq!(result["projects_with_recorded_verification"], 2);
    assert_eq!(result["projects_with_failing_verification"], 1);
    assert_eq!(result["projects_safe_for_cleanup"], 1);
    assert_eq!(result["blocking_projects"].as_array().unwrap().len(), 1);
}

#[test]
fn workspace_verification_evidence_layer_missing_verification() {
    let project = json!({
        "project_id": "proj-new",
        "verification_evidence": {
            "status": "not_recorded",
            "failing_runs": [],
        },
        "observation": {
            "freshness": {
                "verification": { "status": "missing" }
            }
        },
        "safe_for_cleanup": false,
        "safe_for_refactor": false,
        "safe_for_cleanup_reason": "no evidence",
        "safe_for_refactor_reason": "no evidence",
    });
    let result = workspace_verification_evidence_layer(&[project], 1, 0);
    assert_eq!(result["projects_missing_verification"], 1);
    assert_eq!(result["confidence"], "low");
}

#[test]
fn workspace_verification_evidence_layer_has_gate_distribution() {
    let project = json!({
        "project_id": "p1",
        "verification_evidence": {
            "status": "available",
            "gate_assessment": {
                "cleanup": { "level": "allow" },
                "refactor": { "level": "allow" },
            },
            "failing_runs": [],
        },
        "observation": {
            "freshness": { "verification": { "status": "fresh" } }
        },
        "safe_for_cleanup": true,
        "safe_for_refactor": true,
        "safe_for_cleanup_reason": "ok",
        "safe_for_refactor_reason": "ok",
    });
    let result = workspace_verification_evidence_layer(&[project], 1, 1);
    assert_eq!(result["cleanup_gate_distribution"]["allow"], 1);
    assert_eq!(result["refactor_gate_distribution"]["allow"], 1);
}

#[test]
fn workspace_verification_evidence_layer_direct_observations_count() {
    let result = workspace_verification_evidence_layer(&[], 5, 2);
    let obs = result["direct_observations"].as_array().unwrap();
    assert!(obs
        .iter()
        .any(|o| o.as_str().unwrap().contains("Registered projects: 5")));
    assert!(obs
        .iter()
        .any(|o| o.as_str().unwrap().contains("monitoring: 2")));
}

#[test]
fn workspace_verification_evidence_layer_verified_conclusions() {
    let project = json!({
        "project_id": "p1",
        "verification_evidence": {
            "status": "available",
            "failing_runs": [],
        },
        "observation": {
            "freshness": { "verification": { "status": "fresh" } }
        },
        "safe_for_cleanup": true,
        "safe_for_refactor": true,
        "safe_for_cleanup_reason": "ok",
        "safe_for_refactor_reason": "ok",
    });
    let result = workspace_verification_evidence_layer(&[project], 1, 1);
    let vc = result["verified_conclusions"].as_array().unwrap();
    assert!(vc
        .iter()
        .any(|c| c["summary"].as_str().unwrap().contains("1 project(s)")));
}

#[test]
fn workspace_verification_evidence_layer_unverified_conclusions() {
    let project = json!({
        "project_id": "p1",
        "verification_evidence": {
            "status": "not_recorded",
            "failing_runs": [],
        },
        "observation": {
            "freshness": { "verification": { "status": "missing" } }
        },
        "safe_for_cleanup": false,
        "safe_for_refactor": false,
        "safe_for_cleanup_reason": "no evidence",
        "safe_for_refactor_reason": "no evidence",
    });
    let result = workspace_verification_evidence_layer(&[project], 1, 0);
    let uc = result["unverified_conclusions"].as_array().unwrap();
    assert!(uc.iter().any(|c| c["summary"]
        .as_str()
        .unwrap()
        .contains("missing verification")));
}
