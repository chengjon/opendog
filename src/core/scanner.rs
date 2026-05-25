use procfs::process::FDTarget;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct FileSighting {
    pub file_path: String,
    pub process_name: String,
    pub pid: i32,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub sightings: Vec<FileSighting>,
    pub scan_duration_ms: u64,
    pub pids_scanned: usize,
}

pub struct ProcScanner {
    whitelist: Vec<String>,
    snapshot_paths: HashSet<String>,
    root_path: PathBuf,
}

impl ProcScanner {
    pub fn new(root_path: &Path, whitelist: &[String], snapshot_paths: HashSet<String>) -> Self {
        Self {
            whitelist: whitelist.to_vec(),
            snapshot_paths,
            root_path: root_path.to_path_buf(),
        }
    }

    pub fn scan(&self) -> ScanResult {
        let start = std::time::Instant::now();
        let mut sightings = Vec::new();
        let mut seen_fds: HashSet<(i32, i32)> = HashSet::new();
        let mut pids_scanned = 0usize;

        let processes = match procfs::process::all_processes() {
            Ok(procs) => procs,
            Err(e) => {
                warn!("Failed to enumerate /proc: {}", e);
                return ScanResult {
                    sightings: Vec::new(),
                    scan_duration_ms: start.elapsed().as_millis() as u64,
                    pids_scanned: 0,
                };
            }
        };

        for proc_result in processes {
            let process = match proc_result {
                Ok(p) => p,
                Err(_) => continue,
            };
            let pid = process.pid();

            let comm = match process.stat() {
                Ok(stat) => stat.comm,
                Err(_) => continue,
            };

            if !self.is_whitelisted(&comm) {
                continue;
            }

            pids_scanned += 1;
            debug!(pid, comm = %comm, "Scanning whitelisted process");

            let fds = match process.fd() {
                Ok(fds) => fds,
                Err(_) => continue, // Permission denied or process exited
            };

            for fd_info in fds {
                let info = match fd_info {
                    Ok(info) => info,
                    Err(_) => continue,
                };

                if !mark_fd_seen(&mut seen_fds, pid, info.fd) {
                    continue;
                }

                let fd_path = match info.target {
                    FDTarget::Path(p) => p,
                    _ => continue,
                };

                // Resolve to absolute path and check if it's in the project
                let abs_path = if fd_path.is_absolute() {
                    fd_path
                } else {
                    // Try resolving relative to /proc/<pid>/cwd
                    match process.cwd() {
                        Ok(cwd) => cwd.join(&fd_path),
                        Err(_) => continue,
                    }
                };

                if let Some(rel_str) = resolve_snapshot_relative_file_path(
                    &self.root_path,
                    &self.snapshot_paths,
                    &abs_path,
                ) {
                    sightings.push(FileSighting {
                        file_path: rel_str,
                        process_name: comm.clone(),
                        pid,
                    });
                }
            }
        }

        ScanResult {
            sightings,
            scan_duration_ms: start.elapsed().as_millis() as u64,
            pids_scanned,
        }
    }

    fn is_whitelisted(&self, comm: &str) -> bool {
        let comm_lower = comm.to_lowercase();
        self.whitelist.iter().any(|w| {
            let w_lower = w.to_lowercase();
            comm_lower == w_lower || comm_lower.contains(&w_lower)
        })
    }
}

fn mark_fd_seen(seen_fds: &mut HashSet<(i32, i32)>, pid: i32, fd: i32) -> bool {
    seen_fds.insert((pid, fd))
}

fn resolve_snapshot_relative_file_path(
    root_path: &Path,
    snapshot_paths: &HashSet<String>,
    abs_path: &Path,
) -> Option<String> {
    let canonical = std::fs::canonicalize(abs_path).ok()?;
    let metadata = std::fs::metadata(&canonical).ok()?;
    if !metadata.is_file() {
        return None;
    }

    let rel = canonical.strip_prefix(root_path).ok()?;
    let rel_str = rel.to_str().unwrap_or("");
    if rel_str.is_empty() || !snapshot_paths.contains(rel_str) {
        return None;
    }

    Some(rel_str.to_string())
}

pub fn default_process_whitelist() -> Vec<String> {
    vec![
        "claude".to_string(),
        "codex".to_string(),
        "node".to_string(),
        "python".to_string(),
        "python3".to_string(),
        "gpt".to_string(),
        "glm".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::{mark_fd_seen, resolve_snapshot_relative_file_path, ProcScanner, default_process_whitelist};
    use std::collections::HashSet;
    use std::path::Path;

    #[test]
    fn mark_fd_seen_deduplicates_per_pid_and_fd() {
        let mut seen = HashSet::new();

        assert!(mark_fd_seen(&mut seen, 42, 7));
        assert!(!mark_fd_seen(&mut seen, 42, 7));
        assert!(mark_fd_seen(&mut seen, 42, 8));
        assert!(mark_fd_seen(&mut seen, 99, 7));
    }

    #[test]
    fn resolve_snapshot_relative_file_path_ignores_directory_targets() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        let snapshot_paths = HashSet::from([String::from("src/main.rs")]);

        let file =
            resolve_snapshot_relative_file_path(&root, &snapshot_paths, &root.join("src/main.rs"));
        assert_eq!(file.as_deref(), Some("src/main.rs"));

        let dir_target =
            resolve_snapshot_relative_file_path(&root, &snapshot_paths, &root.join("src"));
        assert_eq!(dir_target, None);
    }

    #[test]
    fn is_whitelisted_matches_exact_and_substring() {
        let scanner = ProcScanner::new(
            Path::new("/tmp"),
            &["claude".to_string(), "node".to_string()],
            HashSet::new(),
        );
        assert!(scanner.is_whitelisted("claude"));
        assert!(scanner.is_whitelisted("Claude")); // case-insensitive
        assert!(scanner.is_whitelisted("node"));
        assert!(scanner.is_whitelisted("my-node-addon")); // substring match
        assert!(!scanner.is_whitelisted("python"));
        assert!(!scanner.is_whitelisted("rustc"));
    }

    #[test]
    fn is_whitelisted_empty_whitelist_matches_nothing() {
        let scanner = ProcScanner::new(Path::new("/tmp"), &[], HashSet::new());
        assert!(!scanner.is_whitelisted("claude"));
        assert!(!scanner.is_whitelisted("node"));
    }

    #[test]
    fn default_process_whitelist_contains_expected_entries() {
        let whitelist = default_process_whitelist();
        assert!(whitelist.contains(&"claude".to_string()));
        assert!(whitelist.contains(&"codex".to_string()));
        assert!(whitelist.contains(&"node".to_string()));
        assert!(whitelist.contains(&"python".to_string()));
        assert!(whitelist.contains(&"python3".to_string()));
        assert!(whitelist.contains(&"gpt".to_string()));
        assert!(whitelist.contains(&"glm".to_string()));
        assert_eq!(whitelist.len(), 7);
    }
}
