use serde_json::Value;
use std::fs;
use std::path::Path;

fn file_exists(root: &Path, name: &str) -> bool {
    root.join(name).exists()
}

fn read_project_file(root: &Path, name: &str) -> Option<String> {
    fs::read_to_string(root.join(name)).ok()
}

fn package_json_has_workspaces(root: &Path) -> bool {
    read_project_file(root, "package.json")
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|value| value.get("workspaces").cloned())
        .map(|workspaces| match workspaces {
            Value::Array(items) => !items.is_empty(),
            Value::Object(fields) => !fields.is_empty(),
            _ => false,
        })
        .unwrap_or(false)
}

pub(super) fn cargo_toml_has_workspace(root: &Path) -> bool {
    read_project_file(root, "Cargo.toml")
        .map(|text| text.contains("[workspace]"))
        .unwrap_or(false)
}

fn node_workspace_marker_exists(root: &Path) -> bool {
    file_exists(root, "pnpm-workspace.yaml")
        || file_exists(root, "lerna.json")
        || file_exists(root, "nx.json")
        || file_exists(root, "turbo.json")
        || package_json_has_workspaces(root)
}

pub(super) fn docs_only_marker_exists(root: &Path) -> bool {
    let has_docs_config = file_exists(root, "mkdocs.yml")
        || file_exists(root, "mkdocs.yaml")
        || file_exists(root, "docusaurus.config.js")
        || file_exists(root, "docusaurus.config.ts");
    let has_docs_content = file_exists(root, "README.md")
        || file_exists(root, "docs/index.md")
        || root.join("docs").is_dir();
    has_docs_config && has_docs_content
}

pub(super) fn workspace_signal_present(root: &Path) -> bool {
    cargo_toml_has_workspace(root)
        || node_workspace_marker_exists(root)
        || file_exists(root, "go.work")
}

pub(super) fn detected_stack_markers(root: &Path) -> Vec<&'static str> {
    let mut markers = Vec::new();
    if file_exists(root, "Cargo.toml") {
        markers.push("rust");
    }
    if file_exists(root, "package.json") || node_workspace_marker_exists(root) {
        markers.push("node");
    }
    if file_exists(root, "pyproject.toml")
        || file_exists(root, "requirements.txt")
        || file_exists(root, "pytest.ini")
        || file_exists(root, "Pipfile")
    {
        markers.push("python");
    }
    if file_exists(root, "go.mod") || file_exists(root, "go.work") {
        markers.push("go");
    }
    markers
}

pub(super) fn manifest_backed_stack_markers(root: &Path) -> Vec<&'static str> {
    let mut markers = Vec::new();
    if file_exists(root, "Cargo.toml") {
        markers.push("rust");
    }
    if file_exists(root, "package.json") {
        markers.push("node");
    }
    if file_exists(root, "pyproject.toml")
        || file_exists(root, "requirements.txt")
        || file_exists(root, "pytest.ini")
        || file_exists(root, "Pipfile")
    {
        markers.push("python");
    }
    if file_exists(root, "go.mod") || file_exists(root, "go.work") {
        markers.push("go");
    }
    markers
}

pub(super) fn node_workspace_has_manifest_context(root: &Path) -> bool {
    file_exists(root, "package.json") && node_workspace_marker_exists(root)
}
