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
