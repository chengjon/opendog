use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    dirs().join("data")
}

pub fn registry_path() -> PathBuf {
    dirs().join("registry.db")
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
    dirs().join("projects").join(format!("{}.db", project_id))
}

fn dirs() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".opendog")
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
