use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    dirs().join("data")
}

pub fn registry_path() -> PathBuf {
    data_dir().join("registry.db")
}

pub fn global_config_path() -> PathBuf {
    dirs().join("config.json")
}

pub fn daemon_socket_path() -> PathBuf {
    data_dir().join("daemon.sock")
}

pub fn daemon_pid_path() -> PathBuf {
    data_dir().join("daemon.pid")
}

pub fn project_db_path(project_id: &str) -> PathBuf {
    data_dir()
        .join("projects")
        .join(format!("{}.db", project_id))
}

fn dirs() -> PathBuf {
    resolve_dirs(
        std::env::var_os("OPENDOG_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

fn resolve_dirs(opendog_home: Option<PathBuf>, home: Option<PathBuf>) -> PathBuf {
    if let Some(opendog_home) = non_empty_path(opendog_home) {
        return opendog_home;
    }

    non_empty_path(home)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".opendog")
}

fn non_empty_path(path: Option<PathBuf>) -> Option<PathBuf> {
    path.filter(|value| !value.as_os_str().is_empty())
}

pub fn daemon_pid_is_live() -> bool {
    let path = daemon_pid_path();
    let Ok(pid) = std::fs::read_to_string(path) else {
        return false;
    };
    let pid = pid.trim();
    if pid.is_empty() {
        return false;
    }

    #[cfg(unix)]
    {
        std::path::Path::new("/proc").join(pid).exists()
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_dirs;
    use std::path::PathBuf;

    #[test]
    fn opendog_home_overrides_home_derived_default() {
        let root = resolve_dirs(
            Some(PathBuf::from("/tmp/shared-opendog")),
            Some(PathBuf::from("/home/tester")),
        );
        assert_eq!(root, PathBuf::from("/tmp/shared-opendog"));
    }

    #[test]
    fn home_falls_back_to_dot_opendog_when_override_is_missing() {
        let root = resolve_dirs(None, Some(PathBuf::from("/home/tester")));
        assert_eq!(root, PathBuf::from("/home/tester/.opendog"));
    }

    #[test]
    fn registry_path_lives_under_data_dir() {
        assert_eq!(
            super::registry_path(),
            super::data_dir().join("registry.db"),
        );
    }

    #[test]
    fn project_db_path_lives_under_data_projects_dir() {
        assert_eq!(
            super::project_db_path("demo"),
            super::data_dir().join("projects/demo.db"),
        );
    }

    // --- additional project_db_path tests ---

    #[test]
    fn project_db_path_simple_id() {
        let path = super::project_db_path("myproject");
        assert!(path.ends_with("projects/myproject.db"));
    }

    #[test]
    fn project_db_path_dashed_id() {
        let path = super::project_db_path("my-cool-app");
        assert!(path.ends_with("projects/my-cool-app.db"));
    }

    #[test]
    fn project_db_path_underscore_id() {
        let path = super::project_db_path("test_project");
        assert!(path.ends_with("projects/test_project.db"));
    }

    // --- resolve_dirs edge cases ---

    #[test]
    fn resolve_dirs_empty_opendog_home_falls_through() {
        let root = resolve_dirs(
            Some(PathBuf::from("")),
            Some(PathBuf::from("/home/tester")),
        );
        assert_eq!(root, PathBuf::from("/home/tester/.opendog"));
    }

    #[test]
    fn resolve_dirs_both_missing_uses_tmp() {
        let root = resolve_dirs(None, None);
        assert_eq!(root, PathBuf::from("/tmp/.opendog"));
    }

    #[test]
    fn resolve_dirs_both_empty_uses_tmp() {
        let root = resolve_dirs(Some(PathBuf::from("")), Some(PathBuf::from("")));
        assert_eq!(root, PathBuf::from("/tmp/.opendog"));
    }

    // --- non_empty_path tests ---

    #[test]
    fn non_empty_path_some_with_value() {
        let result = super::non_empty_path(Some(PathBuf::from("/opt/data")));
        assert_eq!(result, Some(PathBuf::from("/opt/data")));
    }

    #[test]
    fn non_empty_path_some_empty_returns_none() {
        let result = super::non_empty_path(Some(PathBuf::from("")));
        assert_eq!(result, None);
    }

    #[test]
    fn non_empty_path_none_returns_none() {
        let result = super::non_empty_path(None);
        assert_eq!(result, None);
    }
}
