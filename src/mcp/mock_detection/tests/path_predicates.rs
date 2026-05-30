use super::*;

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
