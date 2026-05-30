use super::*;

#[test]
fn test_matched_keywords_basic() {
    let result = matched_keywords(
        "mock fixture stub",
        &["mock", "fixture", "stub", "fake"],
        10,
    );
    assert_eq!(result, vec!["mock", "fixture", "stub"]);
}

#[test]
fn test_matched_keywords_respects_limit() {
    let result = matched_keywords(
        "mock fixture stub fake",
        &["mock", "fixture", "stub", "fake"],
        2,
    );
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "mock");
    assert_eq!(result[1], "fixture");
}

#[test]
fn test_matched_keywords_no_matches() {
    let result = matched_keywords("nothing here", &["mock", "fixture"], 5);
    assert!(result.is_empty());
}

#[test]
fn test_matched_keywords_empty_haystack() {
    let result = matched_keywords("", &["mock", "fixture"], 5);
    assert!(result.is_empty());
}

#[test]
fn test_matched_keywords_empty_keywords() {
    let result = matched_keywords("mock fixture", &[], 5);
    assert!(result.is_empty());
}

// ---- previous_char_boundary ----

#[test]
fn test_previous_char_boundary_ascii_within_bounds() {
    assert_eq!(previous_char_boundary("hello world", 3), 3);
}

#[test]
fn test_previous_char_boundary_at_start() {
    assert_eq!(previous_char_boundary("hello", 0), 0);
}

#[test]
fn test_previous_char_boundary_exceeds_length() {
    assert_eq!(previous_char_boundary("hello", 100), 5);
}

#[test]
fn test_previous_char_boundary_zero() {
    assert_eq!(previous_char_boundary("hello", 0), 0);
}

#[test]
fn test_previous_char_boundary_finds_boundary_in_multibyte() {
    // "cafe\u{301}" = "caf\u{e9}" = "cafe" with combining accent
    // Actually let's use a simpler multibyte case
    let s = "héllo"; // 'é' is 2 bytes in UTF-8
                     // 'h' = 0..1, 'é' = 1..3, 'l' = 3..4, 'l' = 4..5, 'o' = 5..6
                     // index 2 is mid-character in 'é'
    assert_eq!(previous_char_boundary(s, 2), 1);
    // index 1 is a valid boundary
    assert_eq!(previous_char_boundary(s, 1), 1);
    // index 3 is valid
    assert_eq!(previous_char_boundary(s, 3), 3);
}

#[test]
fn test_previous_char_boundary_empty_string() {
    assert_eq!(previous_char_boundary("", 0), 0);
    assert_eq!(previous_char_boundary("", 5), 0);
}

// ---- next_char_boundary ----

#[test]
fn test_next_char_boundary_ascii_within_bounds() {
    assert_eq!(next_char_boundary("hello world", 3), 3);
}

#[test]
fn test_next_char_boundary_at_end() {
    assert_eq!(next_char_boundary("hello", 5), 5);
}

#[test]
fn test_next_char_boundary_exceeds_length() {
    assert_eq!(next_char_boundary("hello", 100), 5);
}

#[test]
fn test_next_char_boundary_finds_boundary_in_multibyte() {
    let s = "héllo"; // 'é' is 2 bytes: positions 1..3
                     // index 2 is mid-character, should advance to 3
    assert_eq!(next_char_boundary(s, 2), 3);
    // index 1 is a valid boundary
    assert_eq!(next_char_boundary(s, 1), 1);
}

#[test]
fn test_next_char_boundary_empty_string() {
    assert_eq!(next_char_boundary("", 0), 0);
    assert_eq!(next_char_boundary("", 5), 0);
}

// ---- discounted_weak_literal_hits ----

#[test]
fn test_discounted_weak_literal_hits_even() {
    assert_eq!(discounted_weak_literal_hits(4), 2);
    assert_eq!(discounted_weak_literal_hits(0), 0);
    assert_eq!(discounted_weak_literal_hits(2), 1);
}

#[test]
fn test_discounted_weak_literal_hits_odd_truncates() {
    assert_eq!(discounted_weak_literal_hits(5), 2);
    assert_eq!(discounted_weak_literal_hits(1), 0);
    assert_eq!(discounted_weak_literal_hits(3), 1);
}

#[test]
fn test_discounted_weak_literal_hits_large() {
    assert_eq!(discounted_weak_literal_hits(100), 50);
}

// ---- path_is_infrastructure / infrastructure classification ----
