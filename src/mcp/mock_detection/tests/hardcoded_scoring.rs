use super::*;

#[test]
fn test_content_has_template_placeholder_dollar_brace() {
    assert!(content_has_template_placeholder("value is ${name}"));
}

#[test]
fn test_content_has_template_placeholder_mustache() {
    assert!(content_has_template_placeholder("hello {{name}}"));
}

#[test]
fn test_content_has_template_placeholder_angle_your() {
    assert!(content_has_template_placeholder(
        "enter <your_api_key> here"
    ));
}

#[test]
fn test_content_has_template_placeholder_angle_insert() {
    assert!(content_has_template_placeholder("fill in <insert_token>"));
}

#[test]
fn test_content_has_template_placeholder_example_dot_com() {
    assert!(content_has_template_placeholder("email: user@example.com"));
}

#[test]
fn test_content_has_template_placeholder_no_match() {
    assert!(!content_has_template_placeholder(
        "normal text without placeholders"
    ));
    assert!(!content_has_template_placeholder(""));
}

// ---- is_strong_hardcoded_combo ----

#[test]
fn test_is_strong_hardcoded_combo_runtime_shared_meets_threshold() {
    assert!(is_strong_hardcoded_combo("runtime_shared", 2, 2));
    assert!(is_strong_hardcoded_combo("runtime_shared", 5, 3));
}

#[test]
fn test_is_strong_hardcoded_combo_runtime_shared_below_threshold() {
    assert!(!is_strong_hardcoded_combo("runtime_shared", 1, 2));
    assert!(!is_strong_hardcoded_combo("runtime_shared", 2, 1));
    assert!(!is_strong_hardcoded_combo("runtime_shared", 0, 0));
}

#[test]
fn test_is_strong_hardcoded_combo_test_only_always_false() {
    assert!(!is_strong_hardcoded_combo("test_only", 10, 10));
}

#[test]
fn test_is_strong_hardcoded_combo_generated_artifact_always_false() {
    assert!(!is_strong_hardcoded_combo("generated_artifact", 10, 10));
}

#[test]
fn test_is_strong_hardcoded_combo_other_classification_higher_threshold() {
    // "unknown" or "documentation" requires business_hits >= 3
    assert!(is_strong_hardcoded_combo("unknown", 3, 2));
    assert!(is_strong_hardcoded_combo("documentation", 3, 2));
    assert!(!is_strong_hardcoded_combo("unknown", 2, 2));
    assert!(!is_strong_hardcoded_combo("documentation", 2, 2));
}

// ---- hardcoded_review_priority ----

#[test]
fn test_hardcoded_review_priority_runtime_shared_no_template() {
    assert_eq!(hardcoded_review_priority("runtime_shared", false), "high");
}

#[test]
fn test_hardcoded_review_priority_runtime_shared_with_template() {
    // runtime_shared with template -> falls through to has_template_placeholder check -> "low"
    assert_eq!(hardcoded_review_priority("runtime_shared", true), "low");
}

#[test]
fn test_hardcoded_review_priority_documentation() {
    assert_eq!(hardcoded_review_priority("documentation", false), "low");
    assert_eq!(hardcoded_review_priority("documentation", true), "low");
}

#[test]
fn test_hardcoded_review_priority_has_template_placeholder() {
    assert_eq!(hardcoded_review_priority("unknown", true), "low");
}

#[test]
fn test_hardcoded_review_priority_default_medium() {
    assert_eq!(hardcoded_review_priority("unknown", false), "medium");
    assert_eq!(hardcoded_review_priority("test_only", false), "medium");
}

// ---- hardcoded_confidence ----

#[test]
fn test_hardcoded_confidence_runtime_shared_no_template() {
    assert_eq!(hardcoded_confidence("runtime_shared", false), "high");
}

#[test]
fn test_hardcoded_confidence_runtime_shared_with_template() {
    // runtime_shared with template -> falls through to has_template_placeholder check -> "low"
    assert_eq!(hardcoded_confidence("runtime_shared", true), "low");
}

#[test]
fn test_hardcoded_confidence_documentation() {
    assert_eq!(hardcoded_confidence("documentation", false), "low");
    assert_eq!(hardcoded_confidence("documentation", true), "low");
}

#[test]
fn test_hardcoded_confidence_has_template_placeholder() {
    assert_eq!(hardcoded_confidence("unknown", true), "low");
}

#[test]
fn test_hardcoded_confidence_default_medium() {
    assert_eq!(hardcoded_confidence("unknown", false), "medium");
    assert_eq!(hardcoded_confidence("test_only", false), "medium");
}

// ---- matched_keywords ----
