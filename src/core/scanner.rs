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
                let target = match fd_info {
                    Ok(info) => info.target,
                    Err(_) => continue,
                };

                let fd_path = match target {
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

                let canonical = match std::fs::canonicalize(&abs_path) {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                // Check if this file is within our project root
                if let Ok(rel) = canonical.strip_prefix(&self.root_path) {
                    let rel_str = rel.to_str().unwrap_or("");
                    if !rel_str.is_empty() && self.snapshot_paths.contains(rel_str) {
                        sightings.push(FileSighting {
                            file_path: rel_str.to_string(),
                            process_name: comm.clone(),
                            pid,
                        });
                    }
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
