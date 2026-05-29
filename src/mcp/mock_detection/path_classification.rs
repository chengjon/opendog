use std::path::Path;

pub(super) fn is_text_like_file(file_path: &str, file_type: &str) -> bool {
    let normalized = if file_type.is_empty() {
        Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_lowercase()
    } else {
        file_type.to_lowercase()
    };
    matches!(
        normalized.as_str(),
        "rs" | "toml"
            | "json"
            | "yaml"
            | "yml"
            | "md"
            | "txt"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "py"
            | "go"
            | "java"
            | "kt"
            | "swift"
            | "c"
            | "cc"
            | "cpp"
            | "h"
            | "hpp"
            | "rb"
            | "php"
            | "sh"
            | "env"
            | "ini"
            | "cfg"
            | "conf"
            | "sql"
    )
}

pub(super) fn path_is_test_only(path_lower: &str) -> bool {
    [
        "tests/",
        "test/",
        "__tests__/",
        "spec/",
        "specs/",
        "fixtures/",
        "__fixtures__/",
        "testdata/",
        "examples/",
        "example/",
    ]
    .iter()
    .any(|token| path_lower.contains(token))
}

pub(super) fn path_is_runtime_shared(path_lower: &str) -> bool {
    ["src/", "app/", "config/", "internal/", "lib/", "server/"]
        .iter()
        .any(|token| path_lower.contains(token))
}

pub(super) fn path_is_documentation(path_lower: &str) -> bool {
    [
        "docs/",
        "doc/",
        "documentation/",
        "operations/",
        "runbooks/",
        "readme",
        "changelog",
    ]
    .iter()
    .any(|token| path_lower.contains(token))
}

pub(super) fn path_is_generated_artifact(path_lower: &str) -> bool {
    [
        "target/",
        "node_modules/",
        "dist/",
        "build/",
        ".next/",
        "coverage/",
        ".turbo/",
    ]
    .iter()
    .any(|token| path_lower.contains(token))
}

pub(super) fn path_is_infrastructure(path_lower: &str) -> bool {
    let infra_dirs = [
        ".claude/",
        ".cursor/",
        ".agents/",
        ".amazonq/",
        ".zread/",
        ".vscode/",
        ".idea/",
    ];
    infra_dirs.iter().any(|dir| path_lower.contains(dir))
}

pub(super) fn classify_path_kind(path_lower: &str) -> &'static str {
    if path_is_infrastructure(path_lower) {
        "infrastructure"
    } else if path_is_generated_artifact(path_lower) {
        "generated_artifact"
    } else if path_is_test_only(path_lower) {
        "test_only"
    } else if path_is_runtime_shared(path_lower) {
        "runtime_shared"
    } else if path_is_documentation(path_lower) {
        "documentation"
    } else {
        "unknown"
    }
}
