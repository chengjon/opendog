use super::*;

#[test]
fn data_risk_summary_empty() {
    let summary = execution_strategy_data_risk_focus_summary(&[]);
    assert_eq!(
        summary.data_risk_focus_distribution.to_value(),
        json!({"hardcoded": 0, "mixed": 0, "mock": 0, "none": 0})
    );
    assert_eq!(summary.projects_requiring_hardcoded_review, 0);
    assert_eq!(summary.projects_requiring_mock_review, 0);
    assert_eq!(summary.projects_requiring_mixed_file_review, 0);
}

#[test]
fn data_risk_summary_hardcoded() {
    let overviews = vec![json!({
        "mock_data_summary": {"data_risk_focus": {"primary_focus": "hardcoded"}}
    })];
    let summary = execution_strategy_data_risk_focus_summary(&overviews);
    assert_eq!(summary.projects_requiring_hardcoded_review, 1);
    assert_eq!(summary.projects_requiring_mock_review, 0);
    assert_eq!(summary.projects_requiring_mixed_file_review, 0);
    assert_eq!(summary.data_risk_focus_distribution.hardcoded, 1);
}

#[test]
fn data_risk_summary_mock() {
    let overviews = vec![json!({
        "mock_data_summary": {"data_risk_focus": {"primary_focus": "mock"}}
    })];
    let summary = execution_strategy_data_risk_focus_summary(&overviews);
    assert_eq!(summary.projects_requiring_hardcoded_review, 0);
    assert_eq!(summary.projects_requiring_mock_review, 1);
    assert_eq!(summary.projects_requiring_mixed_file_review, 0);
    assert_eq!(summary.data_risk_focus_distribution.mock, 1);
}

#[test]
fn data_risk_summary_mixed() {
    let overviews = vec![json!({
        "mock_data_summary": {"data_risk_focus": {"primary_focus": "mixed"}}
    })];
    let summary = execution_strategy_data_risk_focus_summary(&overviews);
    assert_eq!(summary.projects_requiring_hardcoded_review, 0);
    assert_eq!(summary.projects_requiring_mock_review, 0);
    assert_eq!(summary.projects_requiring_mixed_file_review, 1);
    assert_eq!(summary.data_risk_focus_distribution.mixed, 1);
}

#[test]
fn data_risk_summary_none_focus() {
    let overviews = vec![json!({
        "mock_data_summary": {"data_risk_focus": {"primary_focus": "none"}}
    })];
    let summary = execution_strategy_data_risk_focus_summary(&overviews);
    assert_eq!(summary.projects_requiring_hardcoded_review, 0);
    assert_eq!(summary.projects_requiring_mock_review, 0);
    assert_eq!(summary.projects_requiring_mixed_file_review, 0);
    assert_eq!(summary.data_risk_focus_distribution.none, 1);
}

#[test]
fn data_risk_summary_missing_focus() {
    // Missing primary_focus field defaults to "none"
    let overviews = vec![json!({
        "mock_data_summary": {"data_risk_focus": {}}
    })];
    let summary = execution_strategy_data_risk_focus_summary(&overviews);
    assert_eq!(summary.data_risk_focus_distribution.none, 1);
}

#[test]
fn data_risk_summary_mixed_overviews() {
    let overviews = vec![
        json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "hardcoded"}}}),
        json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "mock"}}}),
        json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "hardcoded"}}}),
        json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "mixed"}}}),
        json!({"mock_data_summary": {"data_risk_focus": {"primary_focus": "none"}}}),
        json!({"mock_data_summary": {}}),
    ];
    let summary = execution_strategy_data_risk_focus_summary(&overviews);
    assert_eq!(summary.projects_requiring_hardcoded_review, 2);
    assert_eq!(summary.projects_requiring_mock_review, 1);
    assert_eq!(summary.projects_requiring_mixed_file_review, 1);
    assert_eq!(summary.data_risk_focus_distribution.hardcoded, 2);
    assert_eq!(summary.data_risk_focus_distribution.mock, 1);
    assert_eq!(summary.data_risk_focus_distribution.mixed, 1);
    assert_eq!(summary.data_risk_focus_distribution.none, 2);
}
