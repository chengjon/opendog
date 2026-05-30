use super::*;

#[test]
fn test_is_text_like_file_common_source_extensions() {
    assert!(is_text_like_file("main.rs", ""));
    assert!(is_text_like_file("app.py", ""));
    assert!(is_text_like_file("index.js", ""));
    assert!(is_text_like_file("component.tsx", ""));
    assert!(is_text_like_file("app.go", ""));
    assert!(is_text_like_file("main.java", ""));
    assert!(is_text_like_file("main.kt", ""));
    assert!(is_text_like_file("main.swift", ""));
    assert!(is_text_like_file("main.rb", ""));
    assert!(is_text_like_file("main.php", ""));
    assert!(is_text_like_file("main.c", ""));
    assert!(is_text_like_file("main.cc", ""));
    assert!(is_text_like_file("main.cpp", ""));
    assert!(is_text_like_file("main.h", ""));
    assert!(is_text_like_file("main.hpp", ""));
}

#[test]
fn test_is_text_like_file_config_and_data() {
    assert!(is_text_like_file("Cargo.toml", ""));
    assert!(is_text_like_file("package.json", ""));
    assert!(is_text_like_file("config.yaml", ""));
    assert!(is_text_like_file("config.yml", ""));
    assert!(is_text_like_file("notes.txt", ""));
    assert!(is_text_like_file("README.md", ""));
    assert!(is_text_like_file("setup.sh", ""));
    assert!(is_text_like_file(".env", "env"));
    assert!(is_text_like_file("app.ini", ""));
    assert!(is_text_like_file("app.cfg", ""));
    assert!(is_text_like_file("app.conf", ""));
    assert!(is_text_like_file("query.sql", ""));
    assert!(is_text_like_file("app.jsx", ""));
}

#[test]
fn test_is_text_like_file_non_text() {
    assert!(!is_text_like_file("image.png", ""));
    assert!(!is_text_like_file("image.jpg", ""));
    assert!(!is_text_like_file("image.jpeg", ""));
    assert!(!is_text_like_file("binary.exe", ""));
    assert!(!is_text_like_file("archive.zip", ""));
    assert!(!is_text_like_file("data.bin", ""));
    assert!(!is_text_like_file("font.woff", ""));
    assert!(!is_text_like_file("video.mp4", ""));
}

#[test]
fn test_is_text_like_file_uses_file_type_when_provided() {
    // When file_type is non-empty, it takes precedence over the extension
    assert!(is_text_like_file("data.bin", "py"));
    assert!(!is_text_like_file("app.py", "bin"));
}

#[test]
fn test_is_text_like_file_case_insensitive_file_type() {
    assert!(is_text_like_file("script", "PY"));
    assert!(is_text_like_file("script", "Rs"));
    assert!(is_text_like_file("script", "JSON"));
}

#[test]
fn test_is_text_like_file_no_extension_empty_file_type() {
    assert!(!is_text_like_file("Makefile", ""));
    assert!(!is_text_like_file("Dockerfile", ""));
}

// ---- count_keyword_hits ----

#[test]
fn test_count_keyword_hits_multiple() {
    let hits = count_keyword_hits(
        "customer invoice payment",
        &["customer", "invoice", "payment", "missing"],
    );
    assert_eq!(hits, 3);
}

#[test]
fn test_count_keyword_hits_single() {
    let hits = count_keyword_hits("only one match here", &["match", "missing", "absent"]);
    assert_eq!(hits, 1);
}

#[test]
fn test_count_keyword_hits_zero() {
    let hits = count_keyword_hits("nothing relevant", &["customer", "invoice", "payment"]);
    assert_eq!(hits, 0);
}

#[test]
fn test_count_keyword_hits_empty_haystack() {
    let hits = count_keyword_hits("", &["customer", "invoice"]);
    assert_eq!(hits, 0);
}

#[test]
fn test_count_keyword_hits_empty_keywords() {
    let hits = count_keyword_hits("customer invoice", &[]);
    assert_eq!(hits, 0);
}

// ---- path_is_test_only ----
