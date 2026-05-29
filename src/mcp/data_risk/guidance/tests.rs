use super::*;
use crate::mcp::data_risk::{DataCandidate, MockDataReport};
use tempfile::TempDir;

fn empty_report() -> MockDataReport {
    MockDataReport::default()
}

fn report_with_mock() -> MockDataReport {
    MockDataReport {
        mock_candidates: vec![DataCandidate {
            file_path: "tests/mock_data.json".to_string(),
            confidence: "medium",
            review_priority: "medium",
            path_classification: "test_only",
            rule_hits: vec!["path.mock_token".to_string()],
            matched_keywords: vec![],
            reasons: vec![],
            evidence: vec![],
            access_count: 5,
            file_type: "json".to_string(),
        }],
        hardcoded_candidates: vec![],
        mixed_review_files: vec![],
    }
}

fn report_with_hardcoded() -> MockDataReport {
    MockDataReport {
        mock_candidates: vec![],
        hardcoded_candidates: vec![DataCandidate {
            file_path: "src/config.py".to_string(),
            confidence: "high",
            review_priority: "high",
            path_classification: "runtime_shared",
            rule_hits: vec!["path.runtime_shared".to_string()],
            matched_keywords: vec![],
            reasons: vec![],
            evidence: vec![],
            access_count: 10,
            file_type: "py".to_string(),
        }],
        mixed_review_files: vec![],
    }
}

#[test]
fn data_risk_guidance_empty_report_has_no_candidates_summary() {
    let dir = TempDir::new().unwrap();
    let guidance = data_risk_guidance(dir.path(), &empty_report());
    assert_eq!(guidance["data_risk_focus"]["primary_focus"], "none");
}

#[test]
fn data_risk_guidance_has_layers() {
    let dir = TempDir::new().unwrap();
    let guidance = data_risk_guidance(dir.path(), &empty_report());
    assert!(guidance["layers"].is_object());
}

#[test]
fn data_risk_guidance_has_workspace_observation_layer() {
    let dir = TempDir::new().unwrap();
    let guidance = data_risk_guidance(dir.path(), &empty_report());
    assert!(guidance["layers"]["workspace_observation"].is_object());
    assert_eq!(
        guidance["layers"]["workspace_observation"]["mock_candidate_count"],
        0
    );
    assert_eq!(
        guidance["layers"]["workspace_observation"]["hardcoded_candidate_count"],
        0
    );
}

#[test]
fn data_risk_guidance_has_cleanup_refactor_candidates_layer() {
    let dir = TempDir::new().unwrap();
    let guidance = data_risk_guidance(dir.path(), &empty_report());
    assert!(guidance["layers"]["cleanup_refactor_candidates"].is_object());
}

#[test]
fn data_risk_guidance_empty_report_recommended_flow_mentions_no_candidates() {
    let dir = TempDir::new().unwrap();
    let guidance = data_risk_guidance(dir.path(), &empty_report());
    let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
        .as_array()
        .unwrap();
    assert!(flow[0].as_str().unwrap().contains("No mock or hardcoded"));
}

#[test]
fn data_risk_guidance_hardcoded_report_mentions_hardcoded() {
    let dir = TempDir::new().unwrap();
    let guidance = data_risk_guidance(dir.path(), &report_with_hardcoded());
    assert_eq!(guidance["data_risk_focus"]["primary_focus"], "hardcoded");
    let flow = guidance["layers"]["execution_strategy"]["recommended_flow"]
        .as_array()
        .unwrap();
    assert!(flow[0].as_str().unwrap().contains("hardcoded"));
}

#[test]
fn data_risk_guidance_mock_report_mentions_mock() {
    let dir = TempDir::new().unwrap();
    let guidance = data_risk_guidance(dir.path(), &report_with_mock());
    assert_eq!(guidance["data_risk_focus"]["primary_focus"], "mock");
}

#[test]
fn data_risk_guidance_constraints_boundaries_has_counts() {
    let dir = TempDir::new().unwrap();
    let guidance = data_risk_guidance(dir.path(), &report_with_mock());
    assert_eq!(
        guidance["layers"]["constraints_boundaries"]["mock_candidate_count"],
        1
    );
    assert_eq!(
        guidance["layers"]["constraints_boundaries"]["hardcoded_candidate_count"],
        0
    );
}

#[test]
fn data_risk_guidance_execution_strategy_counts() {
    let dir = TempDir::new().unwrap();
    let report = report_with_hardcoded();
    let guidance = data_risk_guidance(dir.path(), &report);
    assert_eq!(
        guidance["layers"]["execution_strategy"]["hardcoded_candidate_count"],
        1
    );
    assert_eq!(
        guidance["layers"]["execution_strategy"]["review_mock_data_before_cleanup"],
        true
    );
}

#[test]
fn data_risk_guidance_has_suggested_commands() {
    let dir = TempDir::new().unwrap();
    let guidance = data_risk_guidance(dir.path(), &empty_report());
    let cmds = guidance["suggested_commands"].as_array().unwrap();
    assert!(!cmds.is_empty());
}

#[test]
fn data_risk_guidance_has_next_tools() {
    let dir = TempDir::new().unwrap();
    let guidance = data_risk_guidance(dir.path(), &empty_report());
    let tools = guidance["next_tools"].as_array().unwrap();
    assert!(!tools.is_empty());
}
