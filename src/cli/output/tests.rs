use super::*;

// ---- truncate ----

#[test]
fn truncate_short_string_unchanged() {
    assert_eq!(truncate("hello", 10), "hello");
}

#[test]
fn truncate_exact_length_unchanged() {
    assert_eq!(truncate("hello", 5), "hello");
}

#[test]
fn truncate_long_string_truncated_with_ellipsis() {
    let input = "abcdefghij";
    // len=10, max=7 => keep last 4 chars: "efghij" wait...
    // s.len() - max + 3 = 10 - 7 + 3 = 6 => &s[6..] = "ghij" => "...ghij"
    assert_eq!(truncate(input, 7), "...ghij");
}

#[test]
fn truncate_empty_string() {
    assert_eq!(truncate("", 5), "");
}

#[test]
fn truncate_single_char_long_max() {
    assert_eq!(truncate("x", 10), "x");
}

#[test]
fn truncate_produces_max_length_output() {
    let input = "a_very_long_filename_here.rs";
    let result = truncate(input, 15);
    assert_eq!(result.len(), 15);
    assert!(result.starts_with("..."));
}
