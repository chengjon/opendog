use super::*;

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
