use super::*;

// ---- is_text_like_file ----

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

#[test]
fn test_path_is_test_only_various_prefixes() {
    assert!(path_is_test_only("tests/unit/test_foo.rs"));
    assert!(path_is_test_only("test/bar.py"));
    assert!(path_is_test_only("__tests__/component.test.js"));
    assert!(path_is_test_only("spec/models/user_spec.rb"));
    assert!(path_is_test_only("specs/example.spec.ts"));
    assert!(path_is_test_only("fixtures/data.json"));
    assert!(path_is_test_only("__fixtures__/sample.json"));
    assert!(path_is_test_only("testdata/input.csv"));
    assert!(path_is_test_only("examples/demo.py"));
    assert!(path_is_test_only("example/sample.txt"));
}

#[test]
fn test_path_is_test_only_negative() {
    assert!(!path_is_test_only("src/main.rs"));
    assert!(!path_is_test_only("lib/customer.py"));
    assert!(!path_is_test_only("config/app.yaml"));
    assert!(!path_is_test_only("README.md"));
}

// ---- path_is_runtime_shared ----

#[test]
fn test_path_is_runtime_shared_various() {
    assert!(path_is_runtime_shared("src/main.rs"));
    assert!(path_is_runtime_shared("app/controllers/user.py"));
    assert!(path_is_runtime_shared("config/settings.yaml"));
    assert!(path_is_runtime_shared("internal/service.go"));
    assert!(path_is_runtime_shared("lib/utils.py"));
    assert!(path_is_runtime_shared("server/handler.rs"));
}

#[test]
fn test_path_is_runtime_shared_negative() {
    assert!(!path_is_runtime_shared("tests/test_main.rs"));
    assert!(!path_is_runtime_shared("docs/guide.md"));
    assert!(!path_is_runtime_shared("dist/bundle.js"));
    assert!(!path_is_runtime_shared("README.md"));
}

// ---- path_is_documentation ----

#[test]
fn test_path_is_documentation_various() {
    // Function expects already-lowercased input
    assert!(path_is_documentation("docs/api.md"));
    assert!(path_is_documentation("doc/guide.txt"));
    assert!(path_is_documentation("documentation/setup.md"));
    assert!(path_is_documentation("operations/runbook.md"));
    assert!(path_is_documentation("runbooks/incident.md"));
    assert!(path_is_documentation("readme.txt"));
    assert!(path_is_documentation("readme.md"));
    assert!(path_is_documentation("changelog.md"));
    assert!(path_is_documentation("changelog.txt"));
}

#[test]
fn test_path_is_documentation_negative() {
    assert!(!path_is_documentation("src/main.rs"));
    assert!(!path_is_documentation("tests/test_foo.rs"));
    assert!(!path_is_documentation("config/app.yaml"));
}

// ---- path_is_generated_artifact ----

#[test]
fn test_path_is_generated_artifact_various() {
    assert!(path_is_generated_artifact("target/debug/opendog"));
    assert!(path_is_generated_artifact("node_modules/lodash/index.js"));
    assert!(path_is_generated_artifact("dist/bundle.js"));
    assert!(path_is_generated_artifact("build/output.o"));
    assert!(path_is_generated_artifact(".next/static/chunk.js"));
    assert!(path_is_generated_artifact("coverage/lcov.info"));
    assert!(path_is_generated_artifact(".turbo/cache.dat"));
}

#[test]
fn test_path_is_generated_artifact_negative() {
    assert!(!path_is_generated_artifact("src/main.rs"));
    assert!(!path_is_generated_artifact("tests/test_foo.rs"));
    assert!(!path_is_generated_artifact("docs/api.md"));
}

// ---- classify_path_kind ----

#[test]
fn test_classify_path_kind_generated_artifact_takes_precedence() {
    assert_eq!(
        classify_path_kind("target/debug/test_main.rs"),
        "generated_artifact"
    );
    assert_eq!(
        classify_path_kind("dist/tests/test_bundle.js"),
        "generated_artifact"
    );
}

#[test]
fn test_classify_path_kind_test_only() {
    assert_eq!(classify_path_kind("tests/unit/test_foo.rs"), "test_only");
    assert_eq!(classify_path_kind("spec/models/user_spec.rb"), "test_only");
}

#[test]
fn test_classify_path_kind_runtime_shared() {
    assert_eq!(classify_path_kind("src/main.rs"), "runtime_shared");
    assert_eq!(classify_path_kind("app/handlers/user.py"), "runtime_shared");
    assert_eq!(classify_path_kind("lib/utils.js"), "runtime_shared");
}

#[test]
fn test_classify_path_kind_documentation() {
    assert_eq!(classify_path_kind("docs/api.md"), "documentation");
    assert_eq!(classify_path_kind("readme.md"), "documentation");
}

#[test]
fn test_classify_path_kind_unknown() {
    assert_eq!(classify_path_kind("Cargo.toml"), "unknown");
    assert_eq!(classify_path_kind("Makefile"), "unknown");
    assert_eq!(classify_path_kind("scripts/setup.sh"), "unknown");
}

#[test]
fn test_classify_path_kind_precedence_generated_over_test() {
    // "build/" contains generated_artifact, "test/" contains test_only
    // generated_artifact has highest precedence
    assert_eq!(
        classify_path_kind("build/test/output"),
        "generated_artifact"
    );
}

#[test]
fn test_classify_path_kind_precedence_test_over_runtime() {
    // "tests/" is test_only, but not runtime_shared (no src/app/etc.)
    assert_eq!(classify_path_kind("tests/src/test_foo.rs"), "test_only");
}

// ---- content_has_template_placeholder ----

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

#[test]
fn classify_path_kind_infrastructure_claude_paths() {
    assert_eq!(
        classify_path_kind(".claude/settings.json"),
        "infrastructure"
    );
    assert_eq!(
        classify_path_kind(".claude/build-checker.json"),
        "infrastructure"
    );
    assert_eq!(
        classify_path_kind(".claude/skills/playwright/references/guide.md"),
        "infrastructure"
    );
    assert_eq!(
        classify_path_kind(".cursor/rules/project.mdc"),
        "infrastructure"
    );
    assert_eq!(
        classify_path_kind(".agents/prompts/review.md"),
        "infrastructure"
    );
}

#[test]
fn infrastructure_paths_are_not_unknown() {
    assert_ne!(classify_path_kind(".claude/settings.json"), "unknown");
}
