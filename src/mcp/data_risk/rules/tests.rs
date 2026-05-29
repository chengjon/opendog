use super::*;

// ---- normalize_candidate_type ----

#[test]
fn test_normalize_candidate_type_all() {
    assert_eq!(
        normalize_candidate_type(Some("all".to_string())).unwrap(),
        "all"
    );
}

#[test]
fn test_normalize_candidate_type_mock() {
    assert_eq!(
        normalize_candidate_type(Some("mock".to_string())).unwrap(),
        "mock"
    );
}

#[test]
fn test_normalize_candidate_type_hardcoded() {
    assert_eq!(
        normalize_candidate_type(Some("hardcoded".to_string())).unwrap(),
        "hardcoded"
    );
}

#[test]
fn test_normalize_candidate_type_none_defaults_to_all() {
    assert_eq!(normalize_candidate_type(None).unwrap(), "all");
}

#[test]
fn test_normalize_candidate_type_invalid() {
    let result = normalize_candidate_type(Some("bogus".to_string()));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err["error"],
        "candidate_type must be one of: all, mock, hardcoded"
    );
}

// ---- normalize_min_review_priority ----

#[test]
fn test_normalize_min_review_priority_low() {
    assert_eq!(
        normalize_min_review_priority(Some("low".to_string())).unwrap(),
        "low"
    );
}

#[test]
fn test_normalize_min_review_priority_medium() {
    assert_eq!(
        normalize_min_review_priority(Some("medium".to_string())).unwrap(),
        "medium"
    );
}

#[test]
fn test_normalize_min_review_priority_high() {
    assert_eq!(
        normalize_min_review_priority(Some("high".to_string())).unwrap(),
        "high"
    );
}

#[test]
fn test_normalize_min_review_priority_none_defaults_to_low() {
    assert_eq!(normalize_min_review_priority(None).unwrap(), "low");
}

#[test]
fn test_normalize_min_review_priority_invalid() {
    let result = normalize_min_review_priority(Some("urgent".to_string()));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err["error"],
        "min_review_priority must be one of: low, medium, high"
    );
}

// ---- review_priority_score ----

#[test]
fn test_review_priority_score_high() {
    assert_eq!(review_priority_score("high"), 3);
}

#[test]
fn test_review_priority_score_medium() {
    assert_eq!(review_priority_score("medium"), 2);
}

#[test]
fn test_review_priority_score_low() {
    assert_eq!(review_priority_score("low"), 1);
}

#[test]
fn test_review_priority_score_unknown() {
    assert_eq!(review_priority_score("unknown"), 0);
    assert_eq!(review_priority_score(""), 0);
    assert_eq!(review_priority_score("critical"), 0);
}

// ---- path_kind_score ----

#[test]
fn test_path_kind_score_runtime_shared() {
    assert_eq!(path_kind_score("runtime_shared"), 3);
}

#[test]
fn test_path_kind_score_unknown() {
    assert_eq!(path_kind_score("unknown"), 2);
}

#[test]
fn test_path_kind_score_test_only() {
    assert_eq!(path_kind_score("test_only"), 1);
}

#[test]
fn test_path_kind_score_documentation() {
    assert_eq!(path_kind_score("documentation"), 1);
}

#[test]
fn test_path_kind_score_generated_artifact() {
    assert_eq!(path_kind_score("generated_artifact"), 0);
}

#[test]
fn test_path_kind_score_unrecognized() {
    assert_eq!(path_kind_score("other"), 0);
    assert_eq!(path_kind_score(""), 0);
}

// ---- data_risk_severity_score ----

#[test]
fn test_data_risk_severity_score_critical() {
    assert_eq!(data_risk_severity_score("critical"), 4);
}

#[test]
fn test_data_risk_severity_score_high() {
    assert_eq!(data_risk_severity_score("high"), 3);
}

#[test]
fn test_data_risk_severity_score_medium() {
    assert_eq!(data_risk_severity_score("medium"), 2);
}

#[test]
fn test_data_risk_severity_score_low() {
    assert_eq!(data_risk_severity_score("low"), 1);
}

#[test]
fn test_data_risk_severity_score_unknown() {
    assert_eq!(data_risk_severity_score("unknown"), 0);
    assert_eq!(data_risk_severity_score(""), 0);
    assert_eq!(data_risk_severity_score("info"), 0);
}

// ---- data_risk_rule_meta ----

#[test]
fn test_data_risk_rule_meta_known_rules() {
    assert!(data_risk_rule_meta("path.mock_token").is_some());
    assert!(data_risk_rule_meta("content.mock_token").is_some());
    assert!(data_risk_rule_meta("path.test_only").is_some());
    assert!(data_risk_rule_meta("path.generated_artifact").is_some());
    assert!(data_risk_rule_meta("path.documentation").is_some());
    assert!(data_risk_rule_meta("content.business_literal_combo").is_some());
    assert!(data_risk_rule_meta("content.template_placeholder").is_some());
    assert!(data_risk_rule_meta("path.runtime_shared").is_some());
}

#[test]
fn test_data_risk_rule_meta_unknown_rule() {
    assert!(data_risk_rule_meta("nonexistent.rule").is_none());
    assert!(data_risk_rule_meta("").is_none());
}

#[test]
fn test_data_risk_rule_meta_fields() {
    let meta = data_risk_rule_meta("path.mock_token").unwrap();
    assert_eq!(meta.rule, "path.mock_token");
    assert_eq!(meta.group, "path");
    assert_eq!(meta.severity, "low");
    assert!(!meta.description.is_empty());
}

#[test]
fn test_data_risk_rule_meta_all_rules_have_consistent_fields() {
    for meta in DATA_RISK_RULES.iter() {
        assert!(!meta.rule.is_empty(), "rule should not be empty");
        assert!(!meta.group.is_empty(), "group should not be empty");
        assert!(
            matches!(meta.severity, "low" | "medium" | "high" | "critical"),
            "severity should be recognized: got {}",
            meta.severity
        );
        assert!(
            !meta.description.is_empty(),
            "description should not be empty"
        );
    }
}
