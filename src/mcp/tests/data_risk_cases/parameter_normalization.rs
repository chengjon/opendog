use super::*;

#[test]
fn normalize_data_risk_filters_reject_invalid_values() {
    assert_eq!(
        normalize_candidate_type(Some("weird".to_string())).unwrap_err(),
        json!({"error": "candidate_type must be one of: all, mock, hardcoded"})
    );
    assert_eq!(
        normalize_min_review_priority(Some("urgent".to_string())).unwrap_err(),
        json!({"error": "min_review_priority must be one of: low, medium, high"})
    );
}
