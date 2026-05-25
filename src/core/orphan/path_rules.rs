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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn normalize_path_converts_backslashes() {
        assert_eq!(normalize_path(r"src\foo\bar.rs"), "src/foo/bar.rs");
    }

    #[test]
    fn relative_path_strips_root() {
        let root = Path::new("/project");
        let file = Path::new("/project/src/main.rs");
        assert_eq!(relative_path(root, file), Some("src/main.rs".to_string()));
    }

    #[test]
    fn relative_path_returns_none_for_outside() {
        let root = Path::new("/project");
        let file = Path::new("/other/main.rs");
        assert!(relative_path(root, file).is_none());
    }

    #[test]
    fn is_docs_path_matches_docs_and_planning() {
        assert!(is_docs_path("docs/guide.md"));
        assert!(is_docs_path(".planning/ROADMAP.md"));
        assert!(is_docs_path("README.md"));
        assert!(is_docs_path("SPEC.md"));
        assert!(!is_docs_path("src/main.rs"));
    }

    #[test]
    fn is_entrypoint_file_matches_makefile_docker_ci() {
        assert!(is_entrypoint_file("Makefile"));
        assert!(is_entrypoint_file("Dockerfile"));
        assert!(is_entrypoint_file(".github/workflows/ci.yml"));
        assert!(is_entrypoint_file("scripts/deploy.sh"));
        assert!(is_entrypoint_file("docker-compose.yml"));
        assert!(is_entrypoint_file("app.service"));
        assert!(!is_entrypoint_file("src/main.rs"));
    }

    #[test]
    fn is_docs_or_ownership_file_matches_owners_and_docs() {
        assert!(is_docs_or_ownership_file("OWNERS"));
        assert!(is_docs_or_ownership_file("CODEOWNERS"));
        assert!(is_docs_or_ownership_file("src/OWNERS"));
        assert!(is_docs_or_ownership_file("docs/guide.md"));
        assert!(is_docs_or_ownership_file("openspec/v1.md"));
        assert!(!is_docs_or_ownership_file("src/lib.rs"));
    }

    #[test]
    fn is_frontend_source_file_matches_ts_js_vue() {
        assert!(is_frontend_source_file("src/App.tsx"));
        assert!(is_frontend_source_file("web/index.js"));
        assert!(is_frontend_source_file("frontend/View.vue"));
        assert!(is_frontend_source_file("app/main.svelte"));
        assert!(!is_frontend_source_file("src/util.rs"));
        assert!(!is_frontend_source_file("src/styles.css"));
    }
}
