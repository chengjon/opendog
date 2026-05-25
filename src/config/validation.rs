use std::path::Path;

pub fn validate_project_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

pub fn validate_root_path(path: &Path) -> bool {
    path.is_absolute() && path.is_dir()
}

pub fn is_windows_mount_path(path: &Path) -> bool {
    path.starts_with("/mnt")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // --- validate_project_id ---

    #[test]
    fn validate_project_id_alphanumeric() {
        assert!(validate_project_id("myproject123"));
    }

    #[test]
    fn validate_project_id_with_dashes() {
        assert!(validate_project_id("my-project"));
    }

    #[test]
    fn validate_project_id_with_underscores() {
        assert!(validate_project_id("my_project"));
    }

    #[test]
    fn validate_project_id_mixed() {
        assert!(validate_project_id("My-Project_2024"));
    }

    #[test]
    fn validate_project_id_rejects_spaces() {
        assert!(!validate_project_id("my project"));
    }

    #[test]
    fn validate_project_id_rejects_special_chars() {
        assert!(!validate_project_id("my@project!"));
        assert!(!validate_project_id("project.json"));
        assert!(!validate_project_id("a/b"));
    }

    #[test]
    fn validate_project_id_rejects_empty() {
        assert!(!validate_project_id(""));
    }

    #[test]
    fn validate_project_id_rejects_too_long() {
        let long_id = "a".repeat(65);
        assert!(!validate_project_id(&long_id));
    }

    #[test]
    fn validate_project_id_accepts_max_length() {
        let max_id = "a".repeat(64);
        assert!(validate_project_id(&max_id));
    }

    // --- validate_root_path ---

    #[test]
    fn validate_root_path_rejects_relative() {
        // A relative path that does not exist as a dir
        assert!(!validate_root_path(Path::new("relative/path")));
    }

    #[test]
    fn validate_root_path_rejects_nonexistent_absolute() {
        assert!(!validate_root_path(Path::new("/no/such/directory/ever")));
    }

    #[test]
    fn validate_root_path_accepts_existing_absolute_dir() {
        // /tmp should always exist on Linux
        assert!(validate_root_path(Path::new("/tmp")));
    }

    // --- is_windows_mount_path ---

    #[test]
    fn is_windows_mount_path_mnt_prefix() {
        assert!(is_windows_mount_path(Path::new("/mnt/c")));
        assert!(is_windows_mount_path(Path::new("/mnt/d/project")));
    }

    #[test]
    fn is_windows_mount_path_non_mnt() {
        assert!(!is_windows_mount_path(Path::new("/home/user")));
        assert!(!is_windows_mount_path(Path::new("/opt/project")));
    }

    #[test]
    fn is_windows_mount_path_exact_mnt() {
        assert!(is_windows_mount_path(Path::new("/mnt")));
    }

    #[test]
    fn is_windows_mount_path_relative() {
        assert!(!is_windows_mount_path(Path::new("mnt/c")));
    }
}
