use std::fs;
use std::path::Path;

pub(super) fn relative_path(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root)
        .ok()
        .and_then(|path| path.to_str())
        .map(normalize_path)
}

pub(super) fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

pub(super) fn is_docs_path(rel_path: &str) -> bool {
    let lower = rel_path.to_ascii_lowercase();
    lower == "readme.md"
        || lower.ends_with(".md")
        || lower.starts_with("docs/")
        || lower.starts_with(".planning/")
}

pub(super) fn is_entrypoint_file(rel_path: &str) -> bool {
    let lower = rel_path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(lower.as_str());
    name == "dockerfile"
        || name == "procfile"
        || name == "makefile"
        || lower.starts_with(".github/workflows/")
        || lower.starts_with("scripts/")
        || lower.ends_with(".service")
        || (name.starts_with("docker-compose")
            && (name.ends_with(".yml") || name.ends_with(".yaml")))
        || (name.starts_with("pm2") && name.ends_with(".json"))
}

pub(super) fn is_docs_or_ownership_file(rel_path: &str) -> bool {
    let lower = rel_path.to_ascii_lowercase();
    lower == "owners"
        || lower == "codeowners"
        || lower.ends_with("/owners")
        || lower.ends_with("/codeowners")
        || lower == "architecture/standards.md"
        || lower.starts_with("openspec/")
        || lower.starts_with(".planning/")
        || lower.starts_with("docs/")
        || lower.ends_with(".md")
}

pub(super) fn is_frontend_source_file(rel_path: &str) -> bool {
    let lower = rel_path.to_ascii_lowercase();
    (lower.starts_with("web/")
        || lower.starts_with("frontend/")
        || lower.starts_with("src/")
        || lower.starts_with("app/"))
        && matches!(
            lower.rsplit('.').next(),
            Some("ts" | "tsx" | "js" | "jsx" | "vue" | "svelte")
        )
}

pub(super) fn frontend_marker_exists(root: &Path) -> bool {
    root.join("web").is_dir()
        || root.join("frontend").is_dir()
        || root.join("package.json").exists()
}

pub(super) fn python_marker_exists(root: &Path) -> bool {
    root.join("pyproject.toml").exists()
        || root.join("requirements.txt").exists()
        || root.join("setup.py").exists()
}

pub(super) fn fastapi_marker_exists(root: &Path) -> bool {
    if !python_marker_exists(root) {
        return false;
    }
    let candidates = ["pyproject.toml", "requirements.txt"];
    candidates.iter().any(|name| {
        fs::read_to_string(root.join(name))
            .map(|text| text.to_ascii_lowercase().contains("fastapi"))
            .unwrap_or(false)
    })
}

pub(super) fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
